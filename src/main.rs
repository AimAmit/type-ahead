mod storage;
use std::{
    fs::File,
    path::Path,
    sync::Arc,
    time::Instant,
};

use storage::{document::DocumentMap, trie::Trie, word::WordMap};

use axum::extract::Query;
use axum::{
    extract::State, routing::get, Json, Router,
};
use tower_http::cors::CorsLayer;

use serde::Deserialize;


fn load_trie_objects() -> (Trie, WordMap, DocumentMap) {
    let trie_fname = "./trie.bin";
    let word_map_fname = "word_map.bin";
    let doc_map_fname = "doc_map.bin";

    let word_map_path = Path::new(word_map_fname);
    let doc_map_path = Path::new(doc_map_fname);

    let t1 = Instant::now();
    println!("trie load time: {}", (Instant::now() - t1).as_millis());

    let t1 = Instant::now();
    let word_map_file = File::open(word_map_path).unwrap();
    let word_map: WordMap =
        bincode::deserialize_from(word_map_file).expect("Failed to deserialize Trie");
    println!("word map load time: {}", (Instant::now() - t1).as_millis());

    let t1 = Instant::now();
    let doc_map_file = File::open(doc_map_path).unwrap();
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

    axum::Server::bind(&"0.0.0.0:5050".parse().unwrap())
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
    time: u128,
}

async fn search(
    Query(query): Query<SearchQuery>,
    State(state): State<Arc<AppState>>,
) -> Json<SearchResult> {
    let query = query.query.unwrap_or_default();

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
    Json(SearchResult {
        query,
        results: search,
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

//     let t1 = Instant::now();

//     let (trie, word_map, doc_map) = load_trie_objects();

//     let load_time = Instant::now() - t1;

//     let query = "to";

//     let similar_doc_ids = trie.find_matches(&query, &word_map);

//     let _t1 = Instant::now();
//     let search = doc_map.sort_raw_result(&query, &similar_doc_ids);
//     let _t2 = Instant::now();
//     println!("sorting res: {}", (_t2 - _t1).as_millis());

// let t2 = Instant::now();

// println!(
//     "{:?} \n {} ms ",
//     load_time.as_millis(),
//     (t2 - t1).as_millis()
// );

// let mut words = vec![];
// for (w, _) in word_map.word_hash.iter() {
//     words.push(w);
// }

// let t1 = Instant::now();
// words.sort();
// let set = Set::from_iter(words.iter()).unwrap();

// let t2 = Instant::now();

// println!("fst set build {} ms ", (t2 - t1).as_millis());

// let t1 = Instant::now();
// // Build our fuzzy query.
// let lev = Levenshtein::new("revolution", 3).unwrap();

// // Apply our fuzzy query to the set we built.
// let stream = set.search(lev).into_stream();

// let t2 = Instant::now();

// println!("fst search {} ms ", (t2 - t1).as_millis());

// let keys = stream.into_strs().unwrap();
// println!("keys: {:?}", keys);
// }
