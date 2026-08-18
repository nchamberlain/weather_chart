use sqlx::mysql::MySqlRow;
use sqlx::Row;
use log::{error, warn, info, debug, trace, log_enabled, Level};
//use plotters::prelude::*;


// SQL query to retreive all city names in city_names table
pub async fn fetch_cities() -> Result<Vec<MySqlRow>, sqlx::Error> {
    debug!("Selected FETCH all Cities from city_names table");
    let query_string = format!("SELECT name_of_city FROM city_names ORDER by name_of_city asc;"); 
    let rows: Vec<sqlx::mysql::MySqlRow> = sqlx::query(&query_string)
        .fetch_all(crate::DB_POOL.get().expect("Database pool not initialized"))
        .await?; 

    if log_enabled!(Level::Trace) {
        trace!("Trace level is active (most detailed)");
        debug!("debug level is active (lots of details)");
        info!("info is active (std setting)");
        warn!("warning level is set (typical for production)");
        error!("only errors reported (for mature production only)");
    }

    Ok(rows)
}
pub async fn get_temps(tperiod: &str, city: &str, year: i32) -> Result<Vec<MySqlRow>, sqlx::Error> {
    let query_string = format!("SELECT tyear, {}, tmax, tmin, mmax, mmin FROM {} WHERE tyear = {}", tperiod, city, year ); // Adjust table name as needed
    let rows: Vec<sqlx::mysql::MySqlRow> = sqlx::query(&query_string)
        .fetch_all(crate::DB_POOL.get().expect("Database pool not initialized"))
        .await?; // had to make this function return a Result to use the ? operator
    Ok(rows)
}
pub async fn get_temp_ranges(city: &str) -> Result<Vec<sqlx::mysql::MySqlRow>, sqlx::Error> {
    let query_string = format!("SELECT * FROM {}_week_ranges", city); // Adjust table name as needed
    let rows: Vec<sqlx::mysql::MySqlRow> = sqlx::query(&query_string)
        .fetch_all(crate::DB_POOL.get().expect("Database pool not initialized"))
        .await?; // had to make this function return a Result to use the ? operator
    if rows.is_empty() {
        error!("No data found for city: {}", city);
        return Err(sqlx::Error::RowNotFound);
    }
    Ok(rows)
}
pub async fn get_nite_temp_ranges(city: &str) -> Result<Vec<sqlx::mysql::MySqlRow>, sqlx::Error> {
    let query_string = format!("SELECT * FROM {}_nite_ranges", city); // Adjust table name as needed
    let rows: Vec<sqlx::mysql::MySqlRow> = sqlx::query(&query_string)
        .fetch_all(crate::DB_POOL.get().expect("Database pool not initialized"))
        .await?; // had to make this function return a Result to use the ? operator
    if rows.is_empty() {
        error!("No data found for city: {}", city);
        return Err(sqlx::Error::RowNotFound);
    }
    Ok(rows)
}
pub async fn get_city_min_max(city: &str) -> Result<(i32, i32), sqlx::Error> {
    let query_string = format!("SELECT min_temp, max_temp FROM city_names WHERE name_of_city = '{}'", city); // Adjust table name as needed
    let rows: Vec<sqlx::mysql::MySqlRow> = sqlx::query(&query_string)
        .fetch_all(crate::DB_POOL.get().expect("Database pool not initialized"))
        .await?; // had to make this function return a Result to use the ? operator

    let lo: i32 = rows[0].get(0);
    let hi: i32 = rows[0].get(1);

    Ok((lo, hi))
}
pub async fn get_first_year(city: &str) -> i32{
    let query_stmt_string = format!("SELECT tdate FROM {city} order by tdate asc limit 1");
    let rows: Vec<sqlx::mysql::MySqlRow> = sqlx::query(&query_stmt_string)
        .fetch_all(crate::DB_POOL.get().expect("Database pool not initialized"))
        .await.expect("Failed to fetch first year");
    match rows.len() {
         0 => { error!("No data found for city: {}", city); return 0; },
         _ => {
            let first_year_row = &rows[0]; //unwrap the row
            let first_year_str: &str = first_year_row.get("tdate"); //get date string, for ex. 2020-09-05
            let first_year = first_year_str[0..4].parse().unwrap();  //parse first 4 digits as an int
            return first_year;
        }
    }
}
pub async fn get_last_year(city: &str) -> i32 {
    let query_stmt_string = format!("SELECT tdate FROM {city} order by tdate desc limit 1");
    let rows: Vec<sqlx::mysql::MySqlRow> = sqlx::query(&query_stmt_string)
        .fetch_all(crate::DB_POOL.get().expect("Database pool not initialized"))
        .await.expect("Failed to fetch last year");
    match rows.len() {
         0 => { error!("No data found for city: {}", city); return 0; },
         _ => {
            let last_year_row = &rows[0]; //unwrap the row
            let last_year_str: &str = last_year_row.get("tdate"); //get date string, for ex. 2020-11-21
            let last_year = last_year_str[0..4].parse().unwrap();  //parse first 4 digits as an int
            return last_year;
        }
    }
}
