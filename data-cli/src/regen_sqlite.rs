use std::collections::HashMap;
use std::io::BufRead as _;
use std::{fs::File, io::BufReader};

use rusqlite::Connection;
use serde::Deserialize;

pub fn regenerate_db(output_path: &str) {
    let conn = Connection::open(output_path).unwrap();
    create_tables(&conn);
    load_admin1_codes(&conn);
    load_cities(&conn);
    print_stats(&conn);
}

fn create_tables(conn: &Connection) {
    conn.execute_batch(
        r#"
        DROP TABLE IF EXISTS cities;
        DROP TABLE IF EXISTS admin1_codes;
        
        CREATE TABLE cities (
            geoname_id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            ascii_name TEXT NOT NULL,
            alternate_names TEXT,
            feature_class TEXT,
            feature_code TEXT,
            country_code TEXT,
            cou_name_en TEXT,
            country_code_2 TEXT,
            admin1_code TEXT,
            admin2_code TEXT,
            admin3_code INTEGER,
            admin4_code TEXT,
            lat REAL NOT NULL,
            lon REAL NOT NULL,
            elevation_orig TEXT,
            dem INTEGER,
            elevation_api REAL,
            population INTEGER,
            timezone TEXT NOT NULL,
            modification_date TEXT,
            label_en TEXT
        );
        
        CREATE TABLE admin1_codes (
            country_code TEXT NOT NULL,
            admin1_code TEXT NOT NULL,
            name TEXT NOT NULL,
            ascii_name TEXT NOT NULL,
            geoname_id TEXT,
            PRIMARY KEY (country_code, admin1_code)
        );
        
        CREATE INDEX idx_cities_name ON cities(name);
        CREATE INDEX idx_cities_country ON cities(country_code);
        CREATE INDEX idx_cities_coords ON cities(lat, lon);
        CREATE INDEX idx_cities_timezone ON cities(timezone);
        "#,
    )
    .unwrap();
}

fn load_admin1_codes(conn: &Connection) {
    let input_file = File::open("data/admin1CodesASCII.txt").unwrap();
    let reader = BufReader::new(input_file);

    let tx = conn.unchecked_transaction().unwrap();

    let mut stmt = tx
        .prepare_cached(
            "INSERT INTO admin1_codes (country_code, admin1_code, name, ascii_name, geoname_id) VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .unwrap();

    let mut loaded = 0u32;
    let mut parse_errors = 0u32;

    for (line_num, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Line {}: Failed to read: {}", line_num + 1, e);
                parse_errors += 1;
                continue;
            }
        };

        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 4 {
            eprintln!("Line {}: Invalid format (expected 4 fields)", line_num + 1);
            parse_errors += 1;
            continue;
        }

        let concatenated = parts[0];
        let split_pos = match concatenated.find('.') {
            Some(p) => p,
            None => {
                eprintln!(
                    "Line {}: Invalid concatenated code: {}",
                    line_num + 1,
                    concatenated
                );
                parse_errors += 1;
                continue;
            }
        };

        let country_code = &concatenated[..split_pos];
        let admin1_code = &concatenated[split_pos + 1..];
        let name = parts[1];
        let ascii_name = parts[2];
        let geoname_id = parts.get(3).copied().unwrap_or("");

        stmt.execute(rusqlite::params![
            country_code,
            admin1_code,
            name,
            ascii_name,
            if geoname_id.is_empty() {
                None
            } else {
                Some(geoname_id)
            },
        ])
        .unwrap();

        loaded += 1;
    }

    drop(stmt);
    tx.commit().unwrap();

    println!("Admin1 codes loaded: {} ({} errors)", loaded, parse_errors);
}

fn load_cities(conn: &Connection) {
    let coords_to_elevation: HashMap<String, Option<f64>> =
        serde_json::from_str(&std::fs::read_to_string("data/coords_to_elevation.json").unwrap())
            .unwrap();

    let input_file = File::open("data/all-cities.jsonl").unwrap();
    let reader = BufReader::new(input_file);

    let tx = conn.unchecked_transaction().unwrap();

    let mut stmt = tx
        .prepare_cached(
            r#"
            INSERT INTO cities (
                geoname_id, name, ascii_name, alternate_names,
                feature_class, feature_code,
                country_code, cou_name_en, country_code_2,
                admin1_code, admin2_code, admin3_code, admin4_code,
                lat, lon, elevation_orig, dem, elevation_api,
                population, timezone, modification_date, label_en
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)
            "#,
        )
        .unwrap();

    let mut processed = 0u32;
    let mut parse_errors = 0u32;

    for (line_num, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Line {}: Failed to read: {}", line_num + 1, e);
                parse_errors += 1;
                continue;
            }
        };

        let geoname: GeoName = match serde_json::from_str(&line) {
            Ok(g) => g,
            Err(e) => {
                eprintln!("Line {}: Failed to parse: {}", line_num + 1, e);
                parse_errors += 1;
                continue;
            }
        };

        let coord_key = format!("{},{}", geoname.coordinates.lat, geoname.coordinates.lon);
        let elevation_api = coords_to_elevation.get(&coord_key).and_then(|v| *v);

        let alternate_names_json = geoname
            .alternate_names
            .map(|names| serde_json::to_string(&names).unwrap())
            .unwrap_or_default();

        stmt.execute(rusqlite::params![
            geoname.geoname_id,
            geoname.name,
            geoname.ascii_name,
            alternate_names_json,
            geoname.feature_class,
            geoname.feature_code,
            geoname.country_code,
            geoname.cou_name_en,
            geoname.country_code_2,
            geoname.admin1_code,
            geoname.admin2_code,
            geoname.admin3_code,
            geoname.admin4_code,
            geoname.coordinates.lat,
            geoname.coordinates.lon,
            geoname.elevation,
            geoname.dem,
            elevation_api,
            geoname.population,
            geoname.timezone,
            geoname.modification_date,
            geoname.label_en,
        ])
        .unwrap();

        processed += 1;

        if processed % 10000 == 0 {
            println!("Processed {} rows...", processed);
        }
    }

    drop(stmt);
    tx.commit().unwrap();

    println!("Cities loaded: {} ({} errors)", processed, parse_errors);
}

fn print_stats(conn: &Connection) {
    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM cities", [], |row| row.get(0))
        .unwrap();
    let with_elevation: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM cities WHERE elevation_api IS NOT NULL",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let admin1_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM admin1_codes", [], |row| row.get(0))
        .unwrap();

    println!("\n=== Database Statistics ===");
    println!("Cities: {}", total);
    println!("  With elevation_api: {}", with_elevation);
    println!("  Without elevation_api: {}", total - with_elevation);
    println!("Admin1 codes: {}", admin1_count);
}

#[derive(Debug, Clone, Deserialize)]
pub struct GeoName {
    pub geoname_id: String,
    pub name: String,
    pub ascii_name: String,
    pub alternate_names: Option<Vec<String>>,
    pub feature_class: Option<String>,
    pub feature_code: Option<String>,
    pub country_code: Option<String>,
    pub cou_name_en: Option<String>,
    pub country_code_2: Option<String>,
    pub admin1_code: Option<String>,
    pub admin2_code: Option<String>,
    pub admin3_code: Option<i64>,
    pub admin4_code: Option<String>,
    pub population: i64,
    pub elevation: Option<String>,
    pub dem: i32,
    pub timezone: String,
    pub modification_date: String,
    pub label_en: Option<String>,
    pub coordinates: Coordinates,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Coordinates {
    pub lon: f64,
    pub lat: f64,
}
