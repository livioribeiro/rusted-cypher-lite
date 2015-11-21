//! Provides structs used to interact with the cypher transaction endpoint
//!
//! The types declared in this module, save for `Statement`, don't need to be instantiated
//! directly, since they can be obtained from the `GraphClient`.
//!
//! # Examples
//!
//! ## Execute a query
//! ```
//! # use rusted_cypher::{GraphClient, CypherResult};
//! # const URL: &'static str = "http://neo4j:neo4j@localhost:7474/db/data";
//! let graph = GraphClient::connect(URL).unwrap();
//!
//! // use `Cypher::exec` when the query does not return any value
//! graph.cypher().exec("CREATE (n:CYPHER_QUERY { value: 1 })").unwrap();
//!
//! // use `Cypher::query` when the query returns values 
//! let result: CypherResult<(i32,)> = graph.cypher().query(
//!     "MATCH (n:CYPHER_QUERY) RETURN n.value AS value"
//! ).unwrap();
//! # assert_eq!(result.rows().len(), 1);
//!
//! // Iterate over the results
//! for row in result.rows() {
//!     let value = row.0;
//!     assert_eq!(value, 1);
//! }
//! # graph.cypher().exec("MATCH (n:CYPHER_QUERY) delete n").unwrap();
//! ```
//!
//! ## Start a transaction
//! ```ignore
//! # use rusted_cypher::GraphClient;
//! # const URL: &'static str = "http://neo4j:neo4j@localhost:7474/db/data";
//! # let graph = GraphClient::connect(URL).unwrap();
//! let (transaction, results) = graph.cypher().transaction()
//!     .with_statement("MATCH (n:TRANSACTION_CYPHER_QUERY) RETURN n")
//!     .begin().unwrap();
//!
//! # assert_eq!(results.len(), 1);
//! ```

// pub mod transaction;
pub mod statement;
pub mod result;

use std::convert::Into;
use hyper::client::{Client, Response};
use hyper::header::Headers;
use url::Url;
use rustc_serialize::{json, Encodable, Decodable};

use ::error::GraphError;
use ::json_util;

use self::result::{QueryResult, ResultTrait};
pub use self::statement::Statement;
// pub use self::transaction::Transaction;
pub use self::result::CypherResult;

#[derive(RustcEncodable)]
struct Statements {
    statements: Vec<Statement>,
}

fn send_query(client: &Client, endpoint: &str, headers: &Headers, statement: Option<Statement>)
    -> Result<Response, GraphError>
{
    let json_string: String;
    
    if let Some(statement) = statement {
        let statements = Statements { statements: vec![statement] };
    
        json_string = match json::encode(&statements) {
            Ok(value) => value,
            Err(e) => {
                error!("Unable to serialize request: {}", e);
                return Err(GraphError::new_error(Box::new(e)));
            }
        };
    } else {
        json_string = String::new();
    }

    let req = client.post(endpoint)
        .headers(headers.clone())
        .body(&json_string);

    debug!("Seding query:\n{}", json::as_pretty_json(&json_string));

    let res = try!(req.send());
    Ok(res)
}

