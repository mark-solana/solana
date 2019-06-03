use crate::common::Record;
use crossbeam::queue::SegQueue;
use specs::{Component, DenseVecStorage};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

#[derive(Component, Debug)]
pub struct LogDir {
    pub location: PathBuf,
}

#[derive(Component, Default)]
pub struct Queue {
    pub queue: SegQueue<Record>,
}

#[derive(Component)]
pub struct Sink {
    pub writer: BufWriter<Box<dyn Write + Send + Sync>>,
    pub file: Option<File>,
}

impl Queue {
    pub fn new() -> Self {
        Queue {
            queue: SegQueue::new(),
        }
    }
}

impl Sink {
    pub fn new<W>(writer: W, file: Option<File>) -> Self
    where
        W: Write + Send + Sync + 'static,
    {
        let obj: Box<dyn Write + Send + Sync> = Box::new(writer);
        let writer = BufWriter::new(obj);
        Sink { writer, file }
    }
}
