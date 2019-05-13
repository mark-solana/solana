use crate::components::{Cache, Key, Location, LogFile, Value};
use crate::entities::Column;
use crate::systems::Logging;

use evmap::ReadHandle;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use specs::prelude::*;

//mod components;
//mod entities;
//mod storage;
//mod systems;
mod actors;
mod messages;
mod values;

#[cfg(test)]
#[macro_use]
extern crate proptest;
#[macro_use]
extern crate serde_derive;
extern crate shred;
#[macro_use]
extern crate specs_derive;

//const DEFAULT_TABLE_SIZE: usize = 64 * 1024 * 1024;
const DEFAULT_MEM_SIZE: usize = 64 * 1024 * 1024;

type Result<T> = std::result::Result<T, Box<std::error::Error>>;

#[derive(Debug, PartialEq, Clone)]
pub struct Config {
    pub in_memory: bool,
    pub max_cache_size: usize,
}

#[derive(Clone)]
pub struct KvStore {
    root: PathBuf,
    config: Config,
    //world: Arc<World>,
    //reader: ReadHandle<Key, Value>,
}

//impl KvStore

impl Default for Config {
    fn default() -> Self {
        Config {
            in_memory: false,
            max_cache_size: DEFAULT_MEM_SIZE,
        }
    }
}
