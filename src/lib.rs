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
//! use rusted_cypher::{GraphClient, Statement};
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
//! # use rusted_cypher::{GraphClient, Statement};
//! # let graph = GraphClient::connect("http://neo4j:neo4j@localhost:7474/db/data").unwrap();
//! // Statement implements From<&str>
//! graph.cypher().exec::<()>(
//!     "CREATE (n:LANG { name: 'Rust', level: 'low', safe: true })".into()
//! ).unwrap();
//!
//! let statement = Statement::new(
//!     "CREATE (n:LANG { name: {name}, level: {level}, safe: {safe} })")
//!     .with_param("name", "Python".to_owned())
//!     .with_param("level", "high".to_owned())
//!     .with_param("safe", true);
//!
//! graph.cypher().exec::<()>(statement).unwrap();
//!
//! let results: Vec<(String, String, bool)> = graph.cypher().exec(
//!     "MATCH (n:LANG) RETURN n.name, n.level, n.safe".into()
//! ).unwrap();
//!
//! assert_eq!(results.len(), 2);
//!
//! for row in results {
//!     let ref name = row.0;
//!     let ref level = row.1;
//!     let safeness = row.2;
//!     println!("name: {}, level: {}, safe: {}", name, level, safeness);
//! }
//!
//! graph.cypher().exec::<()>("MATCH (n:LANG) DELETE n".into()).unwrap();
//! ```
//!
//! ## With Transactions
//!
//! ```
//! # use std::collections::BTreeMap;
//! # use rusted_cypher::{GraphClient, Statement, CypherResult};
//! # let graph = GraphClient::connect("http://neo4j:neo4j@localhost:7474/db/data").unwrap();
//! let statement: Statement =
//!     "CREATE (n:IN_TRANSACTION { name: 'Rust', level: 'low', safe: true })"
//!     .into();
//!
//! let (mut transaction, _) = graph.cypher().transaction()
//!     .begin::<()>(Some(statement))
//!     .unwrap();
//!
//! // Use `exec` to execute a single statement
//! let statement: Statement =
//!     "CREATE (n:IN_TRANSACTION { name: 'Python', level: 'high', safe: true })"
//!     .into();
//!
//! transaction.exec::<()>(statement).unwrap();
//!
//! // Return values are tuples
//! let statement: Statement =
//!     "MATCH (n:IN_TRANSACTION) RETURN n.name, n.level, n.safe"
//!     .into();
//!
//! let results: Vec<(String, String, bool)> = transaction.exec(statement).unwrap();
//! // or let results = transaction::exec::<(String, String, bool)>(statement).unwrap();
//! assert_eq!(2, results.len());
//!
//! transaction.rollback();
//! ```
//!
//! ## Statements with Macro
//!
//! There is a macro to help building statements
//!
//! ```
//! # #[macro_use] extern crate rusted_cypher;
//! # use rusted_cypher::{GraphClient, Statement};
//! # fn main() {
//! # let graph = GraphClient::connect("http://neo4j:neo4j@localhost:7474/db/data").unwrap();
//! let statement = cypher_stmt!(
//!     "CREATE (n:WITH_MACRO { name: {name}, level: {level}, safe: {safe} })" {
//!         "name" => "Rust".to_owned(),
//!         "level" => "low".to_owned(),
//!         "safe" => true
//!     }
//! );
//! graph.cypher().exec::<()>(statement).unwrap();
//!
//! let statement = cypher_stmt!(
//!     "MATCH (n:WITH_MACRO) WHERE n.name = {name} RETURN n.level, n.safe" {
//!         "name" => "Rust".to_owned()
//!     }
//! );
//!
//! let results: Vec<(String, bool)> = graph.cypher().exec(statement).unwrap();
//! assert_eq!(results.len(), 1);
//!
//! let statement = cypher_stmt!("MATCH (n:WITH_MACRO) DELETE n");
//! graph.cypher().exec::<()>(statement).unwrap();
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
