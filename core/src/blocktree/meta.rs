use crate::{
    erasure::{NUM_CODING, NUM_DATA},
    packet::CodingHeader,
};
use solana_metrics::datapoint;
use std::borrow::Borrow;

#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
// The Meta column family
pub struct SlotMeta {
    // The number of slots above the root (the genesis block). The first
    // slot has slot 0.
    pub slot: u64,
    // The total number of consecutive blobs starting from index 0
    // we have received for this slot.
    pub consumed: u64,
    // The index *plus one* of the highest blob received for this slot.  Useful
    // for checking if the slot has received any blobs yet, and to calculate the
    // range where there is one or more holes: `(consumed..received)`.
    pub received: u64,
    // The index of the blob that is flagged as the last blob for this slot.
    pub last_index: u64,
    // The slot height of the block this one derives from.
    pub parent_slot: u64,
    // The list of slot heights, each of which contains a block that derives
    // from this one.
    pub next_slots: Vec<u64>,
    // True if this slot is full (consumed == last_index + 1) and if every
    // slot that is a parent of this slot is also connected.
    pub is_connected: bool,
}

impl SlotMeta {
    pub fn is_full(&self) -> bool {
        // last_index is std::u64::MAX when it has no information about how
        // many blobs will fill this slot.
        // Note: A full slot with zero blobs is not possible.
        if self.last_index == std::u64::MAX {
            return false;
        }

        // Should never happen
        if self.consumed > self.last_index + 1 {
            datapoint!(
                "blocktree_error",
                (
                    "error",
                    format!(
                        "Observed a slot meta with consumed: {} > meta.last_index + 1: {}",
                        self.consumed,
                        self.last_index + 1
                    ),
                    String
                )
            );
        }

        self.consumed == self.last_index + 1
    }

    pub fn is_parent_set(&self) -> bool {
        self.parent_slot != std::u64::MAX
    }

    pub(in crate::blocktree) fn new(slot: u64, parent_slot: u64) -> Self {
        SlotMeta {
            slot,
            consumed: 0,
            received: 0,
            parent_slot,
            next_slots: vec![],
            is_connected: slot == 0,
            last_index: std::u64::MAX,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq)]
/// Erasure coding information
pub struct ErasureMeta {
    /// Session information for recovery
    header: CodingHeader,
    /// Bitfield representing presence/absence of data blobs
    data: u64,
    /// Bitfield representing presence/absence of coding blobs
    coding: u64,
}

#[derive(Debug, PartialEq)]
pub enum ErasureMetaStatus {
    CanRecover,
    DataFull,
    StillNeed(usize),
    Ambiguous,
}

impl ErasureMeta {
    pub(in crate::blocktree) fn new() -> ErasureMeta {
        ErasureMeta { ..Self::default() }
    }

    pub fn from_header(header: CodingHeader) -> Self {
        ErasureMeta {
            coding: 0,
            data: 0,
            header,
        }
    }

    pub fn session_info(&self) -> CodingHeader {
        self.header
    }

    pub fn set_session_info(&mut self, header: CodingHeader) {
        self.header = header;
    }

    pub fn status(&self) -> ErasureMetaStatus {
        if !self.header.encoded {
            return ErasureMetaStatus::Ambiguous;
        }
        self.num_data();
        self.header.data_count;
        let (data_missing, coding_missing) = (
            self.header.data_count - self.num_data(),
            self.header.parity_count - self.num_coding(),
        );

        if data_missing > 0 && data_missing + coding_missing <= self.header.parity_count {
            assert!(self.header.shard_size != 0);
            ErasureMetaStatus::CanRecover
        } else if data_missing == 0 {
            ErasureMetaStatus::DataFull
        } else {
            ErasureMetaStatus::StillNeed(data_missing + coding_missing - self.header.parity_count)
        }
    }

    pub fn num_coding(&self) -> usize {
        self.coding.count_ones() as usize
    }

    pub fn num_data(&self) -> usize {
        self.data.count_ones() as usize
    }

    pub fn is_coding_present(&self, index: u64) -> bool {
        self.coding & (1 << index) != 0
    }

    pub fn set_size(&mut self, size: usize) {
        self.header.shard_size = size;
    }

    pub fn size(&self) -> usize {
        self.header.shard_size
    }

    pub fn set_coding_present(&mut self, index: u64, present: bool) {
        (index, present);
        if present {
            self.coding |= 1 << index;
        } else {
            self.coding &= !(1 << index);
        }
    }

    pub fn is_data_present(&self, index: u64) -> bool {
        let position = self.data_index_in_set(index);
        self.data & (1 << position) != 0
    }

