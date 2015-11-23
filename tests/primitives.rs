#[macro_use]
extern crate rusted_cypher;

use rusted_cypher::{GraphClient, Statement};

const URI: &'static str = "http://neo4j:neo4j@127.0.0.1:7474/db/data";

#[test]
fn save_retrive_values() {
    let graph = GraphClient::connect(URI).unwrap();

    let statement = Statement::new(
        "CREATE (n:INTG_TEST_1 {name: {name}, level: {level}, safe: {safe}}) RETURN n.name, n.level, n.safe")
        .with_param("name", "Rust".to_owned())
        .with_param("level", "low".to_owned())
        .with_param("safe", true);

    let results: Vec<(String, String, bool)> = graph.cypher().exec(statement).unwrap();
    assert_eq!(1, results.len());

    let ref result = results[0];

    assert_eq!("Rust", result.0);
    assert_eq!("low", result.1);
    assert_eq!(true, result.2);

    graph.cypher().exec::<()>("MATCH (n:INTG_TEST_1) DELETE n".into()).unwrap();
}

#[test]
fn transaction_create_on_begin_commit() {
    let graph = GraphClient::connect(URI).unwrap();

    let statement = Statement::new(
        "CREATE (n:INTG_TEST_2 {name: {name}, level: {level}, safe: {safe}})")
        .with_param("name", "Rust".to_owned())
        .with_param("level", "low".to_owned())
        .with_param("safe", true);

    graph.cypher().transaction()
        .begin::<()>(Some(statement))
        .unwrap()
        .0.commit::<()>(None)
        .unwrap();

    let results: Vec<(String, String, bool)> = graph.cypher()
        .exec("MATCH (n:INTG_TEST_2) RETURN n.name, n.level, n.safe".into())
        .unwrap();

    assert_eq!(1, results.len());

    let ref result = results[0];

    assert_eq!("Rust", result.0);
    assert_eq!("low", result.1);
    assert_eq!(true, result.2);

    graph.cypher().exec::<()>("MATCH (n:INTG_TEST_2) DELETE n".into()).unwrap();
}

#[test]
fn transaction_create_after_begin_commit() {
    let graph = GraphClient::connect(URI).unwrap();
    let (mut transaction, _) = graph.cypher().transaction().begin::<()>(None).unwrap();

    let statement = Statement::new(
        "CREATE (n:INTG_TEST_3 {name: {name}, level: {level}, safe: {safe}})")
        .with_param("name", "Rust".to_owned())
        .with_param("level", "low".to_owned())
        .with_param("safe", true);

    transaction.exec::<()>(statement).unwrap();
    transaction.commit::<()>(None).unwrap();

    let results: Vec<(String, String, bool)> = graph.cypher()
        .exec("MATCH (n:INTG_TEST_3) RETURN n.name, n.level, n.safe".into())
        .unwrap();

    assert_eq!(1, results.len());

    let ref result = results[0];

    assert_eq!("Rust", result.0);
    assert_eq!("low", result.1);
    assert_eq!(true, result.2);

    graph.cypher().exec::<()>("MATCH (n:INTG_TEST_3) DELETE n".into()).unwrap();
}

#[test]
fn transaction_create_on_commit() {
    let graph = GraphClient::connect(URI).unwrap();

    let statement = Statement::new(
        "CREATE (n:INTG_TEST_4 {name: {name}, level: {level}, safe: {safe}})")
        .with_param("name", "Rust".to_owned())
        .with_param("level", "low".to_owned())
        .with_param("safe", true);

    let (transaction, _) = graph.cypher().transaction().begin::<()>(None).unwrap();
    transaction.commit::<()>(Some(statement)).unwrap();

    let results: Vec<(String, String, bool)> = graph.cypher()
        .exec("MATCH (n:INTG_TEST_4) RETURN n.name, n.level, n.safe".into())
        .unwrap();

    assert_eq!(1, results.len());

    let ref result = results[0];

    assert_eq!("Rust", result.0);
    assert_eq!("low", result.1);
    assert_eq!(true, result.2);

    graph.cypher().exec::<()>("MATCH (n:INTG_TEST_4) DELETE n".into()).unwrap();
}

#[test]
fn transaction_create_on_begin_rollback() {
    let graph = GraphClient::connect(URI).unwrap();

    let statement = Statement::new(
        "CREATE (n:INTG_TEST_5 {name: {name}, level: {level}, safe: {safe}})")
        .with_param("name", "Rust".to_owned())
        .with_param("level", "low".to_owned())
        .with_param("safe", true);

    let (mut transaction, _) = graph.cypher().transaction()
        .begin::<()>(Some(statement))
        .unwrap();

    let results: Vec<(String, String, bool)> = transaction
        .exec("MATCH (n:INTG_TEST_5) RETURN n.name, n.level, n.safe".into())
        .unwrap();

    assert_eq!(1, results.len());

    let ref result = results[0];

    assert_eq!("Rust", result.0);
    assert_eq!("low", result.1);
    assert_eq!(true, result.2);

    transaction.rollback().unwrap();

    let results: Vec<()> = graph.cypher()
        .exec("MATCH (n:INTG_TEST_5) RETURN n".into())
        .unwrap();

    assert_eq!(0, results.len());
}

#[test]
fn transaction_create_after_begin_rollback() {
    let graph = GraphClient::connect(URI).unwrap();
    let (mut transaction, _) = graph.cypher().transaction().begin::<()>(None).unwrap();

    let statement = Statement::new(
        "CREATE (n:INTG_TEST_6 {name: {name}, level: {level}, safe: {safe}})")
        .with_param("name", "Rust".to_owned())
        .with_param("level", "low".to_owned())
        .with_param("safe", true);

    transaction.exec::<()>(statement).unwrap();

    let results: Vec<(String, String, bool)> = transaction
        .exec("MATCH (n:INTG_TEST_6) RETURN n.name, n.level, n.safe".into())
        .unwrap();

    assert_eq!(1, results.len());

    let ref result = results[0];

    assert_eq!("Rust", result.0);
    assert_eq!("low", result.1);
    assert_eq!(true, result.2);

    transaction.rollback().unwrap();

    let results: Vec<()> = graph.cypher()
        .exec("MATCH (n:INTG_TEST_6) RETURN n".into())
        .unwrap();

    assert_eq!(0, results.len());
}

#[test]
fn macro_without_params() {
    let graph = GraphClient::connect(URI).unwrap();

    let stmt = cypher_stmt!("MATCH (n:INTG_TEST_7) RETURN n");
    graph.cypher().exec::<()>(stmt).unwrap();
}

#[test]
fn macro_save_retrive_values() {
    let graph = GraphClient::connect(URI).unwrap();

    let stmt = cypher_stmt!(
        "CREATE (n:INTG_TEST_8 {name: {name}, level: {level}, safe: {safe}}) RETURN n.name, n.level, n.safe" {
            "name" => "Rust".to_owned(),
            "level" => "low".to_owned(),
            "safe" => true
        }
    );

    let results: Vec<(String, String, bool)> = graph.cypher().exec(stmt).unwrap();

    assert_eq!(1, results.len());

    let ref result = results[0];

    assert_eq!("Rust", result.0);
    assert_eq!("low", result.1);
    assert_eq!(true, result.2);

    graph.cypher().exec::<()>("MATCH (n:INTG_TEST_8) DELETE n".into()).unwrap();
}
