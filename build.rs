use prost_build;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

// #[path = "src/storage/mod.rs"]
// mod storage;

// use storage::{document::DocumentMap, trie::Trie, word::WordMap};

fn main() {
    prost_build::compile_protos(
        &["src/proto/word.proto", "src/proto/document.proto"],
        &["src/"],
    )
    .unwrap();

    // let trie_fname = "trie.bin";
    // let word_map_fname = "word_map.bin";
    // let doc_map_fname = "doc_map.bin";

    // let trie_path = Path::new(trie_fname);
    // let word_map_path = Path::new(word_map_fname);
    // let doc_map_path = Path::new(doc_map_fname);

    // if trie_path.exists() && word_map_path.exists() && doc_map_path.exists() {
    //     return;
    // }

    // let mut word_map = WordMap::new();
    // let mut doc_map = DocumentMap::new();

    // let file = File::open("./movie_title_tmdb.txt").unwrap();

    // // Create a buffered reader for efficient reading
    // let reader = BufReader::new(file);

    // // Iterate over each line and call the process_line function
    // let mut idx = 0;
    // for line_result in reader.lines() {
    //     if let Ok(mut line) = line_result {
    //         // if line.len() > 32 {
    //         //     line = line[..32].to_owned();
    //         // }
    //         let d = doc_map.add_doc(line.to_owned());
    //         d.process(&mut word_map);
    //         // mx_len = max(mx_len, line.len());
    //         println!("{idx}\r");
    //     } else {
    //         eprintln!("Error reading a line.");
    //     }
    //     idx += 1;
    //     if idx > 100_000 {
    //         break;
    //     }
    // }

    // let mut words = vec![];
    // for (w, _) in word_map.word_hash.iter() {
    //     words.push(w);
    // }

    // // let mut trie = Trie::new(words);
    // // println!("mx_len --- {}", mx_len);

    // // trie.write_to_bytes

    // // let file = File::create(trie_path).expect("Failed to create data file");
    // // bincode::serialize_into(file, &trie).expect("Failed to serialize Trie");

    // // serde_json::to_writer_pretty(file, &trie).expect("Failed to serialize trie");

    // let file = File::create(doc_map_path).expect("Failed to create data file");
    // bincode::serialize_into(file, &doc_map).expect("Failed to serialize doc_map");

    // let file = File::create(word_map_path).expect("Failed to create data file");
    // bincode::serialize_into(file, &word_map).expect("Failed to serialize word_map");
}
