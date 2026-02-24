pub struct LogStore {
    pub id: u64,
    pub data: Vec<u8>
}

pub struct EntryMeta {
    pub offset: usize,
    pub len: usize,
}
