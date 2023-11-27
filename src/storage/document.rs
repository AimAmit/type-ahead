use std::{
    collections::HashMap,
    sync::atomic::{AtomicU32, Ordering},
    usize,
};

use prost::Message;
use serde::{Deserialize, Serialize};
use unidecode::unidecode;

use super::{record::Record, word::{WordMap, WordInRecord}};

static DOCUMENT_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(Serialize, Deserialize, Message)]
pub struct DocumentMap {
    #[prost(map = "uint32, string", tag = "1")]
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
    ) -> Vec<(String, String)> {
        let mut query_pos = HashMap::new();
        for (idx, wi) in query.split_whitespace().enumerate() {
            query_pos.entry(wi.to_string()).or_insert(vec![]).push(idx);
        }

        let query_pos = query_pos
            .iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect::<HashMap<String, Vec<usize>>>();

        let mut matches = vec![];

        for (record, (nr_matches, edit)) in similar_map.iter() {
            let record = self.document_map.get(&record).unwrap();
            println!("record: {record:?} - {:?}", (nr_matches, edit));

            matches.push(Record::new(query, &query_pos, &record, *nr_matches, *edit));
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
            .map(|ele| (ele.updated_record.clone(), ele.record.to_owned()) )
            .take(10)
            .collect()
    }
}

#[derive(Serialize, Deserialize, Message)]
pub struct Document {
    #[prost(uint32, tag = "1")]
    id: u32,
    #[prost(string, tag = "2")]
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

            word.in_records.push( WordInRecord { idx: self.id, pos: pos as u32} );
            word.position.push(pos as u32);
            word.popularity += 1;

            // trie.insert(e);
        }
    }
}
