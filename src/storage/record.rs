use std::collections::HashMap;

#[derive(Debug)]
pub struct Record {
    pub record: String,
    pub positions: Vec<Vec<usize>>,
    pub exact_matches: usize,
    pub operations: usize,
    pub similarity: f64,
}

impl Record {
    pub fn new(query: &str, record: &str, exact_matches: usize, operations: usize) -> Self {
        let mut q_idx = HashMap::new();
        for (idx, wi) in record.split_whitespace().enumerate() {
            q_idx.entry(wi).or_insert(vec![]).push(idx);
        }

        let mut positions = vec![];

        query.split_whitespace().for_each(|wi| {
            if let Some(pos) = q_idx.get(wi) {
                positions.push(pos.to_owned());
            }
        });

        let mut r = Record {
            record: record.to_string(),
            positions,
            exact_matches,
            operations,
            similarity: 0.0,
        };

        r.similarity = r._calculate_distance();
        r
    }

    fn generate_combinations(
        &self,
        positions: &Vec<Vec<usize>>,
        current: Vec<usize>,
        index: usize,
        result: &mut Vec<Vec<usize>>,
    ) {
        if index == positions.len() {
            result.push(current);
            return;
        }

        for &pos in &positions[index] {
            let mut new_current = current.clone();
            new_current.push(pos);
            self.generate_combinations(positions, new_current, index + 1, result);
        }
    }

    fn generate_all_combinations(&self) -> Vec<Vec<usize>> {
        let mut result = Vec::new();
        self.generate_combinations(&self.positions, Vec::new(), 0, &mut result);
        result
    }

    fn _calculate_distance(&self) -> f64 {
        let positons = self.generate_all_combinations();

        let min_distance = positons
            .iter()
            .map(|position| {
                let first = position.first().unwrap_or(&0);
                position
                    .iter()
                    .map(|e| e - first)
                    .zip(position)
                    .map(|(query_pos, record_pos)| (query_pos - record_pos).pow(2))
                    .sum::<usize>()
            })
            .min()
            .unwrap_or(usize::MAX);

        min_distance as f64
    }

    pub fn calculate_distance(&self) -> f64 {
        self.similarity as f64
    }
}
