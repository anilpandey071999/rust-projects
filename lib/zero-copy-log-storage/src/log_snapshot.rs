use crate::log_store::EntryMeta;

pub struct LogSnapShot<'a> {
    data: &'a [u8],            // data
    metadata: &'a [EntryMeta], // location
}

impl<'a> LogSnapShot<'a> {
    pub fn new(data: &'a [u8], metadata: &'a [EntryMeta]) -> Self {
        Self { data, metadata }
    }
    pub fn get(&self, idx: usize) -> Option<&'a str> {
        if let Some(data) = self.metadata.get(idx) {
            // println!("{}", data.offset, data.len);
            let (offset, len) = data.get();
            let log_slice = &self.data[offset..offset + len];

            return std::str::from_utf8(log_slice).ok();
        }
        None
    }
}
