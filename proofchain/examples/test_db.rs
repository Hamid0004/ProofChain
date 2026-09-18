use rusqlite::{Connection, params};

fn main() {
    let conn = Connection::open("/tmp/test_proof.db").unwrap();
    
    // Try execute_batch
    let result = conn.execute_batch("
        CREATE TABLE IF NOT EXISTS blocks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            height INTEGER UNIQUE NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_height ON blocks(height);
    ");
    println!("execute_batch result: {:?}", result);
    
    // Try individual executes
    let result2 = conn.execute(
        "CREATE TABLE IF NOT EXISTS blocks2 (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            height INTEGER UNIQUE NOT NULL
        )",
        [],
    );
    println!("execute result: {:?}", result2);
}
