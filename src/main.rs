mod storage;
mod storage_proto;
mod traits;

use std::collections::{BTreeMap, HashMap};
use std::io::{BufReader, Write, Read};
use std::{fs::File, path::Path, sync::Arc, time::Instant};

use edit_distance::edit_distance;
use fst::automaton::Levenshtein;
use fst::{IntoStreamer, Set};
use prost::Message;
use storage::record::Record;
use storage::{document::DocumentMap, trie::Trie, word::WordMap};

use axum::extract::Query;
use axum::{extract::State, routing::get, Json, Router};
use tower_http::cors::CorsLayer;
use std::{
    io::{BufRead},
};
use serde::Deserialize;
use storage_proto::{WordMap as WordMapProto, DocumentMap as DocumentMapProto};

use crate::traits::{DocumentImpl, DocumentMapImpl, WordProcesImpl};

fn build_fs_bin() {

    let trie_fname = "trie.bin";
    let word_map_fname = "word_map.proto.bin";
    let doc_map_fname = "doc_map.proto.bin";

    let trie_path = Path::new(trie_fname);
    let word_map_path = Path::new(word_map_fname);
    let doc_map_path = Path::new(doc_map_fname);

    if word_map_path.exists() && doc_map_path.exists() {
        return;
    }

        let mut word_map = WordMapProto::default();
    // let mut word_map = WordMap::new();
    let mut doc_map = DocumentMapProto::default();
    // let mut doc_map = DocumentMap::new();

    let file = File::open("./movie_title_tmdb.txt").unwrap();

    // Create a buffered reader for efficient reading
    let reader = BufReader::new(file);

    // Iterate over each line and call the process_line function
    let mut idx = 0;
    for line_result in reader.lines() {
        if let Ok(mut line) = line_result {
            // if line.len() > 32 {
            //     line = line[..32].to_owned();
            // }
            let d = doc_map.add_doc(line.to_owned());
            d.process(&mut word_map);
            // mx_len = max(mx_len, line.len());
            println!("{idx}\r");
        } else {
            eprintln!("Error reading a line.");
        }
        idx += 1;
        // if idx > 100_000 {
        //     break;
        // }
    }

    let mut words = vec![];
    for (w, _) in word_map.word_hash.iter() {
        words.push(w);
    }

    // let mut trie = Trie::new(words);
    // println!("mx_len --- {}", mx_len);

    // trie.write_to_bytes

    // let file = File::create(trie_path).expect("Failed to create data file");
    // bincode::serialize_into(file, &trie).expect("Failed to serialize Trie");

    // serde_json::to_writer_pretty(file, &trie).expect("Failed to serialize trie");

    let mut file = File::create(doc_map_path).expect("Failed to create data file");
    let buf = doc_map.encode_to_vec();
    file.write_all(&buf).expect("Failed to write doc_map");
    

    // bincode::serialize_into(file, &doc_map).expect("Failed to serialize doc_map");

    let mut file = File::create(word_map_path).expect("Failed to create data file");
    let buf = word_map.encode_to_vec();
    file.write_all(&buf).expect("Failed to write word_map");
    // bincode::serialize_into(file, &word_map).expect("Failed to serialize word_map");

}

fn load_trie_objects() -> (Trie, WordMap, DocumentMap) {

    let trie_fname = "trie.bin";
    let word_map_fname = "word_map.bin";
    let word_map_fname_proto = "word_map.proto.bin";
    let doc_map_fname = "doc_map.bin";
    let doc_map_fname_proto = "doc_map.proto.bin";

    let pwd = std::env::current_dir().unwrap();

    let word_map_path = pwd.join(word_map_fname);
    let doc_map_path = pwd.join(doc_map_fname);

    let word_map_path_proto = pwd.join(word_map_fname_proto);
    let doc_map_path_proto = pwd.join(doc_map_fname_proto);

    let t1 = Instant::now();
    println!("trie load time: {}", (Instant::now() - t1).as_millis());

    let t1 = Instant::now();
    let mut word_map_file_proto = File::open(word_map_path_proto).unwrap();
    let mut word_map_file = File::open(word_map_path).unwrap();
    let mut buf = Vec::new();
    word_map_file_proto.read_to_end(&mut buf).expect("Failed to read doc_map");
    let buf = prost::bytes::Bytes::from(buf);
    let word_map_proto: WordMapProto = WordMapProto::decode(buf).expect("Failed to decode doc_map");

    let word_map: WordMap =
        bincode::deserialize_from(word_map_file).expect("Failed to deserialize Trie");
    println!("word map load time: {}", (Instant::now() - t1).as_millis());

    let t1 = Instant::now();
    let mut doc_map_file_proto = File::open(doc_map_path_proto).unwrap();

    let mut doc_map_file = File::open(doc_map_path).unwrap();

    let mut buf = Vec::new();
    doc_map_file_proto.read_to_end(&mut buf).expect("Failed to read doc_map");
    let buf = prost::bytes::Bytes::from(buf);
    let doc_map_proto: DocumentMapProto = DocumentMapProto::decode(buf).expect("Failed to decode doc_map");

    let doc_map: DocumentMap =
        bincode::deserialize_from(doc_map_file).expect("Failed to deserialize Trie");
    println!("doc map load time: {}", (Instant::now() - t1).as_millis());

    let mut words = vec![];
    for (w, _) in word_map.word_hash.iter() {
        words.push(w);
    }

    let trie = Trie::new(words);

    (trie, word_map, doc_map)
}