fn parse_response<T: Decodable, Q: Decodable + ResultTrait<T>>(res: &mut Response) -> Result<Q, GraphError> {
    let result: Q = match json_util::decode_from_reader(res) {
        Ok(value) => value,
        Err(e) => {
            error!("Unable to parse response: {}", e);
            return Err(GraphError::new_error(Box::new(e)))
        }
    };

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
    
    fn send_query<T: Decodable, S: Into<Statement>>(&self, statement: S)
        -> Result<QueryResult<T>, GraphError>
    {
        let endpoint = format!("{}/{}", &self.endpoint, "commit");
        let mut res = try!(send_query(&self.client,
                                      &endpoint,
                                      &self.headers,
                                      Some(statement.into())));

        let result: QueryResult<T> = try!(parse_response(&mut res));
        if result.errors().len() > 0 {
            return Err(GraphError::new_neo4j_error(result.errors().clone()))
        }
        
        Ok(result)
    }
    
    /// Executes the given `Statement`, returning the results
    ///
    /// Parameter can be anything that implements `Into<Statement>`, `&str` or `Statement` itself
    pub fn query<T: Decodable, S: Into<Statement>>(&self, statement: S)
        -> Result<CypherResult<T>, GraphError>
    {
        let mut result: QueryResult<T> = try!(self.send_query(statement));

        result.results.pop().ok_or(GraphError::new("No results returned from server"))
    }
    
    /// Execute the given statement
    pub fn exec<S: Into<Statement>>(&self, statement: S) -> Result<(), GraphError> {
        let _: QueryResult<()> = try!(self.send_query(statement));
        
        Ok(())
    }

    // /// Creates a new `Transaction`
    // pub fn transaction(&self) -> Transaction<self::transaction::Created> {
    //     Transaction::new(&self.endpoint.to_string(), &self.headers)
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_cypher() -> Cypher {
        use hyper::Url;
        use hyper::header::{Authorization, Basic, ContentType, Headers};

        let cypher_endpoint = Url::parse("http://localhost:7474/db/data/transaction").unwrap();

        let mut headers = Headers::new();
        headers.set(Authorization(
            Basic {
                username: "neo4j".to_owned(),
                password: Some("neo4j".to_owned()),
            }
        ));
        headers.set(ContentType::json());

        Cypher::new(cypher_endpoint, headers)
    }

    #[test]
    fn query_without_params() {
        let result: CypherResult<()> = get_cypher().query("MATCH (n:TEST_CYPHER) RETURN n").unwrap();

        assert_eq!(result.columns().len(), 1);
        assert_eq!(result.columns()[0], "n");
    }

    #[test]
    fn query_with_string_param() {
        let statement = Statement::new("MATCH (n:TEST_CYPHER {name: {name}}) RETURN n")
            .with_param("name", "Neo".to_owned());

        let result: CypherResult<()> = get_cypher().query(statement).unwrap();

        assert_eq!(result.columns().len(), 1);
        assert_eq!(result.columns()[0], "n");
    }

    #[test]
    fn query_with_int_param() {
        let statement = Statement::new("MATCH (n:TEST_CYPHER {value: {value}}) RETURN n")
            .with_param("value", 42);

        let result: CypherResult<()> = get_cypher().query(statement).unwrap();

        assert_eq!(result.columns().len(), 1);
        assert_eq!(result.columns()[0], "n");
    }

    #[test]
    fn query_with_complex_param() {
        use std::collections::BTreeMap;
        use rustc_serialize::json::{Json, ToJson};
        
        #[derive(Clone, RustcEncodable, RustcDecodable)]
        struct ComplexType {
            name: String,
            value: i32,
        }
        
        impl ToJson for ComplexType {
            fn to_json(&self) -> Json {
                let mut map: BTreeMap<String, Json> = BTreeMap::new();
                map.insert("name".to_owned(), Json::String(self.name.clone()));
                map.insert("value".to_owned(), Json::I64(self.value as i64));
                
                Json::Object(map)
            }
        }

        let cypher = get_cypher();

        let complex_param = ComplexType {
            name: "Complex".to_owned(),
            value: 42,
        };

        let statement = Statement::new("CREATE (n:TEST_CYPHER_COMPLEX_PARAM {p})")
            .with_param("p", complex_param.clone());

        cypher.exec(statement).unwrap();

        let result: CypherResult<(ComplexType,)> = cypher.query("MATCH (n:TEST_CYPHER_COMPLEX_PARAM) RETURN n").unwrap();
        let row = result.rows().first().unwrap();

        let ref complex_result = row.data().0;
        assert_eq!(complex_result.name, "Complex");
        assert_eq!(complex_result.value, 42);

       cypher.exec("MATCH (n:TEST_CYPHER_COMPLEX_PARAM) DELETE n").unwrap();
    }

    #[test]
    fn query_with_multiple_params() {
        let statement = Statement::new(
            "MATCH (n:TEST_CYPHER {name: {name}}) WHERE n.value = {value} RETURN n")
            .with_param("name", "Neo".to_owned())
            .with_param("value", 42);

        let result: CypherResult<()> = get_cypher().query(statement).unwrap();
        assert_eq!(result.columns().len(), 1);
        assert_eq!(result.columns()[0], "n");
    }
}
