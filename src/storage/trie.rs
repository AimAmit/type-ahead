use edit_distance::edit_distance;
use fst::{automaton::Levenshtein, IntoStreamer, Set};
use priority_queue::PriorityQueue;
use serde::{Deserialize, Serialize};
use std::{
    cmp::{min, Ordering},
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    time::Instant,
};

use crate::storage::cache;

use super::word::WordMap;

#[derive(Default, Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TrieNode {
    is_end_of_word: bool,
    children: BTreeMap<char, TrieNode>,
    popularity: u16,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct QueryWalker<'a> {
    node: &'a TrieNode,
    max_operations: usize,
    operations: usize,
    index: usize,
    word: String,
}

impl<'a> QueryWalker<'a> {
    fn new(node: &'a TrieNode, max_operations: usize) -> Self {
        QueryWalker {
            node,
            max_operations,
            operations: 0,
            index: 0,
            word: String::default(),
        }
    }
}

impl<'a> Ord for QueryWalker<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        let cmp_operations = self.word.cmp(&other.word);
        if cmp_operations != std::cmp::Ordering::Equal {
            return cmp_operations;
        }

        self.index.cmp(&other.index)
    }
}

impl<'a> PartialOrd for QueryWalker<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// #[derive(Default, Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
pub struct Trie {
    root: TrieNode,
    fst: Set<Vec<u8>>,
}

impl Trie {
    pub fn new(mut words: Vec<&String>) -> Self {
        words.sort();

        Trie {
            root: TrieNode::default(),
            fst: Set::from_iter(words.iter()).unwrap(),
        }
    }

    pub fn insert(&mut self, word: &str) {
        let mut current_node = &mut self.root;

        for c in word.chars() {
            current_node = current_node.children.entry(c).or_default();
            current_node.popularity += 1
        }
        current_node.is_end_of_word = true;
    }
}

impl Trie {
    pub fn find_words<'a>(&self, query: &'a str, k: usize) -> Vec<(String, usize)> {
        // let mut pq = PriorityQueue::new();

        let lev = Levenshtein::new(query, k as u32).unwrap();

        // Apply our fuzzy query to the set we built.
        let stream = self.fst.search(lev).into_stream();

        let keys = stream.into_strs().unwrap();

        keys.iter().map(|key| (key.to_owned(), edit_distance(query, key))).collect()

        // let mut pq = BTreeSet::new();

        // let mut similar_words_hm = HashMap::new();
        // let mut similar_words = vec![];

        // let qw = QueryWalker::new(&self.root, k);
        // pq.insert(qw);

        // while !pq.is_empty() {
        //     let mut qw = pq.pop_first().unwrap();

        //     while qw.operations < k {
        //         if qw.operations < k {
        //             let current_char = query.chars().nth(qw.index).unwrap_or('\x00');

        //             for (k, v) in qw.node.children.iter() {
        //                 if *k == current_char || v.popularity < 10 {
        //                     continue;
        //                 }
        //                 // insertion
        //                 let mut word = qw.word.clone();
        //                 word.push(*k);

        //                 let new_qw = QueryWalker {
        //                     node: v,
        //                     max_operations: qw.max_operations,
        //                     operations: qw.operations + 1,
        //                     index: qw.index,
        //                     word: word.clone(),
        //                 };
        //                 pq.insert(new_qw);

        //                 // substituion
        //                 let new_qw = QueryWalker {
        //                     node: v,
        //                     max_operations: qw.max_operations,
        //                     operations: qw.operations + 1,
        //                     index: qw.index + 1,
        //                     word: word.clone(),
        //                 };
        //                 pq.insert(new_qw);
        //             }
        //             // deletion
        //             let new_qw = QueryWalker {
        //                 node: qw.node,
        //                 max_operations: qw.max_operations,
        //                 operations: qw.operations + 1,
        //                 index: qw.index + 1,
        //                 word: qw.word.clone(),
        //             };
        //             pq.insert(new_qw);
        //         }

        //         if let Some(child) = qw.node.children.get(&query.chars().nth(qw.index).unwrap_or('\x00')) {
        //             qw.node = child;
        //             qw.index += 1;
        //             qw.word.push(query.chars().nth(qw.index - 1).unwrap_or('\x00'));
        //         } else {
        //             break;
        //         }
        //     }

        //     if qw.node.is_end_of_word {
        //         if let Some(val) = similar_words_hm.get_mut(&qw.word) {
        //             *val = min(qw.operations, *val);
        //         } else {
        //             similar_words_hm.insert(qw.word.clone(), qw.operations);
        //         }
        //     }

        // }

        // similar_words_hm.iter().for_each(|(word, operations)| similar_words.push((word.clone(), *operations)));

        // println!("similar_words_hm: {similar_words:?}");

        // similar_words
    }

    fn nr_allowed_errors<'a>(&self, w: &'a str, is_last: bool) -> usize {
        if is_last {
            (w.len() as f32).powf(0.8).min(3.0).floor() as usize
        } else if w.len() > 4 {
            2
        } else {
            1
        }
    }

    pub fn find_matches<'a>(
        &self,
        query: &'a str,
        word_map: &WordMap,
    ) -> HashMap<u32, (usize, usize)> {
        let query = query.to_lowercase();

        let mut similar_element_lists: Option<HashMap<u32, (usize, usize)>> = None;

        let mut word_pos_mp = HashSet::new();

        for (i, wi) in query.split_whitespace().enumerate() {
            let k = self.nr_allowed_errors(wi, i == query.split_whitespace().count() - 1);

            let cache_key = format!("{wi}::{k}");

            let data = cache::retrieve_from_cache(&cache_key);

            let word_vec = match data {
                Some(d) => d,
                None => {
                    println!("{cache_key} cache miss");
                    let similar_words = self.find_words(wi, k);

                    cache::insert_into_cache(&cache_key, &similar_words);
                    similar_words
                }
            };

            let mut curr_records: HashMap<u32, (usize, usize)> = HashMap::new();
            let _t1 = Instant::now();

            let mut curr_word_doc = HashSet::new();

            for list in &word_vec {
                let word = word_map.get_word(&list.0);
                let val = ((list.1 == 0) as usize, list.1);
                for rec in &word.in_records {
                    
                    if let None = word_pos_mp.get(rec) {
                        if let Some(_) = curr_word_doc.get(&rec.0) {
                            continue;
                        }
                        curr_word_doc.insert(rec.0.to_owned());
                        curr_records.insert(rec.0.clone(), val);
                        word_pos_mp.insert(rec.clone());
                    }
                }
            }


            let curr_len = &curr_records.len();
            let _t2 = Instant::now();
            println!("find_matches res: {}", (_t2 - _t1).as_millis());

            similar_element_lists = match &similar_element_lists {
                Some(s) => Some({
                    let mut tmp_mp = HashMap::new();
                    for (key, (exact_match, edits)) in s.iter() {
                        if let Some((is_match, edit)) = curr_records.get(key) {
                            tmp_mp.insert(*key, (exact_match + is_match, edits + edit));
                        }
                    }
                    tmp_mp
                }),
                None => Some(curr_records),
            };

            println!(
                "k: {k}, curr_records len = {}, similar_element_lists len = {}",
                curr_len,
                similar_element_lists.as_ref().unwrap().len()
            );
        }        

        match similar_element_lists {
            Some(mp) => mp,
            None => HashMap::new(),
        }
    }
}
