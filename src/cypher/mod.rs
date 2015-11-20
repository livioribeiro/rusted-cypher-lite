//! Provides structs used to interact with the cypher transaction endpoint
//!
//! The types declared in this module, save for `Statement`, don't need to be instantiated
//! directly, since they can be obtained from the `GraphClient`.
//!
//! # Examples
//!
//! ## Execute a single query
//! ```
//! # use rusted_cypher::GraphClient;
//! # const URL: &'static str = "http://neo4j:neo4j@localhost:7474/db/data";
//! let graph = GraphClient::connect(URL).unwrap();
//!
//! graph.cypher().exec("CREATE (n:CYPHER_QUERY {value: 1})").unwrap();
//! let result = graph.cypher().exec("MATCH (n:CYPHER_QUERY) RETURN n.value AS value").unwrap();
//! # assert_eq!(result.data.len(), 1);
//!
//! // Iterate over the results
//! for row in result.rows() {
//!     let value = row.get::<i32>("value").unwrap(); // or: let value: i32 = row.get("value");
//!     assert_eq!(value, 1);
//! }
//! # graph.cypher().exec("MATCH (n:CYPHER_QUERY) delete n");
//! ```
//!
//! ## Execute multiple queries
//! ```
//! # use rusted_cypher::GraphClient;
//! # const URL: &'static str = "http://neo4j:neo4j@localhost:7474/db/data";
//! # let graph = GraphClient::connect(URL).unwrap();
//! let mut query = graph.cypher().query()
//!     .with_statement("MATCH (n:SOME_CYPHER_QUERY) RETURN n.value as value")
//!     .with_statement("MATCH (n:OTHER_CYPHER_QUERY) RETURN n");
//!
//! let results = query.send().unwrap();
//!
//! for row in results[0].rows() {
//!     let value: i32 = row.get("value").unwrap();
//!     assert_eq!(value, 1);
//! }
//! ```
//!
//! ## Start a transaction
//! ```
//! # use rusted_cypher::GraphClient;
//! # const URL: &'static str = "http://neo4j:neo4j@localhost:7474/db/data";
//! # let graph = GraphClient::connect(URL).unwrap();
//! let (transaction, results) = graph.cypher().transaction()
//!     .with_statement("MATCH (n:TRANSACTION_CYPHER_QUERY) RETURN n")
//!     .begin().unwrap();
//!
//! # assert_eq!(results.len(), 1);
//! ```

pub mod transaction;
pub mod statement;
pub mod result;

use std::convert::Into;
use std::collections::BTreeMap;
use hyper::client::{Client, Response};
use hyper::header::Headers;
use url::Url;
use rustc_serialize::{json, Encodable, Decodable};

use ::error::GraphError;
use ::json_util;

use self::result::{QueryResult, ResultTrait};
pub use self::statement::Statement;
pub use self::transaction::Transaction;
pub use self::result::CypherResult;

fn send_query<T: Encodable + PartialEq>(client: &Client, endpoint: &str, headers: &Headers, statements: Option<Statement<T>>)
    -> Result<Response, GraphError> {

    let mut json = BTreeMap::new();
    json.insert("statements", statements);

    let json = match json::encode(&json) {
        Ok(json) => json,
        Err(e) => {
            error!("Unable to serialize request: {}", e);
            return Err(GraphError::new_error(Box::new(e)));
        }
    };

    let req = client.post(endpoint)
        .headers(headers.clone())
        .body(&json);

    debug!("Seding query:\n{}", json::as_pretty_json(&json));

    let res = try!(req.send());
    Ok(res)
}

fn parse_response<T: Decodable, Q: Decodable + ResultTrait<T>>(res: &mut Response) -> Result<Q, GraphError> {
    let result: Q = try!(json_util::decode_from_reader(res));

    if result.errors().len() > 0 {
        return Err(GraphError::new_neo4j_error(result.errors().clone()));
    }

    Ok(result)
}

/// Represents the cypher endpoint of a neo4j server
///
/// The `Cypher` struct holds information about the cypher enpoint. It is used to create the queries
/// that are sent to the server.
pub struct Cypher {
    endpoint: Url,
    client: Client,
    headers: Headers,
}

impl Cypher {
    /// Creates a new Cypher
    ///
    /// Its arguments are the cypher transaction endpoint and the HTTP headers containing HTTP
    /// Basic Authentication, if needed.
    pub fn new(endpoint: Url, headers: Headers) -> Self {
        Cypher {
            endpoint: endpoint,
            client: Client::new(),
            headers: headers,
        }
    }

