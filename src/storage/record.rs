use std::{
    borrow::BorrowMut,
    collections::{BTreeMap, HashMap, HashSet},
};

use difference::{Changeset, Difference};

#[derive(Debug)]
pub struct Record {
    pub record: String,
    query: String,
    query_pos: HashMap<String, Vec<usize>>,
    record_pos: Vec<(String, Vec<usize>)>,
    pub positions: HashMap<String, Vec<usize>>,
    pub exact_matches: usize,
    pub operations: usize,
    pub similarity: f32,
    pub updated_record: String,
}

impl Record {
    pub fn new(
        query: &str,
        query_pos: &HashMap<String, Vec<usize>>,
        record: &str,
        exact_matches: usize,
        operations: usize,
    ) -> Self {
        let mut record_pos_mp = HashMap::new();
        for (idx, wi) in record.to_lowercase().split_whitespace().enumerate() {
            record_pos_mp
                .entry(wi.to_string())
                .or_insert(vec![])
                .push(idx);
        }

        let mut record_pos = Vec::new();
        let mut tmp_record_pos = HashSet::new();
        for wi in query.to_lowercase().split_whitespace() {
            if let Some(_) = tmp_record_pos.get(wi) {
                continue;
            }
            tmp_record_pos.insert(wi);
            if let Some(v) = record_pos_mp.get(wi) {
                record_pos.push((wi.to_owned(), v.to_owned()));
            }
        }

        println!("query_pos: {query_pos:?}\nrecord_pos: {record_pos:?}");

        //record_pos.iter().map(|(k, v)| (k.to_owned(), (v.len(), v.to_owned()))).collect::<HashMap<String, (usize, Vec<usize>)>>();

        let mut r = Record {
            record: record.to_string(),
            query: query.to_owned(),
            query_pos: query_pos.clone(),
            record_pos,
            positions: HashMap::new(),
            exact_matches,
            operations,
            similarity: 0.0,
            updated_record: record.to_owned(),
        };

        r.similarity = r._calculate_distance();
        // println!("--- {record}  {}", r.similarity);
        r
    }

    fn generate_combinations(
        &self,
        current: &mut Vec<usize>,
        index: usize,
        result: &mut Vec<Vec<usize>>,
    ) {
        // println!("index -- {index}, data == {:?}", self.record_pos[index]);

        if index == self.record_pos.len() {
            current.sort();
            result.push(current.to_vec());
            return;
        }

        let wi = &self.record_pos[index].0;
        let step = self.query_pos.get(wi).unwrap_or(&Vec::new()).len();

        let word_positions = &self.record_pos[index].1;

        let step = std::cmp::min(step, word_positions.len());

        // println!("wi: {wi}\tstpe: {step}");

        for item in word_positions.windows(step) {
            let mut new_current = current.clone();
            new_current.extend(item);
            println!("idx: {index} {wi}  new_current: {new_current:?}");

            self.generate_combinations(&mut new_current, index + 1, result);
        }
    }

    pub fn generate_all_combinations(&self) -> Vec<Vec<usize>> {
        let mut result = Vec::new();

        // println!("record_pos len: {}", self.record_pos.len());

        // let mut itr: std::collections::hash_map::Iter<'_, String, (usize, Vec<usize)>> = self.query_pos.iter();
        // println!("pos: {:?} \nitr: {:?}", self.query_pos, itr.next());

        self.generate_combinations(&mut Vec::new(), 0, &mut result);
        result
    }

    fn _calculate_distance(&mut self) -> f32 {
        let positons = self.generate_all_combinations();

        // println!("#### positons: {positons:?}");

        // let min_distance = positons
        //     .iter()
        //     .map(|position| {
        //         let first = position.first().unwrap_or(&0);
        //         let tmp = position
        //             .iter()
        //             // .map(|e| e - first)
        //             .zip(0..position.len())
        //             .map(|(query_pos, record_pos)| (query_pos - record_pos).pow(2))
        //             .sum::<usize>();
        //         // println!("position: {position:?}   tmp --- {tmp}");
        //         tmp
        //     })
        //     .min()
        //     .unwrap_or(usize::MAX);

        let mut mn_dist = usize::MAX;

        for position in positons {
            let first = position.first().unwrap_or(&0);
            let dist = position
                .iter()
                // .map(|e| e - first)
                .zip(0..position.len())
                .map(|(query_pos, record_pos)| (query_pos - record_pos).pow(2))
                .sum::<usize>();

            if dist < mn_dist {
                mn_dist = dist;
                self.updated_record = self.highlight_changes(&position);
            }
        }

        mn_dist as f32
    }

    pub fn calculate_distance(&self) -> f32 {
        self.similarity as f32
    }

    fn highlight_changes(&self, position: &Vec<usize>) -> String {
        let updated_record = self
            .record
            .to_owned()
            .split_ascii_whitespace()
            .collect::<Vec<&str>>();

        let mut idx = 0;

        let query_vec = self.query.split_ascii_whitespace().collect::<Vec<&str>>();

        let mut record_vec = self.record.split_ascii_whitespace().map(|s| s.to_owned()).collect::<Vec<String>>();

        for (idx, &pos) in position.iter().enumerate() {
            if idx >= query_vec.len() {
                break;
            }

            let wi = self.highlight_word(query_vec[idx], &record_vec[pos]);
            record_vec[pos] = wi;   
        }

        record_vec.join(" ")
    }

    fn highlight_word(&self, original: &str, corrected: &str) -> String {
        let original_lower = original.to_lowercase();
        let corrected_lower = corrected.to_lowercase();
    
        let changeset = Changeset::new(&original_lower, &corrected_lower, "");
    
        let mut highlighted = String::new();
        let mut orig_iter = original.chars();
    
        for diff in changeset.diffs {
            match diff {
                Difference::Same(s) => highlighted.push_str(&format!("<span style='font-weight:bold;'>{}</span>", s)),
                Difference::Add(s) => {
                    highlighted.push_str(&format!("<span style='font-weight:lighter;'>{}</span>", s));
                }
                Difference::Rem(s) => {
                    if let Some(orig_char) = orig_iter.next() {
                        highlighted.push_str(&format!("<span style='font-weight:bold;'>{}</span>", orig_char));
                    }
                }
            }
        }
    
        highlighted
    }
    

    // fn highlight_word(&self, original: &str, suggested: &str) -> String {
    //     let original_chars: Vec<_> = original.chars().collect();
    //     let suggested_chars: Vec<_> = suggested.chars().collect();
    
    //     let mut highlighted = String::new();
    
    //     for (orig_char, sug_char) in original_chars.iter().zip(suggested_chars.iter()) {
    //         if orig_char.to_lowercase().next() != sug_char.to_lowercase().next() {
    //             // Highlight the differing characters
    //             highlighted.push_str(&format!("<span class='highlight'>{}</span>", sug_char));
    //         } else {
    //             highlighted.push(*orig_char);
    //         }
    //     }
    
    //     // Append any remaining characters from the suggested word
    //     // highlighted.push_str(&suggested_chars[original_chars.len()..].iter().collect::<String>());
    
    //     highlighted
    // }
}
