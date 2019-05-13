use crate::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    Started,
    Initialized,
    Recovered,
    GotWrite,
    WriteCommited,
    WriteFailed,
    TableCreated,
    TableDestroyed,
    TxCreated,
    TxCommitted,
    TxFailed,
    MemTableFilled,
    MemTableFlushed,
    GotRead,
    ReadServed,
    ReadFailed,
}
