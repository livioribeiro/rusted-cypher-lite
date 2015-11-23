//! Transaction management through neo4j's transaction endpoint
//!
//! The recommended way to start a transaction is through the `GraphClient`
//!
//! # Examples
//!
//! ## Starting a transaction
//! ```
//! # #![allow(unused_variables)]
//! # use rusted_cypher::GraphClient;
//! # const URL: &'static str = "http://neo4j:neo4j@localhost:7474/db/data";
//! let graph = GraphClient::connect(URL).unwrap();
//!
//! let mut transaction = graph.cypher().transaction();
//! transaction.add_statement("MATCH (n:TRANSACTION) RETURN n");
//!
//! let (transaction, results) = transaction.begin().unwrap();
//! # transaction.rollback().unwrap();
//! ```
//!
//! ## Statement is optional when beggining a transaction
//! ```
//! # #![allow(unused_variables)]
//! # use rusted_cypher::GraphClient;
//! # const URL: &'static str = "http://neo4j:neo4j@localhost:7474/db/data";
//! # let graph = GraphClient::connect(URL).unwrap();
//! let (transaction, _) = graph.cypher().transaction()
//!     .begin().unwrap();
//! # transaction.rollback().unwrap();
//! ```
//!
//! ## Send queries in a started transaction
//! ```
//! # use rusted_cypher::GraphClient;
//! # const URL: &'static str = "http://neo4j:neo4j@localhost:7474/db/data";
//! # let graph = GraphClient::connect(URL).unwrap();
//! # let (mut transaction, _) = graph.cypher().transaction().begin().unwrap();
//! // Send a single query
//! let result = transaction.exec("MATCH (n:TRANSACTION) RETURN n").unwrap();
//!
//! // Send multiple queries
//! let results = transaction
//!     .with_statement("MATCH (n:TRANSACTION) RETURN n")
//!     .with_statement("MATCH (n:OTHER_TRANSACTION) RETURN n")
//!     .send().unwrap();
//! # assert_eq!(results.len(), 2);
//! # transaction.rollback().unwrap();
//! ```
//!
//! ## Commit a transaction
//! ```
//! # use rusted_cypher::GraphClient;
//! # const URL: &'static str = "http://neo4j:neo4j@localhost:7474/db/data";
//! # let graph = GraphClient::connect(URL).unwrap();
//! # let (mut transaction, _) = graph.cypher().transaction().begin().unwrap();
//! transaction.exec("CREATE (n:TRANSACTION)").unwrap();
//! transaction.commit().unwrap();
//!
//! // Send more statements when commiting
//! # let (mut transaction, _) = graph.cypher().transaction().begin().unwrap();
//! let results = transaction
//!     .with_statement("MATCH (n:TRANSACTION) RETURN n")
//!     .send().unwrap();
//! # assert_eq!(results[0].data.len(), 1);
//! # transaction.rollback().unwrap();
//! # graph.cypher().exec("MATCH (n:TRANSACTION) DELETE n").unwrap();
//! ```
//!
//! ## Rollback a transaction
//! ```
//! # use rusted_cypher::GraphClient;
//! # const URL: &'static str = "http://neo4j:neo4j@localhost:7474/db/data";
//! # let graph = GraphClient::connect(URL).unwrap();
//! # let (mut transaction, _) = graph.cypher().transaction().begin().unwrap();
//! transaction.exec("CREATE (n:TRANSACTION)").unwrap();
//! transaction.rollback().unwrap();
//! # let result = graph.cypher().exec("MATCH (n:TRANSACTION) RETURN n").unwrap();
//! # assert_eq!(result.data.len(), 0);
//! ```

use std::any::Any;
use std::convert::Into;
use std::marker::PhantomData;
use hyper::Client;
use hyper::header::{Headers, Location};
use rustc_serialize::Decodable;
use time::{self, Tm};

use super::result::{CypherResult, ResultTrait};
use super::statement::Statement;
use ::error::{GraphError, Neo4jError};

const DATETIME_RFC822: &'static str = "%a, %d %b %Y %T %Z";

pub struct Created;
pub struct Started;

#[derive(Debug, RustcDecodable)]
struct TransactionInfo {
    expires: String,
}

