use icu_calendar::{Iso, cal::Hebrew};
use jiff::{Timestamp, Zoned, civil::Date, tz::TimeZone};
use jiff_icu::ConvertInto as _;
use rust_zmanim::prelude::{ComplexZmanimCalendar, GeoLocation, UseElevation};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn get_current_time(timezone: String, timestamp_ms: i64) -> String {
    match TimeZone::get(&timezone) {
        Ok(tz) => {
            let ts = Timestamp::from_millisecond(timestamp_ms).unwrap();
            let zoned = ts.to_zoned(tz);
            zoned.strftime("%-I:%M %p").to_string()
        }
        Err(_) => "N/A".to_string(),
    }
}

#[wasm_bindgen]
pub fn dbg_zemanim(
    latitude: f64,
    longitude: f64,
    elevation: f64,
    timezone: String,
    location_name: String,
    admin1_code: Option<String>,
    country_code: String,
    date_str: Option<String>,
) -> String {
    let tz = TimeZone::get(&timezone).unwrap();
    let geo = GeoLocation {
        latitude,
        longitude,
        elevation,
        timezone: tz.clone(),
    };

    // Parse date or use today
    let date: Date = match date_str {
        Some(ref d) if !d.is_empty() => {
            Date::strptime("%Y-%m-%d", d).unwrap_or_else(|_| Zoned::now().date())
        }
        _ => Zoned::now().date(),
    };

    let calc = ComplexZmanimCalendar {
        date,
        geo_location: geo,
        use_elevation: UseElevation::All,
    };

    let alot = calc.alos_baal_hatanya();
    let misheyakir = calc.misheyakir_10_2_degrees();
    let hanetz = calc.hanetz();
    let sof_zman_shema = calc.sof_zman_shema_baal_hatanya();
    let sof_zman_tefila = calc.sof_zman_tefila_baal_hatanya();
    let chatzos = calc.chatzos();
    let mincha_gedola = calc.mincha_gedola_baal_hatanya();
    let mincha_ketana = calc.mincha_ketana_baal_hatanya();
    let plag = calc.plag_baal_hatanya();
    let shkia = calc.shkia();
    let tzeis = calc.tzeis_baal_hatanya();
    let chatzos_halayla = calc.chatzos_halayla();
    let shaah_zmanis = calc.shaah_zmanis_baal_hatanya();

    let format_time = |z: Option<Zoned>| -> String {
        match z {
            Some(t) => t.strftime("%-I:%M %p").to_string(),
            None => "N/A".to_string(),
        }
    };

    let shaah_zmanis_str = match shaah_zmanis {
        Some(d) => {
            let total_mins = d.as_secs_f64() / 60.0;
            let mins = total_mins.floor() as i64;
            let secs = ((total_mins - mins as f64) * 60.0).round() as i64;
            format!("{} min {} sec", mins, secs)
        }
        None => "N/A".to_string(),
    };

    let location_display = match admin1_code {
        Some(ref code) if !code.is_empty() => {
            format!("{}, {}, {}", location_name, code, country_code)
        }
        _ => format!("{}, {}", location_name, country_code),
    };

    // Format dates
    let gregorian_date = date.strftime("%B %d, %Y").to_string();

    let hebrew_date = he_date(date);
    let hebrew_str = format_hebrew_date(&hebrew_date);

    let output = format!(
        r#"<div class="zmanim-results">
<h3>{}</h3>
<div class="date-info">{}</div>
<div class="date-info">{}</div>
<div class="location-info">Timezone: {} | Lat: {:.4}, Lon: {:.4}, Elev: {:.0}m</div>
<table>
<tr><td class="time">{}</td><td class="label">Dawn (Alot Hashachar)</td></tr>
<tr><td class="time">{}</td><td class="label">Earliest Tallit and Tefillin (Misheyakir)</td></tr>
<tr><td class="time highlight">{}</td><td class="label">Sunrise (Hanetz Hachamah)</td></tr>
<tr><td class="time">{}</td><td class="label">Latest Shema</td></tr>
<tr><td class="time">{}</td><td class="label">Latest Shacharit</td></tr>
<tr><td class="time">{}</td><td class="label">Midday (Chatzot Hayom)</td></tr>
<tr><td class="time">{}</td><td class="label">Earliest Mincha (Mincha Gedolah)</td></tr>
<tr><td class="time">{}</td><td class="label">Mincha Ketanah ("Small Mincha")</td></tr>
<tr><td class="time">{}</td><td class="label">Plag Hamincha ("Half of Mincha")</td></tr>
<tr><td class="time highlight">{}</td><td class="label">Sunset (Shkiah)</td></tr>
<tr><td class="time">{}</td><td class="label">Nightfall (Tzeit Hakochavim)</td></tr>
<tr><td class="time">{}</td><td class="label">Midnight (Chatzot HaLailah)</td></tr>
</table>
<div class="shaah-zmanis">Shaah Zmanit (proportional hour): {}</div>
</div>"#,
        location_display,
        gregorian_date,
        hebrew_str,
        timezone,
        latitude,
        longitude,
        elevation,
        format_time(alot),
        format_time(misheyakir),
        format_time(hanetz),
        format_time(sof_zman_shema),
        format_time(sof_zman_tefila),
        format_time(chatzos),
        format_time(mincha_gedola),
        format_time(mincha_ketana),
        format_time(plag),
        format_time(shkia),
        format_time(tzeis),
        format_time(chatzos_halayla),
        shaah_zmanis_str,
    );

    output
}

fn he_date(date: jiff::civil::Date) -> icu_calendar::Date<Hebrew> {
    let icu_date: icu_calendar::Date<Iso> = date.convert_into();
    icu_date.to_calendar(Hebrew)
}

fn format_hebrew_date(hebrew_date: &icu_calendar::Date<Hebrew>) -> String {
    let year = hebrew_date.year().extended_year();
    let day = hebrew_date.day_of_month().0;
    let month = hebrew_date.month();

    let month_num = month.number();

    // Check if the year is a leap year
    let is_leap_year = hebrew_date.is_in_leap_year();

    // ICU Hebrew calendar uses civil year numbering (Tishrei = month 1)
    // In a REGULAR year (12 months):
    //   1=Tishrei, 2=Cheshvan, 3=Kislev, 4=Tevet, 5=Shevat, 6=Adar,
    //   7=Nissan, 8=Iyar, 9=Sivan, 10=Tammuz, 11=Av, 12=Elul
    // In a LEAP year (13 months):
    //   1=Tishrei, 2=Cheshvan, 3=Kislev, 4=Tevet, 5=Shevat, 6=Adar I, 7=Adar II,
    //   8=Nissan, 9=Iyar, 10=Sivan, 11=Tammuz, 12=Av, 13=Elul

    let month_name = match month_num {
        1 => "Tishrei",
        2 => "Cheshvan",
        3 => "Kislev",
        4 => "Tevet",
        5 => "Shevat",
        6 if is_leap_year => "Adar I",
        7 if is_leap_year => "Adar II",
        6 => "Adar",
        m @ 7..=12 if !is_leap_year => match m {
            7 => "Nissan",
            8 => "Iyar",
            9 => "Sivan",
            10 => "Tammuz",
            11 => "Av",
            12 => "Elul",
            _ => "Unknown",
        },
        m @ 8..=13 if is_leap_year => match m {
            8 => "Nissan",
            9 => "Iyar",
            10 => "Sivan",
            11 => "Tammuz",
            12 => "Av",
            13 => "Elul",
            _ => "Unknown",
        },
        _ => "Unknown",
    };

    format!("{} {}, {}", day, month_name, year)
}
