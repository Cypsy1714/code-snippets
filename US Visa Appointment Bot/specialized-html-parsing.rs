// Specialized HTML parsing for appointment data extraction
// with localized date handling (Turkish dates)

use chrono::NaiveDate;
use scraper::{Html, Selector};
use std::collections::HashMap;
use super::super::structs::KonsoloslukSehir;

pub async fn get_earliest_date(
    user_payment_id: String,
    cookie_string: String,
    proxy: String,
) -> Result<((KonsoloslukSehir, NaiveDate), (KonsoloslukSehir, NaiveDate)), String> {
    // Get the HTML response using the API helper
    let res = api::usvisainfo_api::get_earliest_date(
        user_payment_id.clone(), 
        cookie_string.clone(), 
        proxy.clone()
    ).await.map_err(|e| format!(
        "usvisainfo.rs | get_earliest_date(user_payment_id={}, cookie_string={}) | Error occured when sending the api request. | {:?}", 
        user_payment_id.clone(), cookie_string.clone(), e
    ))?;

    // Extract the HTML text
    let res_http = res.text().await.map_err(|e| format!(
        "usvisainfo.rs | get_earliest_date(user_payment_id={}, cookie_string={}) | Error occured when getting the text from the api response. | {:?}", 
        user_payment_id.clone(), cookie_string.clone(), e
    ))?;

    // Parse the HTML to extract appointment information
    let appointments = parse_appointments(&res_http).map_err(|e| format!(
        "usvisainfo.rs | get_earliest_date(user_payment_id={}, cookie_string={}) | Error occured when parsing the appointments. | {:?}", 
        user_payment_id.clone(), cookie_string.clone(), e
    ))?;

    // Convert HashMap to Vec<(Konsolosluk, NaiveDate)>
    let vec: Vec<_> = appointments.into_iter().collect();

    // Convert Vec to tuple (ensuring we have exactly two appointments)
    let tuple = match vec.as_slice() {
        [(k1, v1), (k2, v2)] => ((k1.clone(), *v1), (k2.clone(), *v2)),
        _ => return Err(format!(
            "usvisainfo.rs | get_earliest_date(user_payment_id={}, cookie_string={}) | Error occured while more than 2 appointment dates were found. | vec={:?}", 
            user_payment_id.clone(), cookie_string.clone(), vec
        )),
    };

    Ok(tuple)
}

// Parse Turkish date strings into NaiveDate objects
fn parse_turkish_date(date_str: &str) -> Result<NaiveDate, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = date_str.split_whitespace().collect();
    if parts.len() != 3 {
        return Err("Invalid date format".into());
    }

    let day: u32 = parts[0].parse()?;
    
    // Map Turkish month names to their numerical values
    let month = match parts[1].trim_end_matches(',') {
        "Ocak" => 1,     // January
        "Şubat" => 2,    // February
        "Mart" => 3,     // March
        "Nisan" => 4,    // April
        "Mayıs" => 5,    // May
        "Haziran" => 6,  // June
        "Temmuz" => 7,   // July
        "Ağustos" => 8,  // August
        "Eylül" => 9,    // September
        "Ekim" => 10,    // October
        "Kasım" => 11,   // November
        "Aralık" => 12,  // December
        _ => return Err("Invalid month".into()),
    };
    
    let year: i32 = parts[2].parse()?;

    // Create the date, ensuring it's valid
    Ok(NaiveDate::from_ymd_opt(year, month, day).ok_or("Invalid date")?)
}

// Extract appointment information from HTML using CSS selectors
fn parse_appointments(
    html: &str,
) -> Result<HashMap<KonsoloslukSehir, NaiveDate>, Box<dyn std::error::Error>> {
    // Parse the HTML document
    let document = Html::parse_document(html);
    
    // Define CSS selectors for the elements containing appointment data
    let div_selector = Selector::parse("div.medium-3.column").unwrap();
    let table_selector = Selector::parse("table.for-layout").unwrap();
    let row_selector = Selector::parse("tr").unwrap();
    let cell_selector = Selector::parse("td").unwrap();

    // Store city -> date mappings
    let mut appointments = HashMap::new();

    // Find the div containing the appointment table
    if let Some(div) = document.select(&div_selector).next() {
        // Find the table within that div
        if let Some(table) = div.select(&table_selector).next() {
            // Process each row in the table
            for row in table.select(&row_selector) {
                let cells: Vec<_> = row.select(&cell_selector).collect();
                if cells.len() == 2 {
                    // Extract city and date from each row
                    let city = cells[0].text().next().unwrap_or("").trim().to_string();
                    let date_str = cells[1].text().next().unwrap_or("").trim();
                    
                    if !city.is_empty() && !date_str.is_empty() {
                        // Parse the city name and date
                        if let (Ok(date), Ok(city_enum)) =
                            (parse_turkish_date(date_str), parse_city(&city))
                        {
                            appointments.insert(city_enum, date);
                        }
                    }
                }
            }
        }
    }

    Ok(appointments)
}

// Convert city string to enum value
fn parse_city(city_string: &str) -> Result<KonsoloslukSehir, String> {
    match city_string {
        "Istanbul" => return Ok(KonsoloslukSehir::Istanbul),
        "Ankara" => return Ok(KonsoloslukSehir::Ankara),
        _ => {}
    }

    Err(format!("Given city string: {}", city_string))
}
