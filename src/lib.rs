use dotenv::dotenv;
use std::env;
use bitcoincore_rpc::{Auth, Client, json::GetBlockTemplateResult};

use bitcoin::{
    consensus::deserialize,
    Block, OutPoint, ScriptBuf, Transaction, TxIn, TxOut, Sequence,
    blockdata::block::Header as BlockHeader,
    pow::CompactTarget,
    hash_types::TxMerkleNode, block::Version, hashes::Hash
};

use std::{
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        mpsc::{self, Receiver, Sender}, Arc,
    },
    thread,
    time::SystemTime,
};

use bitcoin::absolute::LockTime;


pub fn connect() -> Client {
    dotenv().ok();
    let url = env::var("url").expect("url (ip:port) must be set");
    let rpcuser = env::var("rpcuser").expect("rpcuser must be set");
    let rpcpassword = env::var("rpcpassword").expect("rpcpassword must be set");
    Client::new(&url, Auth::UserPass(rpcuser, rpcpassword)).expect("can`t connect to rpc")
}

pub fn mine(template: &GetBlockTemplateResult, script_pubkey: &ScriptBuf, message: &str) {
    let mut txdata = vec![create_coinbase(template.height, template.coinbase_value.to_sat(), script_pubkey.clone(), message)];
    // txdata.reserve_exact(1 + template.transactions.len());
    txdata.extend(template.transactions.iter().map(|tx| deserialize(&tx.raw_tx).unwrap()));

    /* for tx in &template.transactions {
        txdata.push(deserialize(&tx.raw_tx).unwrap());
    } */

    let header = BlockHeader {
        version: Version::from_consensus(template.version.try_into().unwrap()),
        prev_blockhash: template.previous_block_hash,
        merkle_root: get_merkle_root(&txdata),
        time: template.min_time.try_into().unwrap(),
        bits: CompactTarget::from_consensus(u32::from_be_bytes(template.bits.clone().try_into().unwrap())),
        nonce: 0
    };
    let nonce_max = u32::from_be_bytes(template.nonce_range[4..8].try_into().unwrap());
    let INTERVAL = nonce_max / 1500 | 0;
    // let nonce_max = 1_000_000;

    /* for nonce in 0..nonce_max {
        header.nonce = nonce;
        let hash = header.block_hash();
        let hash_value = hash.as_byte_array();

        let target_bytes = header.target().to_le_bytes();
        println!("{}", nonce);

        if valid_pow_fast(hash_value, &target_bytes) {
            return Some(Block { header, txdata })
        }
    } */

    // threads
    let min = Arc::new(AtomicU32::default());
    let max = Arc::new(AtomicU32::new(INTERVAL));
    let found = Arc::new(AtomicBool::default());
    let start = SystemTime::now();

    let mut handles = vec![];
    let threads = num_cpus::get_physical();

    let (tx, rx): (Sender<(u32, bool)>, Receiver<(u32, bool)>) = mpsc::channel();

    for cpu in 0..threads {
        let min = min.clone();
        let max = max.clone();
        let found = found.clone();
        let header_template = header.clone();
        let tx = tx.clone();

        handles.push(thread::spawn(move || loop {
            let curr_min = min.fetch_add(INTERVAL, Ordering::SeqCst);
            let curr_max = max.fetch_add(INTERVAL, Ordering::SeqCst);
            if curr_max == u32::max_value() {
                break;
            }
            if found.load(Ordering::SeqCst) {
                break;
            }
            // send a status update with the maximum nonce we've tried
            if let Err(_) = tx.send((curr_max, false)) {
                break;
            }
            // stop if we've reached the max u32 value
            for nonce in curr_min..=curr_max {
                // stop if another thread found a valid nonce
                let header = BlockHeader {
                    version: header_template.version,
                    prev_blockhash: header_template.prev_blockhash,
                    merkle_root: header_template.merkle_root,
                    time: header_template.time,
                    bits: header_template.bits,
                    nonce,
                };

                let hash = header.block_hash();
                let hash_value = hash.as_byte_array();

                let target_bytes = header.target().to_le_bytes();
                // println!("{} :: {}", cpu, nonce);

                if valid_pow_fast(hash_value, &target_bytes) {
                    tx.send((nonce, true)).unwrap();
                    found.store(true, Ordering::SeqCst);
                    break;
                }
            }
        }));
    }

    thread::spawn(move || {
        let mut max = 0;
        while !found.load(Ordering::SeqCst) {
            match rx.recv() {
                Ok((nonce, found)) if found => {
                    println!("Found nonce {}", nonce);
                    break;
                }
                Ok((hashes, found)) if !found && hashes > max => {
                    max = hashes;
                    let hashrate = hashes as f32 / start.elapsed().unwrap().as_secs_f32();
                    println!(
                        "Status: hashrate={:.2}MH/s hashes={}",
                        hashrate / 1_000_000.0,
                        hashes
                    );
                }
                Err(_) => break,
                _ => (),
            }
        }
    });

    for handle in handles {
        handle.join().expect("child thread panicked");
    }
    // Block { header, txdata }
}

fn create_coinbase(block_height: u64, value: u64, script_pubkey: ScriptBuf, message: &str) -> Transaction {
    let signature = format!(
        "{}{}",
        hex::encode(block_height.to_string().as_bytes()),
        hex::encode(message)
    );

    let mut script_sig = format!("{}{}", signature.bytes().len(), signature);

    if block_height >= 500 {
        let height = block_height.to_le_bytes();
        // FIXME bytes count
        script_sig = format!(
            "03{}{}",
            hex::encode(height),
            hex::encode(message)
        );
    }

    let input = TxIn {
        previous_output: OutPoint::null(),
        script_sig: ScriptBuf::from_hex(&script_sig).expect("coinbase script sig creation error"),
        sequence: Sequence::max_value(),
        ..Default::default()
    };

    let output = TxOut { value, script_pubkey };

    Transaction {
        version: 1,
        lock_time: LockTime::from_height(block_height.try_into().unwrap()).unwrap(),
        input: vec![input],
        output: vec![output],
    }
}

// Calculate whether the hash is smaller than the target quickly
fn valid_pow_fast(hash: &[u8], target: &[u8]) -> bool {
    for i in (0..hash.len()).rev() {
        if hash[i] < target[i] {
            return true;
        }
        if hash[i] > target[i] {
            return false;
        }
    }
    true
}

fn get_merkle_root(transactions: &Vec<Transaction>) -> TxMerkleNode {
    let hashes = transactions.iter().map(|x| x.txid().to_raw_hash());
    bitcoin::merkle_tree::calculate_root(hashes).map(|h| h.into()).unwrap()
}