#[derive(Debug, RustcDecodable)]
struct TransactionResult<T: Decodable> {
    commit: String,
    transaction: TransactionInfo,
    results: Vec<CypherResult<T>>,
    errors: Vec<Neo4jError>,
}

impl<T: Decodable> ResultTrait<T> for TransactionResult<T> {
    fn results(&self) -> &Vec<CypherResult<T>> {
        &self.results
    }

    fn errors(&self) -> &Vec<Neo4jError> {
        &self.errors
    }
}

#[derive(RustcDecodable)]
#[allow(dead_code)]
struct CommitResult<T: Decodable> {
    results: Vec<CypherResult<T>>,
    errors: Vec<Neo4jError>,
}

impl<T: Decodable> ResultTrait<T> for CommitResult<T> {
    fn results(&self) -> &Vec<CypherResult<T>> {
        &self.results
    }

    fn errors(&self) -> &Vec<Neo4jError> {
        &self.errors
    }
}

/// Provides methods to interact with a transaction
///
/// This struct is used to begin a transaction, send queries, commit an rollback a transaction.
/// Some methods are provided depending on the state of the transaction, for example,
/// `Transaction::begin` is provided on a `Created` transaction and `Transaction::commit` is provided
/// on `Started` transaction
pub struct Transaction<'a, State: Any = Created> {
    transaction: String,
    commit: String,
    expires: Tm,
    client: Client,
    headers: &'a Headers,
    _state: PhantomData<State>,
}

impl<'a, State: Any> Transaction<'a, State> {
    /// Gets the expiration time of the transaction
    pub fn get_expires(&self) -> &Tm {
        &self.expires
    }
}

