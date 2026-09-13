// helpers/suid.rs
#![allow(dead_code)]
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use rand::{thread_rng, RngCore};

const EPOCH: u64 = 1700000000000;

static LAST_TIMESTAMP: AtomicU64 = AtomicU64::new(0);
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

const WORKER_ID_BITS: u64 = 5;
const DATACENTER_ID_BITS: u64 = 5;
const SEQUENCE_BITS: u64 = 12;

const MAX_SEQUENCE: u64 = (1 << SEQUENCE_BITS) - 1;
const TIMESTAMP_SHIFT: u64 = WORKER_ID_BITS + DATACENTER_ID_BITS + SEQUENCE_BITS;

fn current_milliseconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Thời gian bị ngược!")
        .as_millis() as u64
}

fn wait_for_next_ms(last_timestamp: u64) -> u64 {
    let mut timestamp = current_milliseconds();
    while timestamp <= last_timestamp {
        timestamp = current_milliseconds();
    }
    timestamp
}

pub fn generate_suid_u64(worker_id: u64, datacenter_id: u64) -> u64 {
    let mut timestamp = current_milliseconds();
    
    let last_timestamp = LAST_TIMESTAMP.load(Ordering::Relaxed);
    let mut sequence = SEQUENCE.load(Ordering::Relaxed);

    if timestamp < last_timestamp {
        panic!("Clock moved backwards. Refusing to generate id for {} milliseconds", last_timestamp - timestamp);
    }

    if timestamp == last_timestamp {
        sequence = (sequence + 1) & MAX_SEQUENCE; 

        if sequence == 0 {
            timestamp = wait_for_next_ms(last_timestamp);
        }
    } else {
        sequence = 0;
    }

    LAST_TIMESTAMP.store(timestamp, Ordering::Relaxed);
    SEQUENCE.store(sequence, Ordering::Relaxed);

    (timestamp - EPOCH) << TIMESTAMP_SHIFT |
    (datacenter_id << (WORKER_ID_BITS + SEQUENCE_BITS)) |
    (worker_id << SEQUENCE_BITS) |
    sequence
}

const BASE62_ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

pub fn to_base62_ordered(mut num: u64) -> String {
    if num == 0 {
        return "000000000000".to_string();
    }
    
    let mut bytes = Vec::with_capacity(12);
    while num > 0 {
        let rem = (num % 62) as usize;
        bytes.push(BASE62_ALPHABET[rem]);
        num /= 62;
    }
    
    while bytes.len() < 12 {
        bytes.push(b'0');
    }
    
    bytes.reverse();
    
    String::from_utf8(bytes).expect("Lỗi chuyển đổi Base62")
}

pub fn suid() -> String {
    let id_u64 = generate_suid_u64(1, 1);
    to_base62_ordered(id_u64)
}

pub fn generate_random_hex() -> String {
    let mut bytes = [0u8; 16];
    thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}
