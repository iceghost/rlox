use std::collections::HashMap;
use std::hash::BuildHasher;
use std::hash::Hasher;

use crate::value::ObjString;

pub type Table<V> = HashMap<ObjString, V, FNV1aBuilder>;

#[derive(Default)]
pub struct FNV1aBuilder;

impl BuildHasher for FNV1aBuilder {
    type Hasher = FNV1aHasher;

    fn build_hasher(&self) -> Self::Hasher {
        FNV1aHasher::default()
    }
}

pub struct FNV1aHasher {
    hash: u32,
}

impl Default for FNV1aHasher {
    fn default() -> Self {
        let hash = 2166136261;
        Self { hash }
    }
}

impl Hasher for FNV1aHasher {
    fn finish(&self) -> u64 {
        self.hash as u64
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.hash ^= byte as u32;
            self.hash = self.hash.wrapping_mul(16777619);
        }
    }
}
