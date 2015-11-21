//! Rust crate for accessing the cypher endpoint of a neo4j server
//!
//! This is a prototype for accessing the cypher endpoint of a neo4j server, like a sql
//! driver for a relational database.
//!
//! You can execute queries inside a transaction or simply execute queries that commit immediately.
//!
//! # Examples
//!
//! Code in examples are assumed to be wrapped in:
//!
//! ```
//! extern crate rusted_cypher;
//!
//! use std::collections::BTreeMap;
//! use rusted_cypher::GraphClient;
//! use rusted_cypher::cypher::Statement;
//!
//! fn main() {
//!   // Connect to the database
//!   let graph = GraphClient::connect(
//!       "http://neo4j:neo4j@localhost:7474/db/data").unwrap();
//!
//!   // Example code here!
//! }
//! ```
//!
//! ## Performing Queries
//!
//! ```
//! # use rusted_cypher::{GraphClient, Statement, CypherResult};
//! # let graph = GraphClient::connect("http://neo4j:neo4j@localhost:7474/db/data").unwrap();
//! // Statement implements From<&str>
//! graph.cypher().exec(
//!		"CREATE (n:LANG { name: 'Rust', level: 'low', safe: true })"
//!	).unwrap();
//!
//!	let statement = Statement::new(
//!		"CREATE (n:LANG { name: {name}, level: {level}, safe: {safe} })")
//!		.with_param("name", "Python".to_owned())
//!		.with_param("level", "high".to_owned())
//!		.with_param("safe", true);
//!
//! graph.cypher().exec(statement).unwrap();
//!
//! let result: CypherResult<(String, String, bool)> = graph.cypher().query(
//!     "MATCH (n:LANG) RETURN n.name, n.level, n.safe"
//!	).unwrap();
//!
//! assert_eq!(result.rows().len(), 2);
//!
//! for row in result.rows() {
//!     let ref name = row.0;
//!     let ref level = row.1;
//!     let safeness = row.2;
//!     println!("name: {}, level: {}, safe: {}", name, level, safeness);
//! }
//!
//! graph.cypher().exec("MATCH (n:LANG) DELETE n").unwrap();
//! ```
//!
//! ## With Transactions
//!
//! ```ignore
//! # use std::collections::BTreeMap;
//! # use rusted_cypher::{GraphClient, Statement, CypherResult};
//! # let graph = GraphClient::connect("http://neo4j:neo4j@localhost:7474/db/data").unwrap();
//! let transaction = graph.cypher().transaction()
//!     .with_statement("CREATE (n:IN_TRANSACTION { name: 'Rust', level: 'low', safe: true })");
//!
//! let (mut transaction, results) = transaction.begin().unwrap();
//!
//! // Use `exec` to execute a single statement
//! transaction.exec("CREATE (n:IN_TRANSACTION { name: 'Python', level: 'high', safe: true })")
//!     .unwrap();
//!
//! // use `add_statement` (or `with_statement`) and `send` to executes multiple statements
//! let stmt = Statement::new("MATCH (n:IN_TRANSACTION) WHERE (n.safe = {safeness}) RETURN n")
//!     .with_param("safeness", true);
//!
//! transaction.add_statement(stmt);
//! let results = transaction.send().unwrap();
//!
//! assert_eq!(results[0].data.len(), 2);
//!
//! transaction.rollback();
//! ```
//!
//! ## Statements with Macro
//!
//! There is a macro to help building statements
//!
//! ```ignore
//! # #[macro_use] extern crate rusted_cypher;
//! # use rusted_cypher::GraphClient;
//! # use rusted_cypher::cypher::Statement;
//! # fn main() {
//! # let graph = GraphClient::connect("http://neo4j:neo4j@localhost:7474/db/data").unwrap();
//! let statement = cypher_stmt!(
//!     "CREATE (n:WITH_MACRO { name: {name}, level: {level}, safe: {safe} })" {
//!         "name" => "Rust",
//!         "level" => "low",
//!         "safe" => true
//!     }
//! );
//! graph.cypher().exec(statement).unwrap();
//!
//! let statement = cypher_stmt!("MATCH (n:WITH_MACRO) WHERE n.name = {name} RETURN n" {
//!     "name" => "Rust"
//! });
//!
//! let results = graph.cypher().exec(statement).unwrap();
//! assert_eq!(results.data.len(), 1);
//!
//! let statement = cypher_stmt!("MATCH (n:WITH_MACRO) DELETE n");
//! graph.cypher().exec(statement).unwrap();
//! # }
//! ```

extern crate hyper;
extern crate url;
extern crate rustc_serialize;
extern crate semver;
extern crate time;
#[macro_use]
extern crate log;

mod json_util;

pub mod cypher;
pub mod graph;
pub mod error;

pub use graph::GraphClient;
pub use cypher::Statement;
pub use cypher::CypherResult;
