use std::sync::atomic::{AtomicU32, Ordering};

use unidecode::unidecode;

use crate::traits::{DocumentMapImpl, DocumentImpl, WordProcesImpl};

static DOCUMENT_COUNTER: AtomicU32 = AtomicU32::new(0);

pub mod pb {
    include!(concat!(env!("OUT_DIR"), "/storage_proto.document.rs"));
}

impl DocumentMapImpl for pb::DocumentMap {
    fn add_doc(&mut self, text: String) -> impl DocumentImpl {
        let id = DOCUMENT_COUNTER.fetch_add(1, Ordering::SeqCst);
        self.document_map.insert(id, text.clone());
        pb::Document { id, text }
    }
}

impl DocumentImpl for pb::Document {
    fn process<T: WordProcesImpl>(&self, word_map: &mut T) {
        let text = unidecode(&self.text);

        let text = text.to_lowercase();

        let mut result_text = String::new();

        for c in text.chars() {
            match c {
                '\'' | ':' | '.' | ',' | '*' | '+' | '?' | '$' | '{' | '}' | '(' | ')' | '|' => {}
                // '-' => {
                //     result_text.push(' '); // Replace '-' with a whitespace
                // }
                _ => {
                    result_text.push(c); // Keep all other characters
                }
            }
        }

        for (pos, e) in result_text.split_whitespace().enumerate() {
            let mut word = word_map.get_or_create_word_mut(e); // as &mut Word;

            word.update_pos(self.id, pos as u32);
            // word.in_records.push((self.id, pos as u16));
            // word.postion.push(pos as u32);
            // word.popularity += 1;

            // trie.insert(e);
        }
    }
}