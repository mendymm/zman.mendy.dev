use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::Write;

use rusqlite::Connection;

pub fn build_admin1(db_path: &str, output_path: &str) {
    let conn = Connection::open(db_path).unwrap();

    let mut stmt = conn
        .prepare(
            r#"
            SELECT DISTINCT c.country_code, c.admin1_code
            FROM cities c
            WHERE c.elevation_api IS NOT NULL
            AND c.admin1_code IS NOT NULL
            "#,
        )
        .unwrap();

    let used_codes: BTreeSet<(String, String)> = stmt
        .query_map([], |row| {
            let cc: String = row.get(0)?;
            let a1: String = row.get(1)?;
            Ok((cc, a1))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    let total_used = used_codes.len();

    let mut admin1_stmt = conn
        .prepare(
            r#"
            SELECT country_code, admin1_code, ascii_name
            FROM admin1_codes
            "#,
        )
        .unwrap();

    let admin1_names: BTreeMap<(String, String), String> = admin1_stmt
        .query_map([], |row| {
            let cc: String = row.get(0)?;
            let a1: String = row.get(1)?;
            let name: String = row.get(2)?;
            Ok((cc, a1, name))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .filter(|(cc, a1, _)| used_codes.contains(&((*cc).clone(), (*a1).clone())))
        .map(|(cc, a1, name)| ((cc, a1), name))
        .collect();

    let found_count = admin1_names.len();
    let excluded_count = total_used - found_count;

    let mut result: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    for ((cc, a1), name) in admin1_names {
        result.entry(cc).or_default().insert(a1, name);
    }

    let file = File::create(output_path).unwrap();
    let mut br_writer = brotli::CompressorWriter::new(file, 4096, 11, 22);
    serde_json::to_writer(&mut br_writer, &result).unwrap();
    br_writer.flush().unwrap();
    br_writer.into_inner().flush().unwrap();

    println!("Admin1 codes written to {}", output_path);
    println!("  Used location codes: {}", total_used);
    println!("  Found names: {}", found_count);
    println!("  Excluded (no name found): {}", excluded_count);
}
