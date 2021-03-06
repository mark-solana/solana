#![feature(test)]
extern crate test;

use solana::entry::EntrySlice;
use solana::entry::{next_entry_mut, Entry};
use solana_sdk::hash::{hash, Hash};
use solana_sdk::signature::{Keypair, KeypairUtil};
use solana_sdk::system_transaction;
use test::Bencher;

const NUM_HASHES: u64 = 400;
const NUM_ENTRIES: usize = 800;

#[bench]
fn bench_poh_verify_ticks(bencher: &mut Bencher) {
    let zero = Hash::default();
    let mut cur_hash = hash(&zero.as_ref());
    let start = *&cur_hash;

    let mut ticks: Vec<Entry> = Vec::with_capacity(NUM_ENTRIES);
    for _ in 0..NUM_ENTRIES {
        ticks.push(next_entry_mut(&mut cur_hash, NUM_HASHES, vec![]));
    }

    bencher.iter(|| {
        ticks.verify(&start);
    })
}

#[bench]
fn bench_poh_verify_transaction_entries(bencher: &mut Bencher) {
    let zero = Hash::default();
    let mut cur_hash = hash(&zero.as_ref());
    let start = *&cur_hash;

    let keypair1 = Keypair::new();
    let pubkey1 = keypair1.pubkey();

    let mut ticks: Vec<Entry> = Vec::with_capacity(NUM_ENTRIES);
    for _ in 0..NUM_ENTRIES {
        let tx = system_transaction::create_user_account(&keypair1, &pubkey1, 42, cur_hash);
        ticks.push(next_entry_mut(&mut cur_hash, NUM_HASHES, vec![tx]));
    }

    bencher.iter(|| {
        ticks.verify(&start);
    })
}
