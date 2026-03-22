use crate::log_snapshot::LogSnapShot;
use std::{cell::UnsafeCell, fs::Metadata, mem::MaybeUninit, sync::atomic::AtomicUsize};

const MAX_ENTRIES: usize = 1_000_000;
const MAX_DATA_BYTES: usize = 10_000_000;
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

unsafe impl Sync for LogStore {}

impl LogStore {
    pub fn new() -> Self {
        Self {
            data: UnsafeCell::new(vec![0u8; MAX_DATA_BYTES]),
            metadata: (0..MAX_ENTRIES)
                .map(|_| UnsafeCell::new(EntryMeta::default()))
                .collect(),
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
            if offset + bytes.len() > data_buf.len() {
                panic!("log buffer full");
            }
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
        let index = self.entry_count.load(std::sync::atomic::Ordering::Acquire);
        if log_id < index {
            let metadata = unsafe { &*self.metadata[log_id].get() };
            let offset = metadata.offset;
            let len = metadata.len;
            let data = unsafe { &*self.data.get() };
            let log_slice = &data[offset..offset + len];

            return std::str::from_utf8(log_slice).ok();
        }

        None
    }

    pub fn snapshot(&self) -> LogSnapShot<'_> {
        let visible = self.entry_count.load(std::sync::atomic::Ordering::Acquire);
        LogSnapShot::new(self, visible)
    }
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
}
