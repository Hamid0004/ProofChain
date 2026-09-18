use rusqlite::Connection;

fn main() {
    let conn = Connection::open("/tmp/test_init2.db").unwrap();
    
    // Enable WAL mode - PRAGMA statements return results, need execute_batch or query_row
    let result = conn.execute("PRAGMA journal_mode=WAL", []);
    println!("WAL mode result: {:?}", result);
    
    // Try with execute_batch instead
    conn.execute_batch("PRAGMA journal_mode=WAL;").unwrap();
    println!("WAL mode enabled with execute_batch");
}
