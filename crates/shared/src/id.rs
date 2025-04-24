use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const EPOCH_MILLIS: u64 = 1735689600000; // 2025-01-01 UTC

pub struct IdGenerator {
    last_timestamp: AtomicU64,
    sequence: AtomicU64,
}

impl IdGenerator {
    pub fn new() -> Self {
        Self {
            last_timestamp: AtomicU64::new(0),
            sequence: AtomicU64::new(0),
        }
    }

    pub fn generate(&self, entity_type: u8) -> u64 {
        let uuid = Uuid::new_v4();
        self.generate_from_uuid(uuid, entity_type)
    }

    pub fn generate_from_uuid(&self, uuid: Uuid, entity_type: u8) -> u64 {
        let now_ms = current_unix_ms() - EPOCH_MILLIS;

        let last = self.last_timestamp.load(Ordering::Relaxed);
        let seq = if now_ms == last {
            (self.sequence.fetch_add(1, Ordering::Relaxed) + 1) & 0b111111
        } else {
            self.sequence.store(0, Ordering::Relaxed);
            self.last_timestamp.store(now_ms, Ordering::Relaxed);
            0
        };

        let uuid_bytes = uuid.as_u128().to_le_bytes();
        let uuid_fragment = (u64::from_le_bytes([
            uuid_bytes[0], uuid_bytes[1], uuid_bytes[2], uuid_bytes[3],
            uuid_bytes[4], uuid_bytes[5], uuid_bytes[6], uuid_bytes[7],
        ]) & 0x1FF) as u64;

        (now_ms << (9 + 5 + 6)) | (uuid_fragment << (5 + 6)) | ((entity_type as u64) << 6) | seq
    }
}

fn current_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64
}
