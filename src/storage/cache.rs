use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct LruLikeCache<K, V> {
    capacity: usize,
    entries: VecDeque<(K, V)>,
}

impl<K: Eq + Clone, V: Clone> LruLikeCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: VecDeque::new(),
        }
    }

    pub fn get(&mut self, key: &K) -> Option<V> {
        let position = self
            .entries
            .iter()
            .position(|(entry_key, _)| entry_key == key)?;
        let (key, value) = self.entries.remove(position)?;
        self.entries.push_back((key, value.clone()));
        Some(value)
    }

    pub fn insert(&mut self, key: K, value: V) {
        if let Some(position) = self
            .entries
            .iter()
            .position(|(entry_key, _)| entry_key == &key)
        {
            let _ = self.entries.remove(position);
        }
        self.entries.push_back((key, value));
        while self.entries.len() > self.capacity {
            let _ = self.entries.pop_front();
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}
