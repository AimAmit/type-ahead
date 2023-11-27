use std::{sync::atomic::{Ordering, AtomicU32}, collections::HashMap};
use prost::Message;

use serde::{Deserialize, Serialize};


static WORD_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(Hash, PartialEq, Eq, Serialize, Deserialize, Message)]
pub struct WordInRecord {
    #[prost(uint32, tag = "1")]
    pub idx: u32,    
    #[prost(uint32, tag = "2")]
    pub pos: u32,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Message)]
pub struct Word {
    #[prost(uint32, tag = "1")]
    id: u32,
    #[prost(message, repeated, tag = "2")]
    pub in_records: Vec<WordInRecord>,
    #[prost(uint32, repeated, tag = "3")]
    pub position: Vec<u32>,
    #[prost(uint32, tag = "4")]
    pub popularity: u32,
}
impl Word {
    pub fn new(id: u32) -> Self {
        Word { id, in_records: vec![], position: vec![], popularity: 0 }
    }
}


#[derive(Serialize, Deserialize, Message)]
pub struct WordMap {
    #[prost(map = "string, message", tag = "1")]
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