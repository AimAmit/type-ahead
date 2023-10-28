use std::collections::{HashMap, BTreeMap, HashSet};

#[derive(Debug)]
pub struct Record {
    pub record: String,
    query_pos: HashMap<String, Vec<usize>>,
    record_pos: Vec<(String, Vec<usize>)>,
    pub positions: HashMap<String, Vec<usize>>,
    pub exact_matches: usize,
    pub operations: usize,
    pub similarity: f64,
}

impl Record {
    pub fn new(query: &str, query_pos: &HashMap<String, Vec<usize>>, record: &str, exact_matches: usize, operations: usize) -> Self {

        let mut record_pos_mp = HashMap::new();
        for (idx, wi) in record.to_lowercase().split_whitespace().enumerate() {
            record_pos_mp.entry(wi.to_string()).or_insert(vec![]).push(idx);
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

        // println!("query_pos: {query_pos:?}\nrecord_pos: {record_pos:?}");

         //record_pos.iter().map(|(k, v)| (k.to_owned(), (v.len(), v.to_owned()))).collect::<HashMap<String, (usize, Vec<usize>)>>();

        
        let mut r = Record {
            record: record.to_string(),
            query_pos: query_pos.clone(),
            record_pos,
            positions: HashMap::new(),
            exact_matches,
            operations,
            similarity: 0.0,
        };

        r.similarity = r._calculate_distance();
        // println!("--- {record}  {}", r.similarity);
        r
    }

    fn generate_combinations(
        &self,
        current: Vec<usize>,
        index: usize,
        result: &mut Vec<Vec<usize>>,
    ) {
        // println!("index -- {index}, data == {:?}", self.record_pos[index]);

        if index == self.record_pos.len() {
            result.push(current);
            return;
        }
    
        let wi = &self.record_pos[index].0;
        let step = self.query_pos.get(wi).unwrap_or(&Vec::new()).len();

        let word_positions = &self.record_pos[index].1;

        // println!("wi: {wi}\tstpe: {step}");

        for (pos, item) in word_positions.windows(step).enumerate() {
                let mut new_current = current.clone();
                new_current.extend(item);
                // println!("idx: {index} {wi}  new_current: {new_current:?}");

                self.generate_combinations(new_current, index + 1, result);
            }
    }

    pub fn generate_all_combinations(&self) -> Vec<Vec<usize>> {
        let mut result = Vec::new();

        // println!("record_pos len: {}", self.record_pos.len());

        // let mut itr: std::collections::hash_map::Iter<'_, String, (usize, Vec<usize)>> = self.query_pos.iter();
        // println!("pos: {:?} \nitr: {:?}", self.query_pos, itr.next());

        self.generate_combinations(Vec::new(), 0, &mut result);
        result
    }

    fn _calculate_distance(&self) -> f64 {
        

        let positons = self.generate_all_combinations();

        // println!("#### positons: {positons:?}");

        let min_distance = positons
            .iter()
            .map(|position| {
                let first = position.first().unwrap_or(&0);
                let tmp = position
                    .iter()
                    .map(|e| e - first)
                    .zip(0..position.len())
                    .map(|(query_pos, record_pos)| (query_pos - record_pos).pow(2))
                    .sum::<usize>();
                // println!("position: {position:?}   tmp --- {tmp}");
                tmp
            })
            .min()
            .unwrap_or(usize::MAX);

        min_distance as f64
    }

    pub fn calculate_distance(&self) -> f64 {
        self.similarity as f64
    }
}
