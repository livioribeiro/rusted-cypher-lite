use std::ops::Deref;
use rustc_serialize::Decodable;

use ::error::Neo4jError;

pub trait ResultTrait<T: Decodable> {
    fn results(&self) -> &Vec<CypherResult<T>>;
    fn errors(&self) -> &Vec<Neo4jError>;
}

#[derive(Debug, PartialEq, RustcDecodable)]
pub struct QueryResult<T: Decodable> {
    pub results: Vec<CypherResult<T>>,
    errors: Vec<Neo4jError>,
}

impl<T: Decodable> ResultTrait<T> for QueryResult<T> {
    fn results(&self) -> &Vec<CypherResult<T>> {
        &self.results
    }

    fn errors(&self) -> &Vec<Neo4jError> {
        &self.errors
    }
}

/// Holds the result of a cypher query
#[derive(Clone, Debug, PartialEq, RustcDecodable)]
pub struct CypherResult<T: Decodable> {
    columns: Vec<String>,
    data: Vec<RowResult<T>>,
}

impl<T: Decodable> CypherResult<T> {
    /// Returns an iterator over the rows of the result
    pub fn rows(&self) -> &Vec<RowResult<T>> {
        &self.data
    }
}

/// Holds a single row of the result of a cypher query
#[derive(Clone, Debug, PartialEq, RustcDecodable)]
pub struct RowResult<T: Decodable> {
    row: T,
}

impl<T: Decodable> RowResult<T> {
    pub fn data(&self) -> &T {
        &self.row
    }
}

impl<T: Decodable> Deref for RowResult<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.row
    }
}

// #[cfg(test)]
// mod tests {
//     use std::collections::BTreeMap;
//     use serde_json::value as json_value;
//     use super::*;
//
//     #[derive(Clone, RustcEncodable)]
//     struct Person {
//         name: String,
//         lastname: String,
//     }
//
//     fn make_result() -> CypherResult {
//         let node = Person {
//             name: "Test".to_owned(),
//             lastname: "Result".to_owned(),
//         };
//
//         let node = json_value::to_value(&node);
//         let row_data = vec![node];
//
//         let row1 = RowResult { row: row_data.clone() };
//         let row2 = RowResult { row: row_data.clone() };
//
//         let data = vec![row1, row2];
//         let columns = vec!["node".to_owned()];
//
//         CypherResult {
//             columns: columns,
//             data: data,
//         }
//     }
//
//     #[test]
//     fn rows() {
//         let result = make_result();
//         for row in result.rows() {
//             let row = row.get::<BTreeMap<String, String>>("node");
//             assert!(row.is_ok());
//
//             let row = row.unwrap();
//             assert_eq!(row.get("name").unwrap(), "Test");
//             assert_eq!(row.get("lastname").unwrap(), "Result");
//         }
//     }
//
//     #[test]
//     #[should_panic(expected = "No such column")]
//     fn no_column_name_in_row() {
//         let result = make_result();
//         let rows: Vec<Row> = result.rows().collect();
//         let ref row = rows[0];
//         row.get::<String>("nonexistent").unwrap();
//     }
//
//     #[test]
//     #[should_panic(expected = "No column at index")]
//     fn no_column_index_in_row() {
//         let result = make_result();
//         let rows: Vec<Row> = result.rows().collect();
//         let ref row = rows[0];
//         row.get_n::<String>(99).unwrap();
//     }
// }