impl<'a> Transaction<'a, Created> {
    pub fn new(endpoint: &str, headers: &'a Headers) -> Transaction<'a, Created> {
        Transaction {
            transaction: endpoint.to_owned(),
            commit: endpoint.to_owned(),
            expires: time::now_utc(),
            client: Client::new(),
            headers: headers,
            _state: PhantomData,
        }
    }

    /// Begins the transaction
    ///
    /// Consumes the `Transaction<Created>` and returns the a `Transaction<Started>` alongside with
    /// the results of any `Statement` sent.
    pub fn begin<T: Decodable>(self, statement: Option<Statement>)
        -> Result<(Transaction<'a, Started>, Vec<T>), GraphError>
    {
        debug!("Beginning transaction");

        let statement = statement.map(|statement| statement.into());
        let mut res = try!(super::send_query(&self.client,
                                             &self.transaction,
                                             self.headers,
                                             statement));

        let mut result: TransactionResult<T> = try!(super::parse_response(&mut res));

        let transaction = match res.headers.get::<Location>() {
            Some(location) => location.0.to_owned(),
            None => {
                error!("No transaction URI returned from server");
                return Err(GraphError::new("No transaction URI returned from server"));
            },
        };

        let mut expires = result.transaction.expires;
        let expires = try!(time::strptime(&mut expires, DATETIME_RFC822));

        debug!("Transaction started at {}, expires in {}", transaction, expires.rfc822z());

        let transaction = Transaction {
            transaction: transaction,
            commit: result.commit,
            expires: expires,
            client: self.client,
            headers: self.headers,
            _state: PhantomData,
        };

        let results = result.results.pop().map(|result| {
            result.data.into_iter().map(|result| result.row).collect()
        }).unwrap_or(Vec::new());
        
        Ok((transaction, results))
    }
}

impl<'a> Transaction<'a, Started> {
    /// Executes the given `Statement`
    pub fn exec<T: Decodable>(&mut self, statement: Statement)
        -> Result<Vec<T>, GraphError>
    {
        let mut res = try!(super::send_query(&self.client,
                                             &self.transaction,
                                             self.headers,
                                             Some(statement.into())));

        let mut result: TransactionResult<T> = try!(super::parse_response(&mut res));

        let mut expires = result.transaction.expires.clone();
        let expires = try!(time::strptime(&mut expires, DATETIME_RFC822));

        self.expires = expires;

        let results = result.results.pop().map(|result| {
            result.data.into_iter().map(|result| result.row).collect()
        }).unwrap_or(Vec::new());

        Ok(results)
    }

    /// Commits the transaction, returning the results
    pub fn commit<T: Decodable>(self, statement: Option<Statement>)
        -> Result<Vec<T>, GraphError>
    {
        debug!("Commiting transaction {}", self.transaction);

        let statement = statement.map(|statement| statement.into());
        let mut res = try!(super::send_query(&self.client,
                                             &self.commit,
                                             self.headers,
                                             statement));

        let mut result: CommitResult<T> = try!(super::parse_response(&mut res));
        debug!("Transaction commited {}", self.transaction);

        let results = result.results.pop().map(|result| {
            result.data.into_iter().map(|result| result.row).collect()
        }).unwrap_or(Vec::new());

        Ok(results)
    }

    /// Rollback the transaction
    pub fn rollback(self) -> Result<(), GraphError> {
        debug!("Rolling back transaction {}", self.transaction);
        let req = self.client.delete(&self.transaction).headers(self.headers.clone());
        let mut res = try!(req.send());

        try!(super::parse_response::<(), CommitResult<()>>(&mut res));
        debug!("Transaction rolled back {}", self.transaction);

        Ok(())
    }

    /// Sends a query to just reset the transaction timeout
    ///
    /// All transactions have a timeout. Use this method to keep a transaction alive.
    pub fn reset_timeout(&mut self) -> Result<(), GraphError> {
        try!(self.exec::<()>("".into()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::header::{Authorization, Basic, ContentType, Headers};

    const URL: &'static str = "http://neo4j:neo4j@localhost:7474/db/data/transaction";

    fn get_headers() -> Headers {
        let mut headers = Headers::new();

        headers.set(Authorization(
            Basic {
                username: "neo4j".to_owned(),
                password: Some("neo4j".to_owned()),
            }
        ));

        headers.set(ContentType::json());

        headers
    }

    #[test]
    fn begin_transaction() {
        let headers = get_headers();
        let transaction = Transaction::new(URL, &headers);
        transaction.begin::<()>(None).unwrap();
    }

    #[test]
    fn create_node_and_commit() {
        let headers = get_headers();

        Transaction::new(URL, &headers)
            .begin::<()>(Some("CREATE (n:TEST_TRANSACTION_CREATE_COMMIT { name: 'Rust', safe: true })".into()))
            .unwrap()
            .0.commit::<()>(None)
            .unwrap();

        let (transaction, results) = Transaction::new(URL, &headers)
            .begin::<(String,)>(Some("MATCH (n:TEST_TRANSACTION_CREATE_COMMIT) RETURN n.name".into()))
            .unwrap();

        assert_eq!(results.len(), 1);

        transaction.rollback().unwrap();

        Transaction::new(URL, &headers)
            .begin::<()>(Some("MATCH (n:TEST_TRANSACTION_CREATE_COMMIT) DELETE n".into()))
            .unwrap()
            .0.commit::<()>(None)
            .unwrap();
    }

    #[test]
    fn create_node_and_rollback() {
        let headers = get_headers();

        let (mut transaction, _) = Transaction::new(URL, &headers)
            .begin::<()>(Some("CREATE (n:TEST_TRANSACTION_CREATE_ROLLBACK { name: 'Rust', safe: true })".into()))
            .unwrap();

        let results: Vec<(String, bool)> = transaction
            .exec("MATCH (n:TEST_TRANSACTION_CREATE_ROLLBACK) RETURN n.name, n.safe".into())
            .unwrap();

        assert_eq!(results.len(), 1);

        transaction.rollback().unwrap();

        let (transaction, results) = Transaction::new(URL, &headers)
            .begin::<()>(Some("MATCH (n:TEST_TRANSACTION_CREATE_ROLLBACK) RETURN n".into()))
            .unwrap();

        assert_eq!(results.len(), 0);

        transaction.rollback().unwrap();
    }

    #[test]
    fn query_open_transaction() {
        let headers = get_headers();

        let (mut transaction, _) = Transaction::new(URL, &headers).begin::<()>(None).unwrap();

        let results: Vec<(String, bool)> = transaction
            .exec("CREATE (n:TEST_TRANSACTION_QUERY_OPEN { name: 'Rust', safe: true }) \
                   RETURN n.name, n.safe".into())
            .unwrap();

        assert_eq!(results.len(), 1);

        transaction.rollback().unwrap();
    }
}
