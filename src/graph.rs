use std::error::Error;
use std::io::Read;
use hyper::{Client, Url};
use hyper::header::{Authorization, Basic, ContentType, Headers};
use rustc_serialize::json;
use semver::Version;

use cypher::Cypher;
use error::GraphError;
use cypher::result::{QueryResult, ResultTrait};

#[derive(PartialEq, RustcDecodable)]
#[allow(dead_code)]
pub struct ServiceRoot {
    pub node: String,
    pub node_index: String,
    pub relationship_index: String,
    pub extensions_info: String,
    pub relationship_types: String,
    pub batch: String,
    pub cypher: String,
    pub indexes: String,
    pub constraints: String,
    pub transaction: String,
    pub node_labels: String,
    pub neo4j_version: String,
}

fn decode_service_root(json_string: &str) -> Result<ServiceRoot, GraphError> {
    let result = json::decode::<ServiceRoot>(json_string);

    result.map_err(|_| {
        match json::decode::<QueryResult<()>>(json_string) {
            Ok(result) => GraphError::new_neo4j_error(result.errors().clone()),
            Err(e) => GraphError::new_error(Box::new(e)),
        }
    })
}

#[allow(dead_code)]
pub struct GraphClient {
    client: Client,
    headers: Headers,
    service_root: ServiceRoot,
    neo4j_version: Version,
    cypher: Cypher,
}

impl GraphClient {
    pub fn connect(endpoint: &str) -> Result<Self, GraphError> {
        let url = match Url::parse(endpoint) {
            Ok(url) => url,
            Err(e) => {
                error!("Unable to parse URL");
                return Err(GraphError::new_error(Box::new(e)));
            },
        };

        let mut headers = Headers::new();

        url.username().map(|username| url.password().map(|password| {
            headers.set(Authorization(
                Basic {
                    username: username.to_owned(),
                    password: Some(password.to_owned()),
                }
            ));
        }));

        headers.set(ContentType::json());

        let client = Client::new();
        let mut res = match client.get(url.clone()).headers(headers.clone()).send() {
            Ok(res) => res,
            Err(e) => {
                error!("Unable to connect to server: {}", e);
                return Err(GraphError::new_error(Box::new(e)));
            },
        };

        let mut buf = String::new();
        if let Err(e) = res.read_to_string(&mut buf) {
            return Err(GraphError::new_error(Box::new(e)));
        }

        let service_root = try!(decode_service_root(&buf));

        let neo4j_version = match Version::parse(&service_root.neo4j_version) {
            Ok(value) => value,
            Err(e) => return Err(GraphError::new_error(Box::new(e))),
        };
        let cypher_endpoint = try!(Url::parse(&service_root.transaction));

        let cypher = Cypher::new(cypher_endpoint, headers.clone());

        Ok(GraphClient {
            client: Client::new(),
            headers: headers,
            service_root: service_root,
            neo4j_version: neo4j_version,
            cypher: cypher,
        })
    }

    pub fn neo4j_version(&self) -> &Version {
        &self.neo4j_version
    }

    /// Returns a reference to the `Cypher` instance of the `GraphClient`
    pub fn cypher(&self) -> &Cypher {
        &self.cypher
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const URL: &'static str = "http://neo4j:neo4j@localhost:7474/db/data";

    #[test]
    fn connect() {
        let graph = GraphClient::connect(URL);
        assert!(graph.is_ok());
        let graph = graph.unwrap();
        assert!(graph.neo4j_version().major >= 2);
    }

    #[test]
    fn query() {
        let graph = GraphClient::connect(URL).unwrap();

        graph.cypher().exec::<()>("MATCH (n:GRAPH_QUERY) RETURN n".into()).unwrap();
    }

    // #[test]
    // fn transaction() {
    //     let graph = GraphClient::connect(URL).unwrap();
    //
    //     let (transaction, result) = graph.cypher().transaction()
    //         .with_statement("MATCH n RETURN n")
    //         .begin()
    //         .unwrap();
    //
    //     assert_eq!(result[0].columns.len(), 1);
    //     assert_eq!(result[0].columns[0], "n");
    //
    //     transaction.rollback().unwrap();
    // }
}
