use std::collections::HashMap;

pub type TagMap = HashMap<String, u32>;

#[derive(Debug, Clone)]
pub struct Note {
    pub word_count: usize,
    pub link_count: usize,
    pub tags: TagMap,
}

#[derive(Debug, Clone, Default)]
pub struct VaultStats {
    pub total_word_count: usize,
    pub total_link_count: usize,
    pub tags: TagMap,
}

impl VaultStats {
    pub fn merge(&mut self, note: Note) {
        self.total_link_count += note.link_count;
        self.total_word_count += note.word_count;
        for (tag, count) in note.tags {
            *self.tags.entry(tag).or_insert(0) += count;
        }
    }
}