struct AppState {
    trie: Trie,
    word_map: WordMap,
    doc_map: DocumentMap,
}

#[tokio::main]
async fn main() {
    build_fs_bin();
    let (trie, word_map, doc_map) = load_trie_objects();

    let shared_state = Arc::new(AppState {
        trie,
        word_map,
        doc_map,
    });

    let app = Router::new()
        .route("/", get(|| async { "Hello" }))
        .route("/search", get(search).with_state(shared_state))
        .layer(CorsLayer::permissive());

    println!("Server starting");

    let port = std::env::var("PORT").unwrap_or("5050".to_string());

    axum::Server::bind(&format!("0.0.0.0:{port}").parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Deserialize, Debug)]
struct SearchQuery {
    query: Option<String>,
}

#[derive(serde::Serialize)]
struct SearchResult {
    query: String,
    results: Vec<String>,
    html_results: Vec<String>,
    time: u128,
}

async fn search(
    Query(query): Query<SearchQuery>,
    State(state): State<Arc<AppState>>,
) -> Json<SearchResult> {
    let query = query.query.unwrap_or_default();

    let query = query.to_lowercase();

    let t1 = Instant::now();

    let trie = &state.trie;
    let word_map = &state.word_map;
    let doc_map = &state.doc_map;

    let similar_doc_ids = trie.find_matches(&query, &word_map);

    println!("similar_doc_ids len {}", similar_doc_ids.len());

    let _t1 = Instant::now();
    let search = doc_map.sort_raw_result(&query, &similar_doc_ids);
    let _t2 = Instant::now();
    println!("sorting res: {}", (_t2 - _t1).as_millis());

    let t2 = Instant::now();

    // println!("{} ms ", (t2 - t1).as_millis());

    // Create a JSON response
    let results = search.iter().map(|r| r.1.to_owned()).collect();
    let html_results = search.iter().map(|r| r.0.to_owned()).collect();
    Json(SearchResult {
        query,
        results,
        html_results,
        time: (t2 - t1).as_millis(),
    })
}

// fn main() {
//     // Sample query and record
//     // let query = "lodr of the";
//     // let records = vec![
//     //     "lodr of the",
//     //     "The Lord of the Rings: The Fellowship of the Ring",
//     //     "Running Out of Time 2",
//     // ];

//     // // Sample records
//     // let mut records: Vec<Record> = records
//     //     .iter()
//     //     .map(|record| {
//     //         Record::new(query, record, 1, 3)
//     //         // Record {
//     //         //     content: record.to_string(),
//     //         //     positions: vec![],
//     //         // },
//     //         // Add more records...
//     //     })
//     //     .collect();

//     // println!("records: {records:?} \n\n");

//     // // Sort records based on positional distance
//     // records.sort_by(|a, b| {
//     //     // if a.exact_matches != b.exact_matches {
//     //     //     return b.exact_matches.cmp(&a.exact_matches);
//     //     // }
//     //     // a.operations.cmp(&b.operations)

//     //     a.calculate_distance().total_cmp(&b.calculate_distance())
//     // });

//     // // Print sorted records
//     // for record in &records {
//     //     println!("{:?} - {}", record, record.calculate_distance());
//     // }

//     // let t1 = Instant::now();

//     // let (trie, word_map, doc_map) = load_trie_objects();

//     // let load_time = Instant::now() - t1;

//     //     let query = "to";

//     //     let similar_doc_ids = trie.find_matches(&query, &word_map);

//     //     let _t1 = Instant::now();
//     //     let search = doc_map.sort_raw_result(&query, &similar_doc_ids);
//     //     let _t2 = Instant::now();
//     //     println!("sorting res: {}", (_t2 - _t1).as_millis());

//     // let t2 = Instant::now();

//     // println!(
//     //     "{:?} \n {} ms ",
//     //     load_time.as_millis(),
//     //     (t2 - t1).as_millis()
//     // );

//     // let mut words = vec![];
//     // for (w, _) in word_map.word_hash.iter() {
//     //     words.push(w);
//     // }

//     // let t1 = Instant::now();
//     // words.sort();
//     // let set = Set::from_iter(words.iter()).unwrap();

//     // let t2 = Instant::now();

//     // println!("fst set build {} ms ", (t2 - t1).as_millis());

//     // let t1 = Instant::now();
//     // // Build our fuzzy query.
//     // let lev = Levenshtein::new("revolution", 3).unwrap();

//     // // Apply our fuzzy query to the set we built.
//     // let stream = set.search(lev).into_stream();

//     // let t2 = Instant::now();

//     // println!("fst search {} ms ", (t2 - t1).as_millis());

//     // let keys = stream.into_strs().unwrap();

//     // let keys: Vec<(&String, usize)> = keys
//     //     .iter()
//     //     .map(|k| (k, edit_distance("revolution", k)))
//     //     .collect();
//     // println!("keys: {:#?}", keys);

//     let query = "the lord of the";
//     let record = "the lord ring of the";

//     let mut query_pos = HashMap::new();
//     for (idx, wi) in query.split_whitespace().enumerate() {
//         query_pos.entry(wi.to_string()).or_insert(vec![]).push(idx);
//     }

//     let query_pos = query_pos
//         .iter()
//         .map(|(k, v)| (k.to_owned(), v.to_owned()))
//         .collect::<HashMap<String, Vec<usize>>>();

//     let r = Record::new(query, &query_pos, record, 0, 0);

//     println!("record: {:?}", r);

//     println!("comb: {:?}", r.generate_all_combinations());

//     println!("res: {:?}", r.updated_record);
// }
