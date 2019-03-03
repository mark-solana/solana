#[cfg(feature = "bpf_c")]
use solana_sdk::bpf_loader;
#[cfg(any(feature = "bpf_c", feature = "bpf_rust"))]
use std::env;
#[cfg(any(feature = "bpf_c", feature = "bpf_rust"))]
use std::fs::File;
#[cfg(any(feature = "bpf_c", feature = "bpf_rust"))]
use std::io::Read;
#[cfg(any(feature = "bpf_c", feature = "bpf_rust"))]
use std::path::PathBuf;

/// BPF program file extension
#[cfg(any(feature = "bpf_c", feature = "bpf_rust"))]
const PLATFORM_FILE_EXTENSION_BPF: &str = "so";
/// Create a BPF program file name
#[cfg(any(feature = "bpf_c", feature = "bpf_rust"))]
fn create_bpf_path(name: &str) -> PathBuf {
    let mut pathbuf = {
        let current_exe = env::current_exe().unwrap();
        PathBuf::from(current_exe.parent().unwrap().parent().unwrap())
    };
    pathbuf.push("bpf/");
    pathbuf.push(name);
    pathbuf.set_extension(PLATFORM_FILE_EXTENSION_BPF);
    pathbuf
}

#[cfg(feature = "bpf_c")]
#[test]
fn test_program_bpf_c_noop() {
    solana_logger::setup();

    let mut file = File::open(create_bpf_path("noop")).expect("file open failed");
    let mut elf = Vec::new();
    file.read_to_end(&mut elf).unwrap();

    let (genesis_block, mint_keypair) = GenesisBlock::new(50);
    let bank = Bank::new(&genesis_block);

    // Call user program
    let program_id = load_program(&bank, &mint_keypair, bpf_loader::id(), elf);
    let tx = Transaction::new(
        &mint_keypair,
        &[],
        program_id,
        &vec![1u8],
        bank.last_blockhash(),
        0,
    );
    bank.process_transaction(&tx).unwrap();
    assert_eq!(bank.get_signature_status(&tx.signatures[0]), Some(Ok(())));
}

#[cfg(feature = "bpf_c")]
#[test]
fn test_program_bpf_c() {
    solana_logger::setup();

    let programs = [
        "bpf_to_bpf",
        "multiple_static",
        "noop",
        "noop++",
        "relative_call",
        "struct_pass",
        "struct_ret",
    ];
    for program in programs.iter() {
        println!("Test program: {:?}", program);
        let mut file = File::open(create_bpf_path(program)).expect("file open failed");
        let mut elf = Vec::new();
        file.read_to_end(&mut elf).unwrap();

        let (genesis_block, mint_keypair) = GenesisBlock::new(50);
        let bank = Bank::new(&genesis_block);

        let loader_id = load_program(
            &bank,
            &mint_keypair,
            native_loader::id(),
            "solana_bpf_loader".as_bytes().to_vec(),
        );

        // Call user program
        let program_id = load_program(&bank, &mint_keypair, loader_id, elf);
        let tx = Transaction::new(
            &mint_keypair,
            &[],
            program_id,
            &vec![1u8],
            bank.last_blockhash(),
            0,
        );
        bank.process_transaction(&tx).unwrap();
        assert_eq!(bank.get_signature_status(&tx.signatures[0]), Some(Ok(())));
    }
}

// Cannot currently build the Rust BPF program as part
// of the rest of the build due to recursive `cargo build` causing
// a build deadlock.  Therefore you must build the Rust programs
// yourself first by calling `make all` in the Rust BPF program's directory
#[cfg(feature = "bpf_rust")]
#[test]
fn test_program_bpf_rust() {
    solana_logger::setup();

    let programs = ["solana_bpf_rust_noop"];
    for program in programs.iter() {
        println!("Test program: {:?}", program);
        let mut file = File::open(create_bpf_path(program)).expect("file open failed");
        let mut elf = Vec::new();
        file.read_to_end(&mut elf).unwrap();

        let (genesis_block, mint_keypair) = GenesisBlock::new(50);
        let bank = Bank::new(&genesis_block);
        let loader_id = load_program(
            &bank,
            &mint_keypair,
            native_loader::id(),
            "solana_bpf_loader".as_bytes().to_vec(),
        );

        // Call user program
        let program_id = load_program(&bank, &mint_keypair, loader_id, elf);
        let tx = Transaction::new(
            &mint_keypair,
            &[],
            program_id,
            &vec![1u8],
            bank.last_blockhash(),
            0,
        );
        bank.process_transaction(&tx).unwrap();
        assert_eq!(bank.get_signature_status(&tx.signatures[0]), Some(Ok(())));
    }
}