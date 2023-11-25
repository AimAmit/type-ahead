use std::sync::atomic::{AtomicU32, Ordering};

use crate::traits::{WordImpl, WordProcesImpl};

static WORD_COUNTER: AtomicU32 = AtomicU32::new(0);

pub mod pb {
    include!(concat!(env!("OUT_DIR"), "/storage_proto.word.rs"));
}

impl pb::Word {
    fn new(id: u32) -> Self {
        pb::Word {
            id,
            in_records: vec![],
            position: vec![],
            popularity: 0,
        }
    }
}

impl WordImpl for pb::Word {

    

    fn update_pos(&mut self, doc_id: u32, pos: u32) {
        self.in_records
            .push(pb::WordRecordTuple { id: doc_id, pos });
        self.position.push(pos);
        self.popularity += 1;
    }
}

impl WordProcesImpl for pb::WordMap {
    fn get_or_create_word_mut(&mut self, word: &str) -> &mut dyn WordImpl {
        if let None = self.word_hash.get_mut(word) {
            let id = WORD_COUNTER.fetch_add(1, Ordering::SeqCst);
            let mut word_obj = pb::Word::new(id);
            self.word_hash.insert(word.to_string(), word_obj);
        }

        let word = self.word_hash.get_mut(word).unwrap();
        word
    }
}