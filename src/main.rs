mod freq_chart;
use freq_chart::*;
mod line_chart;
use line_chart::*;
mod bar_chart;
use bar_chart::*;
mod date_temp_chart;
use date_temp_chart::*;
mod db_ops;
use db_ops::*;
mod gen_video;
use gen_video::*;
use colorize::AnsiColor;
//use execute::Execute;
use sqlx::{mysql::{MySqlPoolOptions, MySqlRow}, MySql, Pool, Row};
use dotenvy::dotenv;
use log::{error, warn, info, debug, trace, log_enabled, Level};
use env_logger;
use std::env;
use std::sync::OnceLock;
//use std::process::Command;

//use static for db connection pooling to create a single pool that can be shared across 
//multiple functions without having to pass it around as an argument. 

//The OnceLock ensures that the static values are only initialized once and are thread-safe.
static DB_POOL: OnceLock<Pool<MySql>> = OnceLock::new();
static DB_URL: OnceLock<String> = OnceLock::new();

// these are ffmpeg related consts
/*const FFMPEG_PATH: &str = "C:\\ProgramData\\chocolatey\\bin\\ffmpeg.exe";
const PNG_FOLDER: &str = "imgs\\";
const VIDEO_FOLDER: &str = "video\\";
const HIDE_BANNER: &str = "-hide_banner";
const START_NUMBER: &str = "-start_number";  //start_year = first_year
const FRAMERATE: &str = "-framerate";
const OVERWRITE: &str = "-y";
*/

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // initialize the database pool just once and share it across all functions. 
    //This is more efficient than creating a new pool for each function call. 
    //The pool will manage the connections and reuse them as needed.
    dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();  
    DB_URL.get_or_init(|| env::var("DATABASE_URL").expect("DATABASE_URL must be set"));
    DB_POOL.set(MySqlPoolOptions::new()
        .max_connections(5) // Set the maximum number of connections
        .connect(&DB_URL.get().expect("Database URL not initialized"))
        .await?)
        .expect("Failed to initialize database pool");

    let result = get_user_choice();
    if log_enabled!(Level::Trace) {
        trace!("Trace level is active (most detailed)");
        debug!("debug level is active (lots of details)");
        info!("info is active (std setting)");
        warn!("warning level is set (typical for production)");
        error!("only errors reported (for mature production only)");
    }

    Ok((result.await)?)
}

async fn get_user_choice() -> Result<(), sqlx::Error>{
    let choices = vec![
        "List all cities",
        "Generate Range Charts",
        "Generate BAR Charts by CITY",
        "Generate LINE Charts by CITY",
        "Generate Wide Date-Temp Charts by CITY",
        "Generate Videos from Charts",
        "Exit",
    ];

    loop {
        let prompt_message = "Please select a charting action".blue();
        let select = inquire::Select::new(&prompt_message, choices.clone())
            .prompt()
            .expect("Failed to select a database action");

        if select == "List all cities" {
            debug!("Listing all cities...");
            list_all_cities().await.expect("Failed to list all cities");
        } 
          else if select == "Generate Range Charts" {
            debug!("Generate Range Charts");
            generate_range_charts().await.expect("Failed to generate range charts");
        }       
          else if select == "Generate BAR Charts by CITY" {
            debug!("Generate BAR Charts by CITY");
            generate_bar_charts_by_city().await.expect("Failed to generate bar charts by city");
        }
          else if select == "Generate LINE Charts by CITY" {
            debug!("Generate LINE Charts by CITY");
            generate_line_charts_by_city().await.expect("Failed to generate line charts by city");
        }
          else if select == "Generate Wide Date-Temp Charts by CITY" {
            debug!("Generate Wide Date-Temp Charts by CITY");
            generate_date_temperature_charts_by_city().await.expect("Failed to generate line charts by city");
        }
          else if select == "Generate Videos from Charts" {
            debug!("Generate Videos from Charts");
            generate_videos_by_city().await.expect("Failed to generate videos from charts");
        }  
          else if select == "Exit" {
            info!("Exiting the program. Goodbye!");
            break;
        }
    }
    Ok(())
}
//    =======================================================================
async fn list_all_cities() -> Result<(), sqlx::Error> { 
    let city_list_result: Result<Vec<MySqlRow>, sqlx::Error> = fetch_cities().await;
    match city_list_result {
        Ok(_) => { //probably only returns Ok if it found something. otherwise it would return err, no empty check
            let city_list = city_list_result.unwrap();
            for a_city in city_list {
                let c_name: &str = a_city.get("name_of_city");
                info!("Available city: {c_name}");
            }
        },
        Err(e) => error!("Cities not found, {} ", e),
    }   
    info!("Listed names of the cities in city_names table in database: {:?}", DB_URL);
    Ok(())
}
async fn select_cities(message: String) -> Vec<String> {
    debug!("The select cities function is being run");
    let city_list_result: Result<Vec<MySqlRow>, sqlx::Error> = fetch_cities().await;
    let mut cities: Vec<String> = Vec::new();
    
    match city_list_result {
        Ok(_) => { //probably only returns Ok if it found something. otherwise it would return err, no empty check
            let city_list = city_list_result.unwrap();
            for a_city in city_list {
                let c_name: String = a_city.get("name_of_city");
                cities.push(c_name);
            }
        },
        Err(e) => error!("Cities not found, {} ", e),
    }   

    let prompt_message = message.green();
    let selected_cities = inquire::MultiSelect::new(&prompt_message, cities)
        .prompt()
        .expect("Failed to select cities");
    return selected_cities
}
async fn generate_range_charts() -> Result<(), sqlx::Error> {
    let selected_cities = select_cities("Please select the Cities to generate Range Charts".to_string()).await;

    for the_city in selected_cities {
        debug!("Selected RANGE chart for the city of {0}", the_city.clone().red());
        generate_one_city_range_charts(&the_city).await?;    
    }
    Ok(())
}
async fn generate_one_city_range_charts(the_city: &str) -> Result<(), sqlx::Error> {
    debug!("Generating one range chart: {0}", the_city);
    let city  = the_city;
    let first_year: i32 = get_first_year(&the_city).await;
    let last_year: i32 = get_last_year(&the_city).await;

    info!("Generating temp range frequency chart for {city} from {first_year} to {last_year}"); 
    let rows = get_temp_ranges(city).await?;
    let mut is_day: bool = true; // daytime chart - draw_freq_chart in module freq_chart
    draw_freq_chart(the_city, first_year, last_year, rows, is_day).expect("Failed to draw frequency chart");
    let nite_rows = get_nite_temp_ranges(city).await?;
    is_day = false; // nighttime chart - draw_freq_chart in module freq_chart
    draw_freq_chart(the_city, first_year, last_year, nite_rows, is_day).expect("Failed to draw frequency chart");
    Ok(())
}

