use std::fs::File;
use std::io::{BufWriter, Write as _};

use rusqlite::Connection;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CityClient {
    #[serde(rename = "id")]
    pub geoname_id: String,
    #[serde(rename = "n")]
    pub name: String,
    #[serde(rename = "cc")]
    pub country_code: String,
    #[serde(rename = "a1")]
    pub admin1_code: Option<String>,
    #[serde(rename = "tz")]
    pub timezone: String,
    #[serde(rename = "lat")]
    pub lat: f64,
    #[serde(rename = "lon")]
    pub lon: f64,
    #[serde(rename = "elv")]
    pub elevation: f64,
    #[serde(rename = "pop")]
    pub population: i64,
}

pub fn build_data(db_path: &str, output_path: &str) {
    let conn = Connection::open(db_path).unwrap();

    let mut stmt = conn
        .prepare(
            r#"
            SELECT 
                geoname_id, name,
                country_code, admin1_code, timezone,
                lat, lon, elevation_api, population
            FROM cities
            WHERE elevation_api IS NOT NULL
            ORDER BY population DESC
            "#,
        )
        .unwrap();

    let output_file = File::create(output_path).unwrap();
    let writer = BufWriter::new(output_file);
    let mut br_writer = brotli::CompressorWriter::new(writer, 4096, 11, 22);

    let mut count = 0u32;

    let rows = stmt
        .query_map([], |row| {
            Ok(CityClient {
                geoname_id: row.get(0)?,
                name: row.get(1)?,
                country_code: row.get(2)?,
                admin1_code: row.get(3)?,
                timezone: row.get(4)?,
                lat: row.get(5)?,
                lon: row.get(6)?,
                elevation: row.get(7)?,
                population: row.get(8)?,
            })
        })
        .unwrap();

    for city in rows {
        let city = city.unwrap();
        writeln!(br_writer, "{}", serde_json::to_string(&city).unwrap()).unwrap();
        count += 1;

        if count % 10000 == 0 {
            println!("Written {} cities...", count);
        }
    }

    br_writer.flush().unwrap();
    println!("Done! Written {} cities to {}", count, output_path);
}
