use std::collections::BTreeMap;
use std::convert::From;
use std::error::Error;
use rustc_serialize::{Encodable};
use rustc_serialize::json::{Json, ToJson};

/// Helper macro to simplify the creation of complex statements
///
/// Pass in the statement text as the first argument followed by the (optional) parameters, which
/// must be in the format `"param" => value` and wrapped in `{}`
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate rusted_cypher;
/// # fn main() {
/// // Without parameters
/// let statement = cypher_stmt!("MATCH n RETURN n");
/// // With parameters
/// let statement = cypher_stmt!("MATCH n RETURN n" {
///     "param1" => "value1".to_owned(),
///     "param2" => 2,
///     "param3" => 3.0
/// });
/// # }
/// ```
#[macro_export]
macro_rules! cypher_stmt {
    ( $s:expr ) => { $crate::Statement::new($s) };
    ( $s:expr { $( $k:expr => $v:expr ),+ } ) => {
        $crate::Statement::new($s)
            $(.with_param($k, $v))*
    }
}

/// Represents a statement to be sent to the server
#[derive(Clone, Debug, RustcEncodable)]
pub struct Statement {
    statement: String,
    parameters: BTreeMap<String, Json>,
}

impl Statement  {
    pub fn new(statement: &str) -> Self {
        Statement {
            statement: statement.to_owned(),
            parameters: BTreeMap::new(),
        }
    }

    /// Returns the statement text
    pub fn statement(&self) -> &str {
        &self.statement
    }

    /// Adds parameter in builder style
    ///
    /// This method consumes `self` and returns it with the parameter added, so the binding does
    /// not need to be mutable
    ///
    /// # Examples
    ///
    /// ```
    /// # use rusted_cypher::Statement;
    /// let statement = Statement::new("MATCH n RETURN n")
    ///     .with_param("param1", "value1".to_owned())
    ///     .with_param("param2", 2)
    ///     .with_param("param3", 3.0);
    /// ```
    pub fn with_param<V: ToJson>(mut self, key: &str, value: V) -> Self {
        self.add_param(key, value);
        self
    }

    /// Adds parameter to the `Statement`
    pub fn add_param<V: ToJson>(&mut self, key: &str, value: V) {
        self.parameters.insert(key.to_owned(), value.to_json());
    }

    /// Gets the value of the parameter
    ///
    /// Returns `None` if there is no parameter with the given name
    pub fn param(&self, key: &str) -> Option<&Json> {
        self.parameters.get(key)
    }

    /// Gets a reference to the underlying parameters `BTreeMap`
    pub fn parameters(&self) -> &BTreeMap<String, Json> {
        &self.parameters
    }

    /// Sets the parameters `BTreeMap`, overriding current values
    pub fn set_parameters(&mut self, params: BTreeMap<String, Json>) {
        self.parameters = params;
    }

    /// Removes parameter from the statment
    ///
    /// Trying to remove a non-existent parameter has no effect
    pub fn remove_param(&mut self, key: &str) {
        self.parameters.remove(key);
    }
}

impl<'a> From<&'a str> for Statement {
    fn from(stmt: &str) -> Self {
        Statement::new(stmt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(unused_variables)]
    fn from_str() {
        let stmt: Statement = "MATCH n RETURN n".into();
    }

    #[test]
    fn with_param() {
        let statement = Statement::new("MATCH n RETURN n")
            .with_param("param1", "value1".to_owned())
            .with_param("param2", 2)
            .with_param("param3", 3.0)
            .with_param("param4", vec![0; 4]);

        assert_eq!(statement.parameters().len(), 4);
    }

    #[test]
    fn add_param() {
        let mut statement = Statement::new("MATCH n RETURN n");
        statement.add_param("param1", "value1".to_owned());
        statement.add_param("param2", 2);
        statement.add_param("param3", 3.0);
        statement.add_param("param4", vec![0; 4]);

        assert_eq!(statement.parameters().len(), 4);
    }

    #[test]
    fn remove_param() {
        let mut statement = Statement::new("MATCH n RETURN n")
            .with_param("param1", "value1".to_owned())
            .with_param("param2", 2)
            .with_param("param3", 3.0)
            .with_param("param4", vec![0; 4]);

        statement.remove_param("param1");

        assert_eq!(statement.parameters().len(), 3);
    }

    #[test]
    #[allow(unused_variables)]
    fn macro_without_params() {
        let stmt = cypher_stmt!("MATCH n RETURN n");
    }

    #[test]
    fn macro_single_param() {
        let statement1 = cypher_stmt!("MATCH n RETURN n" {
            "name" => "test".to_owned()
        });

        let param = 1;
        let statement2 = cypher_stmt!("MATCH n RETURN n" {
            "value" => param
        });

        assert_eq!("test", statement1.param("name").unwrap().as_string().unwrap());
        assert_eq!(param, statement2.param("value").unwrap().as_i64().unwrap());
    }

    #[test]
    fn macro_multiple_params() {
        let param = 3f64;
        let statement = cypher_stmt!("MATCH n RETURN n" {
            "param1" => "one".to_owned(),
            "param2" => 2,
            "param3" => param
        });

        assert_eq!("one", statement.param("param1").unwrap().as_string().unwrap());
        assert_eq!(2, statement.param("param2").unwrap().as_i64().unwrap());
        assert_eq!(param, statement.param("param3").unwrap().as_f64().unwrap());
    }
}
