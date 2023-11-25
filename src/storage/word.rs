use crate::traits::{WordImpl, WordProcesImpl};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::atomic::{AtomicU32, AtomicUsize, Ordering},
};

static WORD_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Serialize, Deserialize)]
pub struct Word {
    id: u32,
    pub in_records: Vec<(u32, u16)>,
    pub postion: Vec<u32>,
    pub popularity: u32,
}
impl Word {
    pub fn new(id: u32) -> Self {
        Word {
            id,
            in_records: vec![],
            postion: vec![],
            popularity: 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WordMap {
    pub word_hash: HashMap<String, Word>,
}

impl WordMap {
    pub fn new() -> Self {
        WordMap {
            word_hash: HashMap::new(),
        }
    }

    pub fn get_word(&self, word: &str) -> &Word {
        let word = self.word_hash.get(word).unwrap();
        word
    }
}

impl WordImpl for Word {
    fn update_pos(&mut self, doc_id: u32, pos: u32) {
        self.in_records.push((doc_id, pos as u16));
        self.postion.push(pos);
        self.popularity += 1;
    }
}

impl WordProcesImpl for WordMap {
    fn get_or_create_word_mut(&mut self, word: &str) -> &mut dyn WordImpl {
        if let None = self.word_hash.get_mut(word) {
            let id = WORD_COUNTER.fetch_add(1, Ordering::SeqCst);
            let mut word_obj = Word::new(id);
            self.word_hash.insert(word.to_string(), word_obj);
        }

        let word = self.word_hash.get_mut(word).unwrap();
        word
    }
}
