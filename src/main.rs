mod storage;
use std::collections::{BTreeMap, HashMap};
use std::io::Read;
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

use serde::Deserialize;

fn load_trie_objects() -> (Trie, WordMap, DocumentMap) {
    let trie_fname = "./trie.bin";
    let word_map_fname = "word_map.proto.bin";
    let doc_map_fname = "doc_map.proto.bin";

    let pwd = std::env::current_dir().unwrap();

    let word_map_path = pwd.join(word_map_fname);
    let doc_map_path = pwd.join(doc_map_fname);

    let t1 = Instant::now();
    println!("trie load time: {}", (Instant::now() - t1).as_millis());

    let t1 = Instant::now();
    let mut word_map_file = File::open(word_map_path).unwrap();
    // let word_map: WordMap =
    //     bincode::deserialize_from(word_map_file).expect("Failed to deserialize Trie");

    let mut buf = Vec::new();
    word_map_file
        .read_to_end(&mut buf)
        .expect("Failed to read doc_map");
    let buf = prost::bytes::Bytes::from(buf);
    let word_map: WordMap = Message::decode(buf).expect("Failed to decode doc_map");

    println!("word map load time: {}", (Instant::now() - t1).as_millis());

    let t1 = Instant::now();
    let mut doc_map_file = File::open(doc_map_path).unwrap();
    // let doc_map: DocumentMap =
    //     bincode::deserialize_from(doc_map_file).expect("Failed to deserialize Trie");

    let mut buf = Vec::new();
    doc_map_file
        .read_to_end(&mut buf)
        .expect("Failed to read doc_map");
    let buf = prost::bytes::Bytes::from(buf);
    let doc_map: DocumentMap = Message::decode(buf).expect("Failed to decode doc_map");

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
    let (trie, word_map, doc_map) = load_trie_objects();

    let shared_state = Arc::new(AppState {
        trie,
        word_map,
        doc_map,
    });

    let app = Router::new()
        .route("/", get(|| async { "How u doing" }))
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
