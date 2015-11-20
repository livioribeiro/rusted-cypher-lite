extern crate rusted_cypher;

use rusted_cypher::GraphClient;

fn main() {
    let client = GraphClient::connect("http://neo4j:neo4j@localhost:7474/db/data").unwrap();

    println!("{:?}", client.neo4j_version());
}
