use std::{collections::HashMap, ffi::OsStr};

use anyhow::Ok;
use once_cell::sync::Lazy;
use regex::Regex;
use tokio::io::AsyncReadExt;
use tokio::task::JoinSet;
use walkdir::{DirEntry, WalkDir};

use crate::config::Config;

static LINKS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[\[.*?\]\]").unwrap());
static TAGS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"#\w+").unwrap());

type TagMap = HashMap<String, u32>;

#[derive(Debug, Clone)]
struct NoteStats {
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
    pub fn frequent_tags(&self, take: usize) -> Vec<(&String, &u32)> {
        let mut tags_vec: Vec<_> = self.tags.iter().collect();
        tags_vec.sort_by(|a, b| b.1.cmp(a.1));
        tags_vec.into_iter().take(take).collect()
    }

    pub async fn walk_vault(&mut self, cfg: &Config) -> anyhow::Result<()> {
        let mut set: JoinSet<NoteStats> = JoinSet::new();

        for entry in WalkDir::new(cfg.vault.clone())
            .into_iter()
            .filter_entry(|e| !is_hidden(e))
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file() && e.path().extension() == Some(OsStr::new("md")))
        {
            set.spawn(async move {
                let mut contents = String::new();
                let mut file = tokio::fs::OpenOptions::new()
                    .read(true)
                    .open(entry.path())
                    .await
                    .unwrap();
                file.read_to_string(&mut contents).await.unwrap();

                let word_count = contents.split_ascii_whitespace().count();
                let link_count = LINKS_REGEX.find_iter(&contents).count();
                let mut tags: TagMap = TagMap::new();

                for t in TAGS_REGEX.find_iter(&contents) {
                    let tag = t.as_str().to_string();
                    *tags.entry(tag).or_insert(0) += 1;
                }

                NoteStats {
                    link_count,
                    word_count,
                    tags,
                }
            });
        }

        while let Some(res) = set.join_next().await {
            self.merge(res?);
        }

        Ok(())
    }

    fn merge(&mut self, note: NoteStats) {
        self.total_link_count += note.link_count;
        self.total_word_count += note.word_count;
        for (tag, count) in note.tags {
            *self.tags.entry(tag).or_insert(0) += count;
        }
    }
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use crate::vault::VaultStats;

    #[test]
    fn frequent_tags() {
        let mut vs = VaultStats {
            ..Default::default()
        };
        vs.tags.insert("#second".to_string(), 3);
        vs.tags.insert("#first".to_string(), 4);
        vs.tags.insert("#third".to_string(), 2);
        vs.tags.insert("#fourth".to_string(), 1);

        let take = 3;
        let freq_tags = vs.frequent_tags(take);

        assert_eq!(freq_tags.len(), take);
        assert_eq!(freq_tags[0], (&("#first".to_string()), &4));
        assert_eq!(freq_tags[1], (&("#second".to_string()), &3));
        assert_eq!(freq_tags[2], (&("#third".to_string()), &2));
    }
}
