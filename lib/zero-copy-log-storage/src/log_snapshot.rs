use crate::log_store::LogStore;

pub struct LogSnapShot<'a> {
    store: &'a LogStore,
    visible_entries: usize,
}

impl<'a> LogSnapShot<'a> {
    pub fn new(store: &'a LogStore, visible_entries: usize) -> Self {
        Self {
            store,
            visible_entries,
        }
    }
    pub fn get(&self, idx: usize) -> Option<&'a str> {
        if idx >= self.visible_entries {
            return None;
        }

        self.get(idx)
    }
}
