extern crate rustc_serialize;
#[macro_use]
extern crate rusted_cypher;

use std::collections::BTreeMap;
use rustc_serialize::json::{Json, ToJson};
use rusted_cypher::{GraphClient, Statement};

const URI: &'static str = "http://neo4j:neo4j@127.0.0.1:7474/db/data";

#[derive(Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
struct Language {
    name: String,
    level: String,
    safe: bool,
}

impl ToJson for Language {
    fn to_json(&self) -> Json {
        let mut map: BTreeMap<String, Json> = BTreeMap::new();
        map.insert("name".to_owned(), Json::String(self.name.clone()));
        map.insert("level".to_owned(), Json::String(self.level.clone()));
        map.insert("safe".to_owned(), Json::Boolean(self.safe));

        Json::Object(map)
    }
}

#[test]
fn save_retrieve_struct() {
    let rust = Language {
        name: "Rust".to_owned(),
        level: "low".to_owned(),
        safe: true,
    };

    let graph = GraphClient::connect(URI).unwrap();

    let statement = Statement::new("CREATE (n:STRUCT_INTG_TEST_1 {lang}) RETURN n")
        .with_param("lang", rust.clone());

    let results: Vec<(Language,)> = graph.cypher().exec(statement).unwrap();
    assert_eq!(1, results.len());

    let ref result = results[0];
    assert_eq!(rust, result.0);

    graph.cypher().exec::<()>("MATCH (n:STRUCT_INTG_TEST_1) DELETE n".into()).unwrap();
}

#[test]
fn transaction_create_on_begin_commit() {
    let rust = Language {
        name: "Rust".to_owned(),
        level: "low".to_owned(),
        safe: true,
    };

    let graph = GraphClient::connect(URI).unwrap();

    let statement = Statement::new(
        "CREATE (n:STRUCT_INTG_TEST_2 {lang})")
        .with_param("lang", rust.clone());

    graph.cypher().transaction()
        .begin::<()>(Some(statement))
        .unwrap()
        .0.commit::<()>(None)
        .unwrap();

    let results: Vec<(Language,)> = graph.cypher()
        .exec("MATCH (n:STRUCT_INTG_TEST_2) RETURN n".into())
        .unwrap();

    assert_eq!(1, results.len());

    let ref result = results[0];
    assert_eq!(rust, result.0);

    graph.cypher().exec::<()>("MATCH (n:STRUCT_INTG_TEST_2) DELETE n".into()).unwrap();
}

#[test]
fn transaction_create_after_begin_commit() {
    let rust = Language {
        name: "Rust".to_owned(),
        level: "low".to_owned(),
        safe: true,
    };

    let graph = GraphClient::connect(URI).unwrap();
    let (mut transaction, _) = graph.cypher().transaction()
        .begin::<()>(None)
        .unwrap();

    let statement = Statement::new(
        "CREATE (n:STRUCT_INTG_TEST_3 {lang})")
        .with_param("lang", rust.clone());

    transaction.exec::<()>(statement).unwrap();
    transaction.commit::<()>(None).unwrap();

    let results: Vec<(Language,)> = graph.cypher()
        .exec("MATCH (n:STRUCT_INTG_TEST_3) RETURN n".into())
        .unwrap();

    assert_eq!(1, results.len());

    let ref result = results[0];
    assert_eq!(rust, result.0);

    graph.cypher().exec::<()>("MATCH (n:STRUCT_INTG_TEST_3) DELETE n".into()).unwrap();
}

#[test]
fn transaction_create_on_commit() {
    let rust = Language {
        name: "Rust".to_owned(),
        level: "low".to_owned(),
        safe: true,
    };

    let graph = GraphClient::connect(URI).unwrap();

    let statement = Statement::new(
        "CREATE (n:STRUCT_INTG_TEST_4 {lang})")
        .with_param("lang", rust.clone());

    let (transaction, _) = graph.cypher().transaction()
        .begin::<()>(None)
        .unwrap();

    transaction.commit::<()>(Some(statement)).unwrap();

    let results: Vec<(Language,)> = graph.cypher()
        .exec("MATCH (n:STRUCT_INTG_TEST_4) RETURN n".into())
        .unwrap();

    assert_eq!(1, results.len());

    let ref result = results[0];
    assert_eq!(rust, result.0);

    graph.cypher().exec::<()>("MATCH (n:STRUCT_INTG_TEST_4) DELETE n".into()).unwrap();
}

#[test]
fn transaction_create_on_begin_rollback() {
    let rust = Language {
        name: "Rust".to_owned(),
        level: "low".to_owned(),
        safe: true,
    };

    let graph = GraphClient::connect(URI).unwrap();

    let statement = Statement::new(
        "CREATE (n:STRUCT_INTG_TEST_5 {lang})")
        .with_param("lang", rust.clone());

    let (mut transaction, _) = graph.cypher().transaction()
        .begin::<()>(Some(statement))
        .unwrap();

    let results: Vec<(Language,)> = transaction
        .exec("MATCH (n:STRUCT_INTG_TEST_5) RETURN n".into())
        .unwrap();

    assert_eq!(1, results.len());

    let ref result = results[0];
    assert_eq!(rust, result.0);

    transaction.rollback().unwrap();

    let results: Vec<(Language,)> = graph.cypher()
        .exec("MATCH (n:STRUCT_INTG_TEST_5) RETURN n".into())
        .unwrap();

    assert_eq!(0, results.len());
}

#[test]
fn transaction_create_after_begin_rollback() {
    let rust = Language {
        name: "Rust".to_owned(),
        level: "low".to_owned(),
        safe: true,
    };

    let graph = GraphClient::connect(URI).unwrap();

    let statement = Statement::new(
        "CREATE (n:STRUCT_INTG_TEST_6 {lang})")
        .with_param("lang", rust.clone());

    let (mut transaction, _) = graph.cypher().transaction()
        .begin::<()>(None)
        .unwrap();

    transaction.exec::<()>(statement).unwrap();

    let results: Vec<(Language,)> = transaction
        .exec("MATCH (n:STRUCT_INTG_TEST_6) RETURN n".into())
        .unwrap();

    assert_eq!(1, results.len());

    let ref result = results[0];
    assert_eq!(rust, result.0);

    transaction.rollback().unwrap();

    let results: Vec<(Language,)> = graph.cypher()
        .exec("MATCH (n:STRUCT_INTG_TEST_6) RETURN n".into())
        .unwrap();

    assert_eq!(0, results.len());
}

#[test]
fn macro_without_params() {
    let graph = GraphClient::connect(URI).unwrap();

    let stmt = cypher_stmt!("MATCH (n:STRUCT_INTG_TEST_7) RETURN n");

    graph.cypher().exec::<()>(stmt).unwrap();
}

#[test]
fn save_retrive_struct() {
    let rust = Language {
        name: "Rust".to_owned(),
        level: "low".to_owned(),
        safe: true,
    };

    let graph = GraphClient::connect(URI).unwrap();

    let stmt = cypher_stmt!("CREATE (n:STRUCT_INTG_TEST_8 {lang}) RETURN n" {
        "lang" => rust.clone()
    });

    let results: Vec<(Language,)> = graph.cypher().exec(stmt).unwrap();
    assert_eq!(1, results.len());

    let ref result = results[0];
    assert_eq!(rust, result.0);

    graph.cypher().exec::<()>("MATCH (n:STRUCT_INTG_TEST_8) DELETE n".into()).unwrap();
}
