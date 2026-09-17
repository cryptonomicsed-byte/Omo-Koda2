use std::collections::VecDeque;

const MAX_ENTRIES: usize = 100;
const INLINE_THRESHOLD: usize = 1024;

#[derive(Clone, Debug)]
pub struct HistoryEntry {
    pub role: String,
    pub content: String,
    pub hash: u64,
    pub pasted: bool,
}

pub struct MessageHistory {
    pub messages: VecDeque<HistoryEntry>,
    pub max: usize,
    paste_counter: usize,
}

fn fnv1a(s: &str) -> u64 {
    let mut h: u64 = 14695981039346656037;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    h
}

impl MessageHistory {
    pub fn new() -> Self {
        Self::with_capacity(MAX_ENTRIES)
    }

    pub fn with_capacity(max: usize) -> Self {
        Self {
            messages: VecDeque::new(),
            max,
            paste_counter: 0,
        }
    }

    /// Push a message, skipping if identical hash to last entry.
    pub fn push(&mut self, role: impl Into<String>, content: impl Into<String>) {
        let role = role.into();
        let content = content.into();
        let hash = fnv1a(&content);

        if let Some(last) = self.messages.back() {
            if last.hash == hash {
                return;
            }
        }

        let pasted = content.len() >= INLINE_THRESHOLD;
        self.messages.push_back(HistoryEntry {
            role,
            content,
            hash,
            pasted,
        });

        while self.messages.len() > self.max {
            self.messages.pop_front();
        }
    }

    /// Return inline content if < 1KB; otherwise a short reference label.
    pub fn inline_or_ref(&mut self, content: &str) -> String {
        if content.len() < INLINE_THRESHOLD {
            return content.to_string();
        }
        self.paste_counter += 1;
        let lines = content.lines().count();
        format!("[Pasted text #{} +{} lines]", self.paste_counter, lines)
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

impl Default for MessageHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedup_skips_identical_consecutive() {
        let mut h = MessageHistory::new();
        h.push("user", "hello");
        h.push("user", "hello");
        assert_eq!(h.len(), 1);
    }

    #[test]
    fn different_messages_kept() {
        let mut h = MessageHistory::new();
        h.push("user", "a");
        h.push("user", "b");
        assert_eq!(h.len(), 2);
    }

    #[test]
    fn trims_to_max() {
        let mut h = MessageHistory::with_capacity(3);
        for i in 0..5u32 {
            h.push("user", format!("msg {i}"));
        }
        assert_eq!(h.len(), 3);
        assert_eq!(h.messages.front().unwrap().content, "msg 2");
    }

    #[test]
    fn inline_short_content() {
        let mut h = MessageHistory::new();
        let s = "short";
        assert_eq!(h.inline_or_ref(s), s);
    }

    #[test]
    fn ref_for_long_content() {
        let mut h = MessageHistory::new();
        let big = "x".repeat(2000);
        let r = h.inline_or_ref(&big);
        assert!(r.starts_with("[Pasted text #1"));
    }
}