async fn generate_bar_charts_by_city() -> Result<(), sqlx::Error> {
    let selected_cities = select_cities("Please select the cities to generate Bar Charts".to_string()).await;

    for the_city in selected_cities {
        info!("\nGenerating BAR charts for city of {0}", the_city.clone().red());
        generate_one_city_bar_charts(&the_city).await?;    
    }
    Ok(())
}
async fn generate_videos_by_city() -> Result<(), sqlx::Error> {
    let selected_cities = select_cities("Please select the cities to GENERATE videos".to_string()).await;
    for the_city in selected_cities {
        debug!("Generating videos for city of {0}", the_city.clone().red());
        generate_videos_from_charts(&the_city).await?;    
    }
    Ok(())
}
async fn generate_date_temperature_charts_by_city() -> Result<(), sqlx::Error> {
    let selected_cities = select_cities("Please select the cities to GENERATE date-time charts".to_string()).await;
    for the_city in selected_cities {
        info!("Generating date-time charts for city of {0}", the_city.clone().red());
        generate_date_temp_charts(&the_city).await.expect("Failed to generate date-time charts");    
    }
    Ok(())
}

async fn generate_line_charts_by_city() -> Result<(), sqlx::Error> {
    let selected_cities = select_cities("Please select the cities to GENERATE line charts".to_string()).await;
    for the_city in selected_cities {
        info!("Generating line charts for city of {0}", the_city.clone().red());
        generate_line_charts(&the_city).await?;    
    }
    Ok(())
}
async fn generate_line_charts(the_city: &str) -> Result<(), sqlx::Error> {
    let city  = the_city;
    let city_low: i32;
    let city_high: i32;   
    let fn_get_min_max: Result<(i32, i32), sqlx::Error> = get_city_min_max(city).await;
    match fn_get_min_max { // city_low & city_high here must be initialized in this block to make compiler happy
        Ok(_) => {let min_max: &(i32, i32) = &fn_get_min_max.unwrap();
                city_low = min_max.0;  city_high = min_max.1; /*println!("Low: {city_low}  High: {city_high}")*/},
        Err(e) =>  {city_low = 0; city_high = 0; error!("Error getting City min max: {}",e)}
    }  
    //let mut first_year: i32 = 0; 
    let first_year: i32 = get_first_year(city).await;
    let last_year: i32 = get_last_year(city).await;

    //I was going to make these run in parallel but svgs run so fast that it doesn't matter.
    // these fn's are in line_chart.rs
    make_monthly_charts(city, first_year, last_year, city_low, city_high).await?;
    make_fortly_charts(city, first_year, last_year, city_low, city_high).await?;
    make_weekly_charts(city, first_year, last_year, city_low, city_high).await?;
    Ok(())
}
