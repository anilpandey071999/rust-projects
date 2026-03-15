use crate::log_snapshot::LogSnapShot;
use std::{cell::UnsafeCell, mem::MaybeUninit, sync::atomic::AtomicUsize};

const MAX_ENTRIES: usize = 1000000;
/// `LogStore` stores
/// data inside the vec of u8
/// metadata is help for calculation the offset and len for retivel
#[derive(Debug, Default)]
pub struct LogStore {
    data: UnsafeCell<Vec<u8>>,            // data
    metadata: Vec<UnsafeCell<EntryMeta>>, // location
    entry_count: AtomicUsize,
    data_pos: AtomicUsize,
}

impl LogStore {
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(MAX_ENTRIES),
            metadata: Vec::with_capacity(MAX_ENTRIES),
            entry_count: AtomicUsize::new(0),
            data_pos: AtomicUsize::new(0),
        }
    }

    pub fn append(&self, data: &str) -> usize {
        let index = self.entry_count.load(std::sync::atomic::Ordering::Relaxed);
        let offset = self
            .data_pos
            .fetch_add(data.len(), std::sync::atomic::Ordering::Relaxed);
        let bytes = data.as_bytes();

        unsafe {
            let data_buf = &mut *self.data.get();
            data_buf[offset..offset + bytes.len()].copy_from_slice(bytes);
        }
        unsafe {
            *self.metadata[index].get() = EntryMeta::new(offset, bytes.len());
        }

        self.entry_count
            .store(index + 1, std::sync::atomic::Ordering::Release);

        index
    }

    pub fn get(&self, log_id: usize) -> Option<&str> {
        if let Some(data) = self.metadata.get(log_id) {
            let offset = data.offset;
            let len = data.len;
            let log_slice = &self.data[offset..offset + len];

            return std::str::from_utf8(log_slice).ok();
        }
        None
    }

    // pub fn snapshot(&self) -> LogSnapShot<'_> {
    //     LogSnapShot::new(&self.data, &self.metadata)
    // }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct EntryMeta {
    offset: usize,
    len: usize,
}

impl EntryMeta {
    pub fn new(offset: usize, len: usize) -> Self {
        Self { offset, len }
    }

    pub fn get(&self) -> (usize, usize) {
        (self.offset, self.len)
    }
}
