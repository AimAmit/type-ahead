use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering, AtomicU32},
    usize,
};

use serde::{Deserialize, Serialize};
use unidecode::unidecode;

use super::{record::Record, trie::Trie, word::WordMap};

static DOCUMENT_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(Serialize, Deserialize)]
pub struct DocumentMap {
    document_map: HashMap<u32, String>,
}

impl DocumentMap {
    pub fn new() -> Self {
        DocumentMap {
            document_map: HashMap::new(),
        }
    }

    pub fn add_doc(&mut self, text: String) -> Document {
        let id = DOCUMENT_COUNTER.fetch_add(1, Ordering::SeqCst);
        self.document_map.insert(id, text.clone());
        Document { id, text }
    }

    pub fn get_document(&self, doc_id: &Vec<u32>) -> Vec<String> {
        doc_id
            .iter()
            .map(|id| self.document_map.get(id).unwrap().clone())
            .collect()
    }

    pub fn sort_raw_result<'a>(
        &self,
        query: &'a str,
        similar_map: &HashMap<u32, (usize, usize)>,
    ) -> Vec<String> {
        let mut query_idx = HashMap::new();
        for (i, wi) in query.split_whitespace().enumerate() {
            query_idx.insert(wi, i);
        }

        let mut matches = vec![];

        for (record, (nr_matches, edit)) in similar_map.iter() {
            let record = self.document_map.get(&record).unwrap();
            matches.push(Record::new(query, &record, *nr_matches, *edit));
        }

        matches.sort_by(|a, b| {
            if a.exact_matches != b.exact_matches {
                return b.exact_matches.cmp(&a.exact_matches);
            }
            // a.operations.cmp(&b.operations)

            a.calculate_distance().total_cmp(&b.calculate_distance())
        });

        matches
            .iter()
            .map(|ele| ele.record.clone())
            .take(10)
            .collect()
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Document {
    id: u32,
    text: String,
    // word_index: Vec<usize>,
}

impl Document {
    pub fn process(&self, word_map: &mut WordMap) {
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
            let mut word = word_map.get_or_create_word_mut(e);

            word.in_records.push(self.id);
            word.postion.push(pos as u32);
            word.popularity += 1;

            // trie.insert(e);
        }
    }
}