    pub fn set_data_present(&mut self, index: u64, present: bool) {
        let position = self.data_index_in_set(index);
        (index, position);
        if present {
            self.data |= 1 << position;
        } else {
            self.data &= !(1 << position);
        }
    }

    pub fn set_data_multi<I, Idx>(&mut self, indexes: I, present: bool)
    where
        I: IntoIterator<Item = Idx>,
        Idx: Borrow<u64>,
    {
        for index in indexes.into_iter() {
            self.set_data_present(*index.borrow(), present);
        }
    }

    pub fn set_coding_multi<I, Idx>(&mut self, indexes: I, present: bool)
    where
        I: IntoIterator<Item = Idx>,
        Idx: Borrow<u64>,
    {
        for index in indexes.into_iter() {
            self.set_coding_present(*index.borrow(), present);
        }
    }

    pub fn data_index_in_set(&self, index: u64) -> u64 {
        index - self.header.start_index
    }

    pub fn coding_index_in_set(&self, index: u64) -> u64 {
        index + self.header.data_count as u64
    }

    pub fn set_index(&self) -> Option<u64> {
        self.header.set_index
    }

    pub fn start_index(&self) -> u64 {
        self.header.start_index
    }

    /// returns a tuple of (data_end, coding_end)
    pub fn end_indexes(&self) -> (u64, u64) {
        let start = self.header.start_index;
        (
            start + self.header.data_count as u64,
            self.header.parity_count as u64,
        )
    }
}

#[test]
fn test_meta_coding_present() {
    let mut e_meta = ErasureMeta::default();

    e_meta.set_coding_multi(0..NUM_CODING as u64, true);
    for i in 0..NUM_CODING as u64 {
        assert_eq!(e_meta.is_coding_present(i), true);
    }
    for i in NUM_CODING as u64..NUM_DATA as u64 {
        assert_eq!(e_meta.is_coding_present(i), false);
    }

    e_meta.header.set_index = Some(17);
    let start_idx = e_meta.header.start_index;
    e_meta.set_coding_multi(start_idx..start_idx + NUM_CODING as u64, true);

    for i in start_idx..start_idx + NUM_CODING as u64 {
        e_meta.set_coding_present(i, true);
        assert_eq!(e_meta.is_coding_present(i), true);
    }
    for i in start_idx + NUM_CODING as u64..start_idx + NUM_DATA as u64 {
        assert_eq!(e_meta.is_coding_present(i), false);
    }
}

#[test]
fn test_erasure_meta_status() {
    use rand::{seq::SliceRandom, thread_rng};
    // Local constansts just used to avoid repetitive casts
    const N_DATA: u64 = crate::erasure::NUM_DATA as u64;
    const N_CODING: u64 = crate::erasure::NUM_CODING as u64;

    let mut e_meta = ErasureMeta::default();
    let mut rng = thread_rng();
    let data_indexes: Vec<u64> = (0..N_DATA).collect();
    let coding_indexes: Vec<u64> = (0..N_CODING).collect();

    assert_eq!(e_meta.status(), ErasureMetaStatus::StillNeed(NUM_DATA));

    e_meta.set_data_multi(0..N_DATA, true);

    assert_eq!(e_meta.status(), ErasureMetaStatus::DataFull);

    e_meta.header.shard_size = 1;
    e_meta.set_coding_multi(0..N_CODING, true);

    assert_eq!(e_meta.status(), ErasureMetaStatus::DataFull);

    for &idx in data_indexes.choose_multiple(&mut rng, NUM_CODING) {
        e_meta.set_data_present(idx, false);

        assert_eq!(e_meta.status(), ErasureMetaStatus::CanRecover);
    }

    e_meta.set_data_multi(0..N_DATA, true);

    for &idx in coding_indexes.choose_multiple(&mut rng, NUM_CODING) {
        e_meta.set_coding_present(idx, false);

        assert_eq!(e_meta.status(), ErasureMetaStatus::DataFull);
    }
}

#[test]
fn test_meta_data_present() {
    let mut e_meta = ErasureMeta::default();

    e_meta.set_data_multi(0..NUM_DATA as u64, true);
    for i in 0..NUM_DATA as u64 {
        assert_eq!(e_meta.is_data_present(i), true);
    }
    for i in NUM_DATA as u64..2 * NUM_DATA as u64 {
        assert_eq!(e_meta.is_data_present(i), false);
    }

    e_meta.header.set_index = Some(23);
    let start_idx = e_meta.start_index();
    e_meta.set_data_multi(start_idx..start_idx + NUM_DATA as u64, true);

    for i in start_idx..start_idx + NUM_DATA as u64 {
        assert_eq!(e_meta.is_data_present(i), true);
    }
    for i in start_idx - NUM_DATA as u64..start_idx {
        assert_eq!(e_meta.is_data_present(i), false);
    }
}
