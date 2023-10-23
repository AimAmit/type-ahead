use std::{sync::atomic::{AtomicUsize, Ordering, AtomicU32}, collections::HashMap};

use serde::{Deserialize, Serialize};


static WORD_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Serialize, Deserialize)]
pub struct Word {
    id: u32,
    pub in_records: Vec<u32>,
    pub postion: Vec<u32>,
    pub popularity: u32,
}
impl Word {
    pub fn new(id: u32) -> Self {
        Word { id, in_records: vec![], postion: vec![], popularity: 0 }
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct WordMap {
    pub word_hash: HashMap<String, Word>
}

impl WordMap {

    pub fn new() -> Self {
        WordMap { word_hash: HashMap::new() }
    }

    pub fn get_or_create_word_mut(&mut self, word: &str) -> &mut Word {
        if let None = self.word_hash.get_mut(word) {
            let id = WORD_COUNTER.fetch_add(1, Ordering::SeqCst);
            let mut word_obj = Word::new(id);
            self.word_hash.insert(word.to_string(), word_obj);
        } 

        let word = self.word_hash.get_mut(word).unwrap();
        word
    }

    pub fn get_word(&self, word: &str) -> &Word {

        let word = self.word_hash.get(word).unwrap();
        word
    }
}