use specs::prelude::*;

#[derive(Debug, Clone)]
pub struct Connection(pub(crate) Entity);

#[derive(Debug)]
pub struct Column(pub(crate) Entity);

#[derive(Debug, Clone)]
pub struct Table(pub(crate) Entity);

#[derive(Debug, Clone)]
pub struct Reader(pub(crate) Entity);

#[derive(Debug, Clone)]
pub struct Writer(pub(crate) Entity);

#[derive(Debug, Clone)]
pub struct Record(pub(crate) Entity);
