use crate::log::{
    components::{LogDir, Queue, Sink},
    Config, LogRecord,
};
use chrono::prelude::*;
use crc::crc32::{Digest, Hasher32, IEEE};
use specs::prelude::*;
use std::{
    fs::File,
    io::{self, BufReader, Seek, SeekFrom, Write},
};

pub struct Log;
pub struct Recover;
pub struct Rotate;

#[derive(SystemData)]
pub struct LogData<'a> {
    pub config: ReadExpect<'a, Config>,
    pub sinks: WriteStorage<'a, Sink>,
    pub queues: WriteStorage<'a, Queue>,
}

#[derive(SystemData)]
pub struct RecoverData<'a> {
    pub config: ReadExpect<'a, Config>,
    pub sinks: WriteStorage<'a, Sink>,
    pub queues: WriteStorage<'a, Queue>,
}

#[derive(SystemData)]
pub struct RotateData<'a> {
    pub config: ReadExpect<'a, Config>,
    pub dirs: WriteStorage<'a, LogDir>,
    pub sinks: WriteStorage<'a, Sink>,
}

impl<'a> System<'a> for Log {
    type SystemData = LogData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (queue, sink) in (&mut data.queues, &mut data.sinks).join() {
            while let Ok(record) = queue.queue.pop() {
                let log_record = LogRecord::new(record);

                bincode::serialize_into(&mut sink.writer, &log_record).unwrap();
                sink.writer.flush().unwrap();

                if data.config.fsync {
                    if let Some(ref mut file) = sink.file {
                        file.sync_all().unwrap();
                    }
                }
            }
        }
    }
}

impl<'a> System<'a> for Recover {
    type SystemData = RecoverData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (queue, sink) in (&mut data.queues, &mut data.sinks).join() {
            if let Some(ref mut file) = sink.file {
                let pos = file.seek(SeekFrom::Start(0)).unwrap();
                assert_eq!(pos, 0);

                let mut reader = BufReader::new(file);

                while let Ok(log_record) = bincode::deserialize_from::<_, LogRecord>(&mut reader) {
                    let LogRecord {
                        checksum: crc,
                        record,
                    } = log_record;

                    let mut digest = Digest::new(IEEE);
                    digest.write(&record.key.bytes);
                    if let Some(ref bytes) = record.value {
                        digest.write(&bytes);
                    }

                    let computed_crc = digest.sum32();
                    if computed_crc != crc {
                        continue;
                    }

                    queue.queue.push(record);
                }

                let file = reader.into_inner();
                file.set_len(0).unwrap();
            }
        }
    }
}

impl<'a> System<'a> for Rotate {
    type SystemData = RotateData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (dir, sink) in (&mut data.dirs, &mut data.sinks).join() {
            if let Some(ref mut file) = sink.file {
                let metadata = file.metadata().unwrap();

                if metadata.len() as usize >= data.config.max_file_size {
                    let name = format!("{}.log", Utc::now());
                    let path = dir.location.join(name);
                    let mut new_file = File::create(&path).expect("Couldn't open log file");

                    file.seek(SeekFrom::Start(0)).unwrap();
                    io::copy(file, &mut new_file).expect("Couldn't rotate log");
                    file.set_len(0).unwrap();
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::common::{Key, Record};
    use bytes::Bytes;
    use specs::DispatcherBuilder;
    use tempfile::{tempdir, tempfile};

    #[test]
    fn test_log() {
        let records = (0..10)
            .map(|i| {
                let key = Key::from(vec![i; 24]);
                let bytes = Bytes::from(vec![i as u8; 64]);

                Record::new(key, Some(bytes))
            })
            .collect::<Vec<Record>>();

        let mut world = World::default();

        world.register::<Sink>();
        world.register::<Queue>();
        world.add_resource(Config::default());

        let file = tempfile().unwrap();
        let dup = file.try_clone().unwrap();

        let col = world
            .create_entity()
            .with(Queue::default())
            .with(Sink::new(file, Some(dup)))
            .build();

        world.maintain();

        {
            let mut storage = world.write_storage::<Queue>();

            let queue = storage.get_mut(col).unwrap();
            for record in &records {
                queue.queue.push(record.clone());
            }
        }

        let mut log = Log;
        let mut recover = Recover;

        log.run_now(&world.res);
        world.maintain();
        recover.run_now(&world.res);
        world.maintain();

        {
            let mut storage = world.write_storage::<Queue>();
            let queue = storage.get_mut(col).unwrap();

            let mut items = vec![];

            while let Ok(record) = queue.queue.pop() {
                items.push(record);
            }

            assert_eq!(items.len(), records.len());
            assert_eq!(items, records);
        }
    }

    #[test]
    fn test_rotate() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.log");
        let config = Config {
            max_file_size: 1024,
            ..Config::default()
        };

        let records = (0..200)
            .map(|i| {
                let key = Key::from(vec![i; 24]);
                let bytes = Bytes::from(vec![i as u8; 64]);

                Record::new(key, Some(bytes))
            })
            .collect::<Vec<Record>>();

        let mut world = World::default();

        world.register::<Sink>();
        world.register::<Queue>();
        world.register::<LogDir>();
        world.add_resource(config);

        let file = File::create(&file_path).unwrap();
        let dup = file.try_clone().unwrap();

        let col = world
            .create_entity()
            .with(Queue::default())
            .with(Sink::new(file, Some(dup)))
            .with(LogDir {
                location: dir.path().to_path_buf(),
            })
            .build();

        world.maintain();

        {
            let mut storage = world.write_storage::<Queue>();

            let queue = storage.get_mut(col).unwrap();
            for record in &records {
                queue.queue.push(record.clone());
            }
        }

        let mut dispatcher = DispatcherBuilder::new()
            .with(Rotate, "log_rotation", &[])
            .with(Log, "logging", &["log_rotation"])
            .with(Recover, "recovery", &["log_rotation"])
            .build();

        dispatcher.dispatch(&mut world.res);
        world.maintain();

        let mut storage = world.write_storage::<Queue>();
        let queue = storage.get_mut(col).unwrap();
        assert!(queue.queue.is_empty());
    }
}