    /// Executes the given `Statement`
    ///
    /// Parameter can be anything that implements `Into<Statement>`, `&str` or `Statement` itself
    pub fn exec<T: Decodable, Q: Encodable + PartialEq, S: Into<Statement<Q>>>(&self, statement: S)
        -> Result<CypherResult<T>, GraphError>
    {
        let endpoint = format!("{}/{}", &self.endpoint, "commit");
        let mut res = try!(send_query(&self.client,
                                      &endpoint,
                                      &self.headers,
                                      Some(statement.into())));

        let mut result: QueryResult<T> = try!(parse_response(&mut res));
        if result.errors().len() > 0 {
            return Err(GraphError::new_neo4j_error(result.errors().clone()))
        }

        result.results.pop().ok_or(GraphError::new("No results returned from server"))
    }

    /// Creates a new `Transaction`
    pub fn transaction(&self) -> Transaction<self::transaction::Created> {
        Transaction::new(&self.endpoint.to_string(), &self.headers)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     fn get_cypher() -> Cypher {
//         use hyper::Url;
//         use hyper::header::{Authorization, Basic, ContentType, Headers};
//
//         let cypher_endpoint = Url::parse("http://localhost:7474/db/data/transaction").unwrap();
//
//         let mut headers = Headers::new();
//         headers.set(Authorization(
//             Basic {
//                 username: "neo4j".to_owned(),
//                 password: Some("neo4j".to_owned()),
//             }
//         ));
//         headers.set(ContentType::json());
//
//         Cypher::new(cypher_endpoint, headers)
//     }
//
//     #[test]
//     fn query_without_params() {
//         let result = get_cypher().exec("MATCH (n:TEST_CYPHER) RETURN n").unwrap();
//
//         assert_eq!(result.columns.len(), 1);
//         assert_eq!(result.columns[0], "n");
//     }
//
//     #[test]
//     fn query_with_string_param() {
//         let statement = Statement::new("MATCH (n:TEST_CYPHER {name: {name}}) RETURN n")
//             .with_param("name", "Neo");
//
//         let result = get_cypher().exec(statement).unwrap();
//
//         assert_eq!(result.columns.len(), 1);
//         assert_eq!(result.columns[0], "n");
//     }
//
//     #[test]
//     fn query_with_int_param() {
//         let statement = Statement::new("MATCH (n:TEST_CYPHER {value: {value}}) RETURN n")
//             .with_param("value", 42);
//
//         let result = get_cypher().exec(statement).unwrap();
//
//         assert_eq!(result.columns.len(), 1);
//         assert_eq!(result.columns[0], "n");
//     }
//
//     #[test]
//     fn query_with_complex_param() {
//         #[cfg(not(feature = "rustc-serialize"))]
//         mod inner {
//             #[derive(RustcEncodable, RustcDecodable)]
//             pub struct ComplexType {
//                 pub name: String,
//                 pub value: i32,
//             }
//         }
//
//         #[cfg(feature = "rustc-serialize")]
//         mod inner {
//             #[derive(RustcEncodable, RustcDecodable)]
//             pub struct ComplexType {
//                 pub name: String,
//                 pub value: i32,
//             }
//         }
//
//         let cypher = get_cypher();
//
//         let complex_param = inner::ComplexType {
//             name: "Complex".to_owned(),
//             value: 42,
//         };
//
//         let statement = Statement::new("CREATE (n:TEST_CYPHER_COMPLEX_PARAM {p})")
//             .with_param("p", &complex_param);
//
//         let result = cypher.exec(statement);
//         assert!(result.is_ok());
//
//         let results = cypher.exec("MATCH (n:TEST_CYPHER_COMPLEX_PARAM) RETURN n").unwrap();
//         let rows: Vec<Row> = results.rows().take(1).collect();
//         let row = rows.first().unwrap();
//
//         let complex_result: inner::ComplexType = row.get("n").unwrap();
//         assert_eq!(complex_result.name, "Complex");
//         assert_eq!(complex_result.value, 42);
//
//         cypher.exec("MATCH (n:TEST_CYPHER_COMPLEX_PARAM) DELETE n").unwrap();
//     }
//
//     #[test]
//     fn query_with_multiple_params() {
//         let statement = Statement::new(
//             "MATCH (n:TEST_CYPHER {name: {name}}) WHERE n.value = {value} RETURN n")
//             .with_param("name", "Neo")
//             .with_param("value", 42);
//
//         let result = get_cypher().exec(statement).unwrap();
//         assert_eq!(result.columns.len(), 1);
//         assert_eq!(result.columns[0], "n");
//     }
//
//     #[test]
//     fn multiple_queries() {
//         let cypher = get_cypher();
//         let statement1 = Statement::new("MATCH (n:TEST_CYPHER) RETURN n");
//         let statement2 = Statement::new("MATCH (n:TEST_CYPHER) RETURN n");
//
//         let query = cypher.query()
//             .with_statement(statement1)
//             .with_statement(statement2);
//
//         let results = query.send().unwrap();
//         assert_eq!(results.len(), 2);
//     }
// }
