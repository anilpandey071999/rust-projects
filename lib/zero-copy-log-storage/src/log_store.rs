use crate::log_snapshot::LogSnapShot;

/// `LogStore` stores
/// data inside the vec of u8
/// metadata is help for calculation the offset and len for retivel
#[derive(Debug, Default)]
pub struct LogStore {
    data: Vec<u8>,            // data
    metadata: Vec<EntryMeta>, // location
}

impl LogStore {
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(1000000),
            metadata: Vec::with_capacity(1000000),
        }
    }

    pub fn append(&mut self, data: &str) -> usize {
        let new_offset = self.data.len();
        self.data.extend_from_slice(data.as_bytes());

        let new_len = data.len();
        let meta_data = EntryMeta::new(new_offset, new_len);
        self.metadata.push(meta_data);
        self.metadata.len() - 1
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

    pub fn snapshot(&self) -> LogSnapShot<'_> {
        LogSnapShot::new(&self.data, &self.metadata)
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

    pub fn get(&self) -> (usize, usize) {
        (self.offset, self.len)
    }
}
