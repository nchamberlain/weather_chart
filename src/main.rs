mod freq_chart;
use freq_chart::draw_freq_chart;
use colorize::AnsiColor;
use execute::Execute;
use sqlx::{mysql::{MySqlPoolOptions, MySqlRow}, MySql, Pool, Row};
use dotenvy::dotenv;
use log::{error, warn, info, debug, trace, log_enabled, Level};
use env_logger;
use std::env;
use std::sync::OnceLock;
use std::process::Command;
use std::io::{self, Write};
use std::collections::HashMap;
use plotters::{prelude::*, style::RGBColor};
use plotters::coord::Shift;
use chrono::{NaiveDate, Months};

//use static for db connection pooling to create a single pool that can be shared across 
//multiple functions without having to pass it around as an argument. 

//The OnceLock ensures that the pool is only initialized once and is thread-safe.
static DB_POOL: OnceLock<Pool<MySql>> = OnceLock::new();
static DB_URL: OnceLock<String> = OnceLock::new();

// constants used when generating charts
const TOP_MARGIN: i32 = 60;
const BOTTOM_MARGIN: i32 = 40;
const LEFT_MARGIN: i32 = 70;
const RIGHT_MARGIN: i32 = 40;
const DWG_WIDTH: i32 = 1280; //this is overall size of image
const DWG_HEIGHT: i32 = 800; // approximately golden ratio
const AXIS_WIDTH: i32 = DWG_WIDTH - LEFT_MARGIN - RIGHT_MARGIN;
const AXIS_HEIGHT: i32 = DWG_HEIGHT - TOP_MARGIN - BOTTOM_MARGIN;
const H_TICK_WIDTH: i32 = AXIS_WIDTH / 4;
const V_TICK_HEIGHT: i32 = AXIS_HEIGHT / 10;
const TOP_LINE_Y: i32 = 0 + TOP_MARGIN; //x height of top line of chart, might NOT = TOP_MARGIN
const BOTTOM_LINE_Y: i32 = TOP_LINE_Y + AXIS_HEIGHT;

// these are ffmpeg related consts
const FFMPEG_PATH: &str = "C:\\ProgramData\\chocolatey\\bin\\ffmpeg.exe";
const PNG_FOLDER: &str = "imgs\\";
const VIDEO_FOLDER: &str = "video\\";
const HIDE_BANNER: &str = "-hide_banner";
const START_NUMBER: &str = "-start_number";  //start_year = first_year
const FRAMERATE: &str = "-framerate";
const OVERWRITE: &str = "-y";

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
            generate_date_time_charts_by_city().await.expect("Failed to generate line charts by city");
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
    let city_list_result: Result<Vec<MySqlRow>, sqlx::Error> = list_cities().await;
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
async fn list_cities() -> Result<Vec<MySqlRow>, sqlx::Error> {
    debug!("Selected List All Cities in city_names table");
    let query_string = format!("SELECT name_of_city FROM city_names ORDER by name_of_city asc;"); 
    let rows: Vec<sqlx::mysql::MySqlRow> = sqlx::query(&query_string)
        .fetch_all(DB_POOL.get().expect("Database pool not initialized"))
        .await?; 
    Ok(rows)
}
async fn select_cities(message: String) -> Vec<String> {
    debug!("The select cities function is being run");
    let city_list_result: Result<Vec<MySqlRow>, sqlx::Error> = list_cities().await;
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
        generate_city_range_charts(&the_city).await?;    
    }
    Ok(())
}
async fn generate_city_range_charts(the_city: &str) -> Result<(), sqlx::Error> {
    debug!("Generating one range chart: {0}", the_city);
    let city  = the_city;
    let first_year: i32 = get_1st_year(&the_city).await;
    let last_year: i32 = get_end_year(&the_city).await;

    info!("Generating temp range frequency chart for {city} from {first_year} to {last_year}"); 
    let rows = get_temp_ranges(city).await?;
    draw_freq_chart(the_city, first_year, last_year, rows).expect("Failed to draw frequency chart");
    //let rows = get_low_temp_ranges(city).await?;
    //draw_freq_chart(the_city, first_year, last_year, rows).expect("Failed to draw frequency chart");

    Ok(())
}

async fn generate_bar_charts_by_city() -> Result<(), sqlx::Error> {
    let selected_cities = select_cities("Please select the cities to generate Bar Charts".to_string()).await;

    for the_city in selected_cities {
        info!("\nGenerating BAR charts for city of {0}", the_city.clone().red());
        generate_city_bar_charts(&the_city).await?;    
    }
    Ok(())
}
async fn generate_city_bar_charts(the_city: &str) -> Result<(), sqlx::Error> {
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
    let first_year: i32 = get_1st_year(&the_city).await;
    let last_year: i32 = get_end_year(&the_city).await;

    info!("City: {city} from {first_year} to {last_year}"); 
    // calc these here so available to the functions
    let y_lowest = city_low - 0; // can adjust chart min temp here 
    let y_highest = city_high + 0; //can adjust chart max temp
    let y_range =  y_highest - y_lowest; //neg y_lowest increases y_range
    let pixel_per_degree: f64 = f64::from(AXIS_HEIGHT) / f64::from(y_range);
    let zero_line_offset: f64; // this is the amount to adjust the bar lengths to account for negative temps. It is the distance from 0 degrees to the bottom line of the chart, in pixels. It is added to the bar length to adjust for negative temps. It is a positive number that represents how many pixels above the bottom line the 0 degree line is.
    if y_lowest < 0  { 
        zero_line_offset = (f64::from(y_lowest) * pixel_per_degree).abs(); //
    } else if y_lowest == 0 {
        zero_line_offset = 0.0;
    } else {
        let z_diff = 0 - y_lowest -1; //makes a negative value
        zero_line_offset = f64::from(z_diff) * pixel_per_degree;
    }

    debug!("Axis Height: {AXIS_HEIGHT} Y range: {y_range} degrees. Pixels per degree: {pixel_per_degree}. Zero offset: {zero_line_offset}");
 
    let mperiod = "Month"; // periods can be Month, Week, or Fort
    let wperiod = "Week"; 
    let fperiod = "Fort"; 
    let mcity_period = format!("{city}_{mperiod}");
    let wcity_period = format!("{city}_{wperiod}");
    let fcity_period = format!("{city}_{fperiod}");
    let tmperiod = "tmonth"; // column names in selected db: can be tmonth, tfort, or tweek
    let twperiod = "tweek"; 
    let tfperiod = "tfort"; 

    let title_style = ("sans-serif", 36).into_font().color(&BLACK);
    let x_axis_style = ("sans-serif", 14).into_font().color(&BLACK);
    let y_axis_style = ("sans-serif", 18).into_font().color(&BLACK);

    for the_year in first_year..=last_year {
        print!("{the_year},");
        io::stdout().flush().unwrap(); // force flush now
        let mfile_name = format!("bar_charts/{city}_{the_year}_month.svg");
        let wfile_name = format!("bar_charts/{city}_{the_year}_week.svg");
        let ffile_name = format!("bar_charts/{city}_{the_year}_fort.svg");

        let mtitle_text = format!("{the_year} {city}  Monthly Avg Temperatures");
        let wtitle_text = format!("{the_year} {city}  Weekly Avg Temperatures");
        let ftitle_text = format!("{the_year} {city}  Fortnightly Avg Temperatures");
       // ------ monthly chart ------------------------------------------------------
        let mdwg = SVGBackend::new(&mfile_name, (DWG_WIDTH as u32, DWG_HEIGHT as u32)).into_drawing_area();
        mdwg.fill(&WHITE).expect("Failed to fill dwg"); //this automatically makes a rectangle size of drawing area and fills it with white

        // Draw axis lines on the drawing area
        draw_axes(&mdwg).expect("Failed to draw axes");
        
        // Draw horizontal and verticlal grid lines with tick marks
        draw_grids(&mdwg).expect("Failed to draw grids");

        // Draw legend
        draw_legend(&mdwg).expect("Failed to draw legend");

        // Draw title
        draw_title(&mdwg, &mtitle_text, title_style.clone()).expect("Failed to draw title");

        // Draw axis labels
        draw_axis_labels(&mdwg, x_axis_style.clone(), y_axis_style.clone(), mperiod, y_lowest, y_highest, y_range).expect("Failed to draw axis labels");

        let fn_result: Result<Vec<sqlx::mysql::MySqlRow>, sqlx::Error> = get_temps(tmperiod, &mcity_period, the_year).await;
        match fn_result {
            Ok(_) => { 
                //print_avgs(period, &city_period, the_year, &fn_result.as_ref().unwrap());
                draw_hi_temps(&mdwg, mperiod, zero_line_offset, pixel_per_degree, &fn_result.as_ref().unwrap()).expect("Draw Hi Temps Failed"); 
                draw_low_temps(&mdwg, mperiod, zero_line_offset, pixel_per_degree, &fn_result.as_ref().unwrap()).expect("Draw Low Temps Failed");
            }
            Err(e) => error!("Error getting temperatures from db: {}", e),
        }
        mdwg.present().expect("Failed monthly Chart drawing");
        // ----------- wweekly chart ------------------------------------------------------
        let wdwg = SVGBackend::new(&wfile_name, (DWG_WIDTH as u32, DWG_HEIGHT as u32)).into_drawing_area();
        wdwg.fill(&WHITE).expect("Failed to fill dwg"); //this automatically makes a rectangle size of drawing area and fills it with white

        // Draw axis lines on the drawing area
        draw_axes(&wdwg).expect("Failed to draw axes");
        
        // Draw horizontal and verticlal grid lines with tick marks
        draw_grids(&wdwg).expect("Failed to draw grids");

        // Draw legend
        draw_legend(&wdwg).expect("Failed to draw legend");

        // Draw title
        draw_title(&wdwg, &wtitle_text, title_style.clone()).expect("Failed to draw title");

        // Draw axis labels
        draw_axis_labels(&wdwg, x_axis_style.clone(), y_axis_style.clone(), wperiod, y_lowest, y_highest, y_range).expect("Failed to draw axis labels");

        let fn_result: Result<Vec<sqlx::mysql::MySqlRow>, sqlx::Error> = get_temps(twperiod, &wcity_period, the_year).await;
        match fn_result {
            Ok(_) => { 
                //print_avgs(period, &city_period, the_year, &fn_result.as_ref().unwrap());
                draw_hi_temps(&wdwg, wperiod, zero_line_offset, pixel_per_degree, &fn_result.as_ref().unwrap()).expect("Draw Hi Temps Failed"); 
                draw_low_temps(&wdwg, wperiod, zero_line_offset, pixel_per_degree, &fn_result.as_ref().unwrap()).expect("Draw Low Temps Failed");
            }
            Err(e) => error!("Error getting temperatures from db: {}", e),
        }
        wdwg.present().expect("Failed weekly Chart drawing");
        // ----------- fortnightly chart ------------------------------------------------------
        let fdwg = SVGBackend::new(&ffile_name, (DWG_WIDTH as u32, DWG_HEIGHT as u32)).into_drawing_area();
        fdwg.fill(&WHITE).expect("Failed to fill dwg"); //this automatically makes a rectangle size of drawing area and fills it with white

        // Draw axis lines on the drawing area
        draw_axes(&fdwg).expect("Failed to draw axes");
        
        // Draw horizontal and verticlal grid lines with tick marks
        draw_grids(&fdwg).expect("Failed to draw grids");

        // Draw legend
        draw_legend(&fdwg).expect("Failed to draw legend");

        // Draw title
        draw_title(&fdwg, &ftitle_text, title_style.clone()).expect("Failed to draw title");

        // Draw axis labels
        draw_axis_labels(&fdwg, x_axis_style.clone(), y_axis_style.clone(), fperiod, y_lowest, y_highest, y_range).expect("Failed to draw axis labels");

        let fn_result: Result<Vec<sqlx::mysql::MySqlRow>, sqlx::Error> = get_temps(tfperiod, &fcity_period, the_year).await;
        match fn_result {
            Ok(_) => { 
                //print_avgs(period, &city_period, the_year, &fn_result.as_ref().unwrap());
                draw_hi_temps(&fdwg, fperiod, zero_line_offset, pixel_per_degree, &fn_result.as_ref().unwrap()).expect("Draw Hi Temps Failed"); 
                draw_low_temps(&fdwg, fperiod, zero_line_offset, pixel_per_degree, &fn_result.as_ref().unwrap()).expect("Draw Low Temps Failed");
            }
            Err(e) => error!("Error getting temperatures from db: {}", e),
        }
        fdwg.present().expect("Failed fortnight Chart drawing");
    }
    Ok(())
}
fn draw_legend(dwg: &DrawingArea<SVGBackend, Shift>) -> Result<(), Box<dyn std::error::Error>> {
    let legend_x = LEFT_MARGIN + AXIS_WIDTH - 150;
    let legend_y = TOP_MARGIN + 12;
    let rect_size = 20;
    let spacing = 30;

    let temperature_colors: Vec<(RGBColor, RGBColor, &str)> = vec![
        (get_cool_colors(110), get_warm_colors(110), "100+°F low hi"),
        (get_cool_colors(90), get_warm_colors(90), "81-100°F low hi"),
        (get_cool_colors(70), get_warm_colors(70), "61-80°F low hi"),
        (get_cool_colors(50), get_warm_colors(50), "33-60°F low hi"),
        (get_cool_colors(30), get_warm_colors(30), "<= 32°F low hi"),
    ];

    // Draw legend background
    dwg.draw(&Rectangle::new(
        [(legend_x - 10, legend_y - 10), (legend_x + rect_size + 128, legend_y + (spacing * temperature_colors.len() as i32))],
        Into::<ShapeStyle>::into(&WHITE).filled(),
    ))?;
    // Draw box around legend
    dwg.draw(&Rectangle::new(
        [(legend_x - 10, legend_y - 10), (legend_x + rect_size + 128, legend_y + (spacing * temperature_colors.len() as i32))],
        Into::<ShapeStyle>::into(&BLACK).stroke_width(2),
    ))?;
    //draw the legend boxes and labels
    for (i, (colorc,color, label)) in temperature_colors.iter().enumerate() {
        let y_offset = i as i32 * spacing;
        dwg.draw(&Rectangle::new(
            [(legend_x, legend_y + y_offset), (legend_x + rect_size, legend_y + rect_size + y_offset)],
            Into::<ShapeStyle>::into(colorc).filled(),
        ))?;
        dwg.draw(&Rectangle::new(
            [(legend_x + rect_size +5, legend_y + y_offset), (legend_x + rect_size + rect_size + 5, legend_y + rect_size + y_offset)],
            Into::<ShapeStyle>::into(color).filled(),
        ))?;
        dwg.draw_text(label, &("sans-serif", 16).into_font().color(&BLACK), (legend_x + rect_size  + rect_size + 15, legend_y + rect_size / 2 + y_offset-3))?;
    }
    Ok(())
}
fn draw_hi_temps(dwg: &DrawingArea<SVGBackend, Shift>, period: &str, z_line_offset: f64,  pixel_per_degree: f64, rows: &Vec<MySqlRow>) -> Result<(), Box<dyn std::error::Error>> {
    let mut y_adj: i32;
    match period {
        "Week" => {    
            for i in 1..53 {
                let x = i * (AXIS_WIDTH / 52) + LEFT_MARGIN;
                let idx: usize = i.try_into().unwrap();
                let tmp: i32; // get ready to hold the hi_temp to display
                let hi_result = rows[idx-1].try_get("tmax");
                match hi_result {
                    Ok(_) => { 
                        tmp = hi_result.unwrap(); 
                        if tmp >200 { // filter out null data 
                            continue
                        }
                    } //set tmp to hi_temp
                    Err(_) => { continue; }
                }
                let y: f64 = f64::from(tmp) * pixel_per_degree; //calc how tall this line should be
                if z_line_offset <= 0.0 { // negative offsets are temps above 0 degrees F
                    y_adj = ((y + z_line_offset) + pixel_per_degree).round() as i32;                   
                } else {
                    y_adj = (y + z_line_offset).round() as i32;
                }
                let custom_color = get_warm_colors(tmp);
                dwg.draw(&Rectangle::new(
                    [(x, BOTTOM_LINE_Y - 2), (x+8, BOTTOM_LINE_Y - y_adj)],
                    Into::<ShapeStyle>::into(&custom_color).filled(),
                ))?
            }   
        },
        "Fort" => {    
            for i in 1..27 {
                let x = i * (AXIS_WIDTH / 26) + LEFT_MARGIN - 16;//-16 is a fundge factor to position bars correctly
                let idx: usize = i.try_into().unwrap();
                let tmp: i32;
                let hi_result = rows[idx-1].try_get("tmax");
                match hi_result {
                    Ok(_) => { 
                        tmp = hi_result.unwrap();
                        if tmp >200 { // filter out null data 
                            continue
                        }
                    }
                    Err(_) => { continue; }
                }
                let y: f64 = f64::from(tmp) * pixel_per_degree;
                if z_line_offset <= 0.0 { // negative offsets are temps above 0 degrees F
                    y_adj = ((y + z_line_offset) + pixel_per_degree).round() as i32;                   
                } else {
                    y_adj = (y + z_line_offset).round() as i32;
                }
                let custom_color = get_warm_colors(tmp);
                dwg.draw(&Rectangle::new(
                    [(x, BOTTOM_LINE_Y - 2), (x+18, BOTTOM_LINE_Y - y_adj)],
                    Into::<ShapeStyle>::into(&custom_color).filled(),
                ))?
            }   
        },
        "Month" => {
            for i in 1..13 {
                let x = i * (AXIS_WIDTH / 12) + LEFT_MARGIN - 50; //-50 is a fundge factor to position bars correctly
                let idx: usize = i.try_into().unwrap();
                let tmp: i32;
                let hi_result = rows[idx-1].try_get("tmax");
                match hi_result {
                    Ok(_) => { 
                        tmp = hi_result.unwrap();
                        if tmp >200 { // filter out null data 
                            continue
                        }
                    }
                    Err(_) => { continue; }
                }
                let y: f64 = f64::from(tmp) * pixel_per_degree;
                if z_line_offset <= 0.0 { // negative offsets are temps above 0 degrees F
                    y_adj = ((y + z_line_offset) + pixel_per_degree).round() as i32;                   
                } else {
                    y_adj = (y + z_line_offset).round() as i32;
                }
                let custom_color = get_warm_colors(tmp);
                dwg.draw(&Rectangle::new(
                    [(x, BOTTOM_LINE_Y - 2), (x+30, BOTTOM_LINE_Y - y_adj)], //2nd y, bigger number = shorter bars
                    Into::<ShapeStyle>::into(&custom_color).filled(),
                ))?;
            }
        },
        _ => error!("Unknown Period"),
    }
    Ok(())
}
fn get_warm_colors(temp: i32) -> RGBColor {
    if temp <= 32 { // DodgerBlue
        RGBColor(30, 144, 255) 
    } else if temp > 32 && temp <= 60 { // Tan
        RGBColor(210, 180, 140) 
    } else if temp > 60 && temp <= 80 { // MediumSpringGreen
        RGBColor(0, 250, 154) 
    } else if temp > 80 && temp <= 100 { // Orange
        RGBColor(255, 165, 0)
    } else {
        RGBColor(255, 0, 0) // Red
    }
}
fn draw_low_temps(dwg: &DrawingArea<SVGBackend, Shift>, period: &str, z_line_offset: f64, pixel_per_degree: f64, rows: &Vec<MySqlRow>) -> Result<(), Box<dyn std::error::Error>>  {
    let mut y_adj: i32;
    match period {
        "Week" => {
            for i in 1..53 {
                let x = i * (AXIS_WIDTH / 52) +  LEFT_MARGIN;
                let idx: usize = i.try_into().unwrap();
                //let tmp: i32 = rows[idx-1].get("tmin");
                let tmp: i32;
                let low_result = rows[idx-1].try_get("tmin");
                match low_result {
                    Ok(_) =>  { 
                        tmp = low_result.unwrap();
                        if tmp >200 { // filter out null data 
                            continue
                        }
                    }
                    Err(_) => { continue; }
                }
                let y: f64 = f64::from(tmp) * pixel_per_degree;
                if z_line_offset <= 0.0 { // negative offsets are temps above 0 degrees F
                    y_adj = ((y + z_line_offset) + pixel_per_degree).round() as i32;                   
                } else {
                    y_adj = (y + z_line_offset).round() as i32;
                }
                let custom_color = get_cool_colors(tmp);
                dwg.draw(&Rectangle::new(
                    [(x, BOTTOM_LINE_Y - 2), (x+8, BOTTOM_LINE_Y - y_adj)],
                    Into::<ShapeStyle>::into(&custom_color).filled(),
                ))?
            }
        },
        "Fort" => {
            for i in 1..27 {
                let x = i * (AXIS_WIDTH / 26) +  LEFT_MARGIN - 16;
                let idx: usize = i.try_into().unwrap();
                // let tmp: i32 = rows[idx-1].get("tmin");
                let tmp: i32;
                let low_result = rows[idx-1].try_get("tmin");
                match low_result {
                    Ok(_) =>  { 
                        tmp = low_result.unwrap(); 
                        if tmp >200 { // filter out null data 
                            continue
                        }
                    }
                    Err(_) => { continue; }
                }
                let y: f64 = f64::from(tmp) * pixel_per_degree;
                if z_line_offset <= 0.0 { // negative offsets are temps above 0 degrees F
                    y_adj = ((y + z_line_offset) + pixel_per_degree).round() as i32;                   
                } else {
                    y_adj = (y + z_line_offset).round() as i32;
                }
                let custom_color = get_cool_colors(tmp);
                dwg.draw(&Rectangle::new(
                    [(x, BOTTOM_LINE_Y - 2), (x+18, BOTTOM_LINE_Y - y_adj)], //2nd y, bigger number = shorter bars
                    Into::<ShapeStyle>::into(&custom_color).filled(),
                ))?
            }
        },
        "Month" => {
            for i in 1..13 {
                let x = i * (AXIS_WIDTH / 12) + LEFT_MARGIN - 50;
                let idx: usize = i.try_into().unwrap();
                let tmp: i32;
                let low_result = rows[idx-1].try_get("tmin");
                match low_result {
                    Ok(_) =>  { 
                        tmp = low_result.unwrap(); 
                        if tmp >200 { // filter out null data 
                            continue
                        }
                    }
                    Err(_) => { continue; }
                }
                let y: f64 = f64::from(tmp) * pixel_per_degree;
                if z_line_offset <= 0.0 { // negative offsets are temps above 0 degrees F
                    y_adj = ((y + z_line_offset) + pixel_per_degree).round() as i32;                   
                } else {
                    y_adj = (y + z_line_offset).round() as i32;
                }
                let custom_color = get_cool_colors(tmp);
                dwg.draw(&Rectangle::new(
                    [(x, BOTTOM_LINE_Y - 2), (x+30, BOTTOM_LINE_Y - y_adj)], //2nd y, bigger number = shorter bars
                    Into::<ShapeStyle>::into(&custom_color).filled(),
                ))?;
            }
        },
        _ => error!("Unknown Period"),
    }
    Ok(())    
}
fn get_cool_colors(temp: i32) -> RGBColor {
    if temp <= 32 { //DarkBlue 
        RGBColor(0,0,139) 
    } else if temp > 32 && temp <= 60 { // SaddleBrown
        RGBColor(139, 69, 19) 
    } else if temp > 60 && temp <= 80 { // ForestGreen
        RGBColor(34, 139, 34) 
    } else if temp > 80 && temp <= 100 { // Burnt Orange
        RGBColor(204,85,0) 
    } else {
        RGBColor(139,0,0)  // Dark Red
    }
}
fn draw_axes(dwg: &DrawingArea<SVGBackend, Shift>) -> Result<(), Box<dyn std::error::Error>> {
    // Draw axis lines on the drawing area
    dwg.draw(&PathElement::new( //draw y axis
        vec![(LEFT_MARGIN, TOP_MARGIN), (LEFT_MARGIN, AXIS_HEIGHT + TOP_MARGIN)],
        Into::<ShapeStyle>::into(&BLACK).stroke_width(5),
    ))?;    
    dwg.draw(&PathElement::new( //draw x axis
        vec![(LEFT_MARGIN-2, AXIS_HEIGHT + TOP_MARGIN), (AXIS_WIDTH + LEFT_MARGIN, AXIS_HEIGHT + TOP_MARGIN)],
        Into::<ShapeStyle>::into(&BLACK).stroke_width(5),
    ))?;  
    Ok(())  
}
fn draw_grids(dwg: &DrawingArea<SVGBackend, Shift>) -> Result<(), Box<dyn std::error::Error>> {
    // Draw 4 vertical grid lines
    for i in 1..5 { 
        let x = LEFT_MARGIN + i * H_TICK_WIDTH;
        dwg.draw(&PathElement::new(  //draw vertical grid line
            vec![(x, TOP_MARGIN), (x, AXIS_HEIGHT + TOP_MARGIN-2)],
            Into::<ShapeStyle>::into(RGBColor(128, 128, 128)).stroke_width(1),
        ))?;
        dwg.draw(&PathElement::new(  //draw tick mark on x axis
            vec![(x, AXIS_HEIGHT + TOP_MARGIN ), (x, AXIS_HEIGHT + TOP_MARGIN+10)],
            Into::<ShapeStyle>::into(&BLACK).stroke_width(3),
        ))?;
    }
    // Draw 10 horizontal grid lines
    for i in 0..10 {
        let y = TOP_MARGIN + i * V_TICK_HEIGHT;
        dwg.draw(&PathElement::new(
            vec![(LEFT_MARGIN+2, y), (AXIS_WIDTH + LEFT_MARGIN, y)],
            Into::<ShapeStyle>::into(RGBColor(128, 128, 128)).stroke_width(1),
        ))?;
        dwg.draw(&PathElement::new(  //draw tick mark on y axis
            vec![(LEFT_MARGIN -10, y), (LEFT_MARGIN , y)],
            Into::<ShapeStyle>::into(&BLACK).stroke_width(3),
        ))?;
        if i <= 9 {
            let v_tick_4 = V_TICK_HEIGHT / 4;
            let y_minor1 = y + v_tick_4;
            let y_minor2 = y + (v_tick_4 * 2);
            let y_minor3 = y + (v_tick_4 * 3);
            dwg.draw(&PathElement::new(
                vec![(LEFT_MARGIN+2, y_minor1), (AXIS_WIDTH + LEFT_MARGIN, y_minor1)],
                Into::<ShapeStyle>::into(RGBColor(128, 128, 128)).stroke_width(1),
            ))?;
            dwg.draw(&PathElement::new(
                vec![(LEFT_MARGIN+2, y_minor2), (AXIS_WIDTH + LEFT_MARGIN, y_minor2)],
                Into::<ShapeStyle>::into(RGBColor(128, 128, 128)).stroke_width(1),
            ))?;
            dwg.draw(&PathElement::new(
                vec![(LEFT_MARGIN+2, y_minor3), (AXIS_WIDTH + LEFT_MARGIN, y_minor3)],
                Into::<ShapeStyle>::into(RGBColor(128, 128, 128)).stroke_width(1),
            ))?;
        }
    }
    Ok(())
}
fn draw_title(dwg: &DrawingArea<SVGBackend, Shift>, title_text: &str, title_style: TextStyle) -> Result<(), Box<dyn std::error::Error>> {
    let (title_width, title_height) = dwg.estimate_text_size(&title_text, &title_style)?;
    
    dwg.draw_text(&title_text, &title_style,
        ((DWG_WIDTH / 2) as i32 - (title_width as i32 / 2), title_height as i32 - 10),
    )?; 
    Ok(())
}
fn draw_axis_labels(dwg: &DrawingArea<SVGBackend, Shift>,
                         x_axis_style: TextStyle, 
                         y_axis_style: TextStyle, 
                         period: &str,
                         _y_lowest: i32,
                         y_highest: i32,
                         y_range: i32) -> Result<(), Box<dyn std::error::Error>> {
    match period {
        "Week" => {
            let (_x_label_width, x_label_height) = dwg.estimate_text_size(&format!("55"), &x_axis_style)?;
            debug!("x_label_width: {}, x_label_height: {}", _x_label_width, x_label_height);
            for i in 1..53 {
                let x = i * (AXIS_WIDTH / 52) + LEFT_MARGIN;
                let i_str = i.to_string();
                dwg.draw_text(&i_str, &x_axis_style, (x - 1, AXIS_HEIGHT + TOP_MARGIN + (x_label_height / 2) as i32 + 5))?;
            }
        },
        "Fort" => {
            let (_x_label_width, x_label_height) = dwg.estimate_text_size(&format!("55"), &x_axis_style)?;
            debug!("x_label_width: {}, x_label_height: {}", _x_label_width, x_label_height);
            for i in 1..27 {
                let x = i * (AXIS_WIDTH / 26) + LEFT_MARGIN - 15;
                let i_str = i.to_string();
                dwg.draw_text(&i_str, &x_axis_style, (x - 1, AXIS_HEIGHT + TOP_MARGIN + (x_label_height / 2) as i32 + 1))?;
            }
        },
        "Month" =>  {   
            for i   in 1..13 {
                let month_abbr = match i {
                    1 => "Jan",
                    2 => "Feb",
                    3 => "Mar",
                    4 => "Apr",
                    5 => "May",
                    6 => "Jun",
                    7 => "Jul",
                    8 => "Aug",
                    9 => "Sep",
                    10 => "Oct",
                    11 => "Nov",
                    12 => "Dec",
                    _ => "",
                };
                let x = i * (AXIS_WIDTH / 12) + LEFT_MARGIN - 45;
                dwg.draw_text(&month_abbr, &x_axis_style, (x, AXIS_HEIGHT + TOP_MARGIN + 10))?;
            }
        },
        _ => error!("Unknown Period"),
    }

    // Draw Y Axis Label
    let (y_label_width, y_label_height) = dwg.estimate_text_size(&format!("{}", y_highest), &y_axis_style)?;
    let temp: f64 = y_range as f64 / 10.0;
    //let tenth_range: i32 = temp.round() as i32; // amount to adjust for each horizontal grid line
    let tenth_range = temp; // amount to adjust for each horizontal grid line
    for i in 0..10 {
        let y = f64::from(TOP_MARGIN) + (f64::from(i) * f64::from(AXIS_HEIGHT)) / 10.0;
        let i_str = format!("{:.1}", (f64::from(y_highest) - (tenth_range * f64::from(i))));
        dwg.draw_text(&i_str, &y_axis_style, (LEFT_MARGIN - 24 - y_label_width as i32, y.round() as i32 - (y_label_height/2) as i32))?;
    }
    Ok(())
}
async fn get_city_min_max(city: &str) -> Result<(i32, i32), sqlx::Error> {
    let query_string = format!("SELECT min_temp, max_temp FROM city_names WHERE name_of_city = '{}'", city); // Adjust table name as needed
    let rows: Vec<sqlx::mysql::MySqlRow> = sqlx::query(&query_string)
        .fetch_all(DB_POOL.get().expect("Database pool not initialized"))
        .await?; // had to make this function return a Result to use the ? operator

    let lo: i32 = rows[0].get(0);
    let hi: i32 = rows[0].get(1);

    Ok((lo, hi))
}
async fn get_temp_ranges(city: &str) -> Result<Vec<sqlx::mysql::MySqlRow>, sqlx::Error> {
    let query_string = format!("SELECT * FROM {}_week_ranges", city); // Adjust table name as needed
    let rows: Vec<sqlx::mysql::MySqlRow> = sqlx::query(&query_string)
        .fetch_all(DB_POOL.get().expect("Database pool not initialized"))
        .await?; // had to make this function return a Result to use the ? operator
    if rows.is_empty() {
        error!("No data found for city: {}", city);
        return Err(sqlx::Error::RowNotFound);
    }
    if log_enabled!(Level::Debug) {
        let first_row = &rows[0];
        let yr: i32 = first_row.get("tyear");
        let frz: i32 = first_row.get("count_freezing");
        let c30s: i32 = first_row.get("count_30s");
        let c40s: i32 = first_row.get("count_40s");
        let c50s: i32 = first_row.get("count_50s");
        let c60s: i32 = first_row.get("count_60s");
        let c70s: i32 = first_row.get("count_70s");
        let c80s: i32 = first_row.get("count_80s");
        let c90s: i32 = first_row.get("count_90s");
        let c100s: i32 = first_row.get("count_100s");
        info!("Temp freqs for {city} year {yr}: {frz}, {c30s}, {c40s}, {c50s}, {c60s}, {c70s}, {c80s}, {c90s}, {c100s}");
    }

    Ok(rows)
}
async fn get_temps(tperiod: &str, city: &str, year: i32) -> Result<Vec<MySqlRow>, sqlx::Error> {
    let query_string = format!("SELECT tyear, {}, tmax, tmin, mmax, mmin FROM {} WHERE tyear = {}", tperiod, city, year ); // Adjust table name as needed
    let rows: Vec<sqlx::mysql::MySqlRow> = sqlx::query(&query_string)
        .fetch_all(DB_POOL.get().expect("Database pool not initialized"))
        .await?; // had to make this function return a Result to use the ? operator
    Ok(rows)
}
async fn get_1st_year(city: &str) -> i32{
    let query_stmt_string = format!("SELECT tdate FROM {city} order by tdate asc limit 1");
    let rows: Vec<sqlx::mysql::MySqlRow> = sqlx::query(&query_stmt_string)
        .fetch_all(DB_POOL.get().expect("Database pool not initialized"))
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
async fn get_end_year(city: &str) -> i32 {
    let query_stmt_string = format!("SELECT tdate FROM {city} order by tdate desc limit 1");
    let rows: Vec<sqlx::mysql::MySqlRow> = sqlx::query(&query_stmt_string)
        .fetch_all(DB_POOL.get().expect("Database pool not initialized"))
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
async fn generate_videos_by_city() -> Result<(), sqlx::Error> {
    let selected_cities = select_cities("Please select the cities to GENERATE videos".to_string()).await;
    for the_city in selected_cities {
        debug!("Generating videos for city of {0}", the_city.clone().red());
        generate_videos_from_charts(&the_city).await?;    
    }
    Ok(())
}
async fn generate_videos_from_charts(the_city: &str) -> Result<(), sqlx::Error> {
    let first_year: i32 = get_1st_year(&the_city).await;
    info!("Generating videos for the city of {0} starting in {1}", the_city, first_year);
    
    let mut fps = 1;
    let _ = generate_videos(the_city, first_year, fps);
    fps = 2;
    let _ = generate_videos(the_city, first_year, fps);
    fps = 4;
    let _ = generate_videos(the_city, first_year, fps);
    fps = 8;
    let _ = generate_videos(the_city, first_year, fps);
    fps = 16;
    let _ = generate_videos(the_city, first_year, fps);
   
    Ok(())
}
fn generate_videos(city: &str, start_year: i32, fps: i32) -> Result<(), Box<dyn std::error::Error>> {
    let mut period = "month";
    let frames_per_second = fps.to_string();
    let mut cmd = Command::new(FFMPEG_PATH);
    cmd.arg(HIDE_BANNER).arg(OVERWRITE).arg(START_NUMBER).arg(start_year.to_string())
    .arg(FRAMERATE).arg(&frames_per_second);
    let input_arg = format!("{}{}_%4d_{}.png", PNG_FOLDER, city, period);
    cmd.arg("-i").arg(&input_arg).arg("-c:v").arg("libx264");
    let mut output_arg = format!("{}{}_{}_{}.mp4", VIDEO_FOLDER, city, period, &frames_per_second);
    cmd.arg(output_arg);

    info!("Executing command: {:?}", cmd);
    let _ = cmd.execute_output().unwrap();

    period = "week";
    //let frames_per_second = fps.to_string();
    let mut cmd2 = Command::new(FFMPEG_PATH);
    cmd2.arg(HIDE_BANNER).arg(OVERWRITE).arg(START_NUMBER).arg(start_year.to_string())
    .arg(FRAMERATE).arg(&frames_per_second);
    let input_arg = format!("{}{}_%4d_{}.png", PNG_FOLDER, city, period);
    cmd2.arg("-i").arg(&input_arg).arg("-c:v").arg("libx264");
    output_arg = format!("{}{}_{}_{}.mp4", VIDEO_FOLDER, city, period, &frames_per_second);
    cmd2.arg(&output_arg);

    info!("Executing command: {:?}", cmd2);
    let _ = cmd2.execute_output().unwrap();

    period = "fort";
    //let frames_per_second = fps.to_string();
    let mut cmd3 = Command::new(FFMPEG_PATH);
    cmd3.arg(HIDE_BANNER).arg(OVERWRITE).arg(START_NUMBER).arg(start_year.to_string())
    .arg(FRAMERATE).arg(&frames_per_second);
    let input_arg = format!("{}{}_%4d_{}.png", PNG_FOLDER, city, period);
    cmd3.arg("-i").arg(&input_arg).arg("-c:v").arg("libx264");
    output_arg = format!("{}{}_{}_{}.mp4", VIDEO_FOLDER, city, period, &frames_per_second);
    cmd3.arg(&output_arg);

    info!("Executing command: {:?}", cmd3);
    let output = cmd3.execute_output().unwrap();

    if let Some(exit_code) = output.status.code() {
        if exit_code == 0 {
            info!("The weekly exit code is 0 (Ok)");
        } else {
            error!("Error executing `{}` with in-file: {} and out-file: {}", FFMPEG_PATH, input_arg, output_arg);
        }
    } 

    Ok(())
}
async fn generate_date_time_charts_by_city()  -> Result<(), sqlx::Error> {
    let selected_cities = select_cities("Please select the cities to GENERATE date-time charts".to_string()).await;
    for the_city in selected_cities {
        info!("Generating date-time charts for city of {0}", the_city.clone().red());
        generate_date_time_charts(&the_city).await.expect("Failed to generate date-time charts");    
    }
    Ok(())
}
async fn generate_date_time_charts(the_city: &str) -> Result<(), sqlx::Error> {
    let city  = String::from(the_city);
    for chart_type in 1..=4 { //1=high-Max, 2=hig-min, 3=low-max, 4=low-min
        make_date_time_charts(&city, chart_type).await?;    
    }
    Ok(())
}
async fn make_date_time_charts(city: &str, chart_type: i32) -> Result<(), sqlx::Error> {
    let mut chart_title = String::new();
    let mut out_file_name = String::new();
    let mut query_string = String::new();//format!("SELECT tdate, tmax, tmin FROM {} ORDER BY tdate", city);
    let query_limit = 10000;
    //let city_fmt: String = city.replace("_", " ");
    match chart_type {
        1 => {debug!("Generating High-Max charts for {city}");
                chart_title = format!("{}: {} Hottest Daytime Temperatures", city.replace("_", " "), query_limit);
                query_string = format!("SELECT tdate, tmax FROM {} where tmax is not null ORDER BY tmax DESC, tdate ASC LIMIT {}", city, query_limit);
                out_file_name = format!("date_charts/{}_high_max.svg", city);
            },
        2 => {debug!("Generating High-Min charts for {city}");
                chart_title = format!("{}: {} Coolest Daytime Temperatures", city.replace("_", " "), query_limit);
                query_string = format!("SELECT tdate, tmax FROM {} where tmax is not null ORDER BY tmax ASC, tdate ASC LIMIT {}", city, query_limit);
                out_file_name = format!("date_charts/{}_high_min.svg", city);
            },
        3 => {debug!("Generating Low-Max charts for {city}");
                chart_title = format!("{}: {} Coldest Nighttime Temperatures", city.replace("_", " "), query_limit);
                query_string = format!("SELECT tdate, tmin FROM {} where tmin is not null ORDER BY tmin ASC, tdate ASC LIMIT {}", city, query_limit);
                out_file_name = format!("date_charts/{}_low_max.svg", city);
            },
        4 => {debug!("Generating Low-Min charts for {city}");
                chart_title = format!("{}: {} Warmest Nighttime Temperatures", city.replace("_", " "), query_limit);
                query_string = format!("SELECT tdate, tmin FROM {} where tmin is not null ORDER BY tmin DESC, tdate ASC LIMIT {}", city, query_limit);
                out_file_name = format!("date_charts/{}_low_min.svg", city);
            },
        _ => error!("Unknown chart type"),
    }
    let rows: Vec<sqlx::mysql::MySqlRow> = sqlx::query(&query_string)
        .fetch_all(DB_POOL.get().expect("Database pool not initialized"))
        .await?;
    let dup_dates: HashMap<&str, (f64, i32)> = find_dup_dates(&rows); //use this hashmap later

    // find highest & lowest Temps from rows to set the y axis range
    let highest_temp: f64= rows.iter().map(|row| row.get::<i32, _>(1)).max().unwrap_or(0) as f64 + 1.0;
    let lowest_temp: f64 = rows.iter().map(|row| row.get::<i32, _>(1)).min().unwrap_or(0) as f64 - 1.0;
    // find the earliest and latest dates to set x axis range
    let earliest_date_str = rows.iter().map(|row| row.get::<&str, _>(0)).min().unwrap_or("1800-01-01");
    let latest_date_str = rows.iter().map(|row| row.get::<&str, _>(0)).max().unwrap_or("2070-01-01");
    let earliest_date = NaiveDate::parse_from_str(earliest_date_str, "%Y-%m-%d").unwrap_or(NaiveDate::from_ymd_opt(1870, 1, 1).unwrap()) - Months::new(11); // subtract 11 month to give some padding on the left side of the chart
    let latest_date = NaiveDate::parse_from_str(latest_date_str, "%Y-%m-%d").unwrap_or(NaiveDate::from_ymd_opt(2070, 1, 1).unwrap()) + Months::new(11); // add 11 month to give some padding on the right side of the chart
    let full_chart_title = format!("{}, {} to {}", chart_title, earliest_date_str, latest_date_str);

    let root = SVGBackend::new(&out_file_name, (15360, 1920)).into_drawing_area();
    let _ = root.fill(&WHITE);
    let mut chart = ChartBuilder::on(&root)
        .margin(10)
        .caption(full_chart_title, ("sans-serif", 36),)
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .set_label_area_size(LabelAreaPosition::Right, 60)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .build_cartesian_2d((earliest_date..latest_date).monthly(),lowest_temp..highest_temp,
        ).unwrap()
        .set_secondary_coord((earliest_date..latest_date).monthly(), f_to_c(lowest_temp)..f_to_c(highest_temp),
        );

    chart
        .configure_mesh()
        .disable_x_mesh()
        //.disable_y_mesh()
        .x_labels(30)
        .label_style(("sans-serif", 14))
        .max_light_lines(4)
        //.y_label_style(("sans-serif", 16))
        .y_desc("Average Temp (F)")
        .draw().unwrap();
    chart
        .configure_secondary_axes()
        .label_style(("sans-serif", 14))
        .y_desc("Average Temp (C)")
        .draw().unwrap();

    chart.draw_series(
        rows.iter()
            .map(|row: &sqlx::mysql::MySqlRow| {
                let date_str: &str = row.get(0);
                let temp: i32 = row.get(1);
                Circle::new(
                (NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap(), temp as f64),
                     2, BLUE.filled())})
            ).expect("Error drawing series");
    // display dup data for diagnostic purposes
    if log_enabled!(Level::Debug) {
        for item in &dup_dates {
            debug!("Duplicate date found: {} for temp: {} count: {}", item.0, item.1.0, item.1.1);
        }
    }
    chart.draw_series(PointSeries::of_element(dup_dates, 4, RGBColor(0,115,153).mix(0.6).filled(),  
        &|item, s, st| {
                let date_str = item.0;
                let temp = item.1.0;
                let count = item.1.1;
                let xy = (NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap(), temp);
                return EmptyElement::at(xy)
                //+ TriangleMarker::new((0,0), s + count +2, st.filled())
                + Circle::new((0,0), s + count + 1, st.filled())
                + Text::new(format!("{}", count), (-2,4),("sans-serif", 12).into_font());
            },
    )).unwrap();

    root.present().expect("Unable to write result to file, please make sure 'date_charts' folder exists under current dir");
    info!("Result has been saved to {}", out_file_name);
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
    let first_year: i32 = get_1st_year(city).await;
    let last_year: i32 = get_end_year(city).await;
    //I was going to make these run in parallel but svgs run so fast that it doesn't matter.
    make_monthly_charts(city, first_year, last_year, city_low, city_high).await?;
    make_fortly_charts(city, first_year, last_year, city_low, city_high).await?;
    make_weekly_charts(city, first_year, last_year, city_low, city_high).await?;
    Ok(())
}
async fn make_monthly_charts(city: &str, first_year: i32, last_year: i32, city_low: i32, city_high: i32) -> Result<(), sqlx::Error> {
    info!("Monthly Line Charts for {city} from {first_year} to {last_year}, with low of {city_low} and high of {city_high}");
    for the_year in first_year..=last_year {
        let file_name = format!("line_charts/{}_{}_month.svg", city, the_year);
        let root = SVGBackend::new(&file_name, (1280, 960)).into_drawing_area();
        let _ = root.fill(&WHITE);
        let title_city = city.to_string().replace("_", " ");
        let mut chartm = ChartBuilder::on(&root)
            .caption(format!("{} {} Average Monthly Temperatures", the_year, title_city), ("sans-serif", 36).into_font())
            .margin_top(20)
            .margin_bottom(10)
            .margin_left(10)
            .margin_right(25)
            .y_label_area_size(54)
            .x_label_area_size(54)
            .build_cartesian_2d((0..624).with_key_points(vec![0,52,104,156,208,260,312,364,416,468,520,572,624]), //624 = 24 * 26 or 12 * 52 
            city_low..city_high
            )
            .unwrap();
        let x_labels = ["|","\u{1F870} Jan | Feb \u{2013}","- Feb | Mar -","- Mar | Apr -","- Apr | May -","- May | Jun -","- Jun | Jul -","\u{2014} Jul | Aug -","- Aug | Sep -","- Sep | Oct -","- Oct | Nov -","\u{2014} Nov | Dec \u{1F872}","|"];
        chartm.configure_mesh()
            .y_max_light_lines(5)// it still makes best guess at optimum number of minor lines, but it won't exceed 5
            .y_label_style(("sans-serif", 20).into_font())
            .x_label_style(("sans-serif", 20).into_font())
            .x_label_formatter(&|x: &i32| x_labels[(x / 52).min(12) as usize].to_string())
            .axis_desc_style(("sans-serif", 24).into_font())
            //.x_desc("Months of the Year")
            .y_desc("Temperature (°F)")
            .draw().unwrap();

        //let tmperiod = "tmonth"; // column names in selected db: can be tmonth, tfort, or tweek
        let city_period = format!("{city}_month");

        let month_positions = [26, 78, 130, 182, 234, 286, 338, 390, 442, 494, 546, 598];
        let temp_result = get_temps("tmonth", &city_period, the_year).await?;

        let hi_temps: Vec<i32> = temp_result.iter().map(|row| row.try_get("tmax").unwrap_or(0)).collect();
        let lo_temps: Vec<i32> = temp_result.iter().map(|row| row.try_get("tmin").unwrap_or(0)).collect();
        let med_hi: Vec<i32> = temp_result.iter().map(|row| row.try_get("mmax").unwrap_or(0)).collect();
        let med_lo: Vec<i32> = temp_result.iter().map(|row| row.try_get("mmin").unwrap_or(0)).collect();

        let high_vec: Vec<(i32, i32)> = month_positions.iter().zip(hi_temps.iter()).map(|(&a, &b)| (a, b)).collect();
        let low_vec: Vec<(i32, i32)> = month_positions.iter().zip(lo_temps.iter()).map(|(&a, &b)| (a, b)).collect();
        let med_hi_vec: Vec<(i32, i32)> = month_positions.iter().zip(med_hi.iter()).map(|(&a, &b)| (a, b)).collect();
        let med_lo_vec: Vec<(i32, i32)> = month_positions.iter().zip(med_lo.iter()).map(|(&a, &b)| (a, b)).collect();
    //draw the high temps 
        let segments = split_segments(high_vec.clone());
        for seg in segments {
            chartm.draw_series(LineSeries::new( //draw line segments for high temps
                seg.clone(),
                &RED 
            )).unwrap();
            chartm.draw_series(PointSeries::of_element( //draw points for high temps
                seg,
                2,
                &RGBColor(128,5,0), //medium dark Red
                &|c, s, st| {
                    return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
                    + Circle::new((0,0),s,st.filled()) // At this point, the new pixel coordinate is established
                    + Text::new(format!("{:?}", c.1), (-4, -12), ("sans-serif", 12).into_font()); //-4,-12 are fudges to position the text better
                },
            )).unwrap();
        }
        //draw the average legend
        let fake_high_avgs = vec![(0, 0)]; //have to do it separately to avoid multiple entries for various segments
        chartm.draw_series(LineSeries::new( // draw legend for high temps
            fake_high_avgs,
            &RED,
            )).unwrap().label("High Avg Temps").legend(|(x,y)| PathElement::new(vec![(x,y),(x+20,y)], &RED));
        //draw the high median points ONLY 
        let segments = split_segments(med_hi_vec.clone());
        for seg in segments {
            chartm.draw_series(PointSeries::of_element( //draw rectangles for high median temps
                seg,
                1,
                &RGBColor(128,5,0), //medium dark Red
                &|c, _s, st| {
                    return EmptyElement::at(c)   
                    + Rectangle::new([(-8,0),(10,1)], st.filled()) 
                    + Text::new(format!("{:?}", c.1), (-4, -12), ("sans-serif", 12).into_font()) 
                ;},
            )).unwrap();
        }
        //draw the median legend
        let fake_high_median = vec![(0, 0)]; //have to do it separately to avoid multiple entries for various segments
        chartm.draw_series(LineSeries::new( // draw median high temp rectangles for legend
            fake_high_median,
            &RGBColor(128,5,0), //medium dark Red
            )).unwrap().label("High Median Temps").legend(|(x,y)| PathElement::new(vec![(x,y),(x+20,y)], &RGBColor(128,5,0)));
    // draw the low temps
        let segments = split_segments(low_vec.clone());
        for seg in segments {
            chartm.draw_series(LineSeries::new( //draw line segments for low temps
                seg.clone(),
                &BLUE // Ocean Blue for low temp line segments
            )).unwrap();
            chartm.draw_series(PointSeries::of_element( //draw circles for low temps
                seg,
                2,
                &RGBColor(0,62,120), //dark ocean blue for low temp dots
                &|c, s, st| {
                    return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
                    + Circle::new((0,0),s,st.filled()) // At this point, the new pixel coordinate is established
                    + Text::new(format!("{:?}", c.1), (-4, -12), ("sans-serif", 12).into_font()); //-4,-12 are fudges to position the text better
                },
            )).unwrap();
        }
        //draw the low avg legend
        let fake_low_avgs = vec![(624, 0)];
        chartm.draw_series(LineSeries::new(
            fake_low_avgs,
            &BLUE, //blue for low temps on legend
            )).unwrap().label("Low Avg Temps").legend(|(x,y)| PathElement::new(vec![(x,y),(x+20,y)], &BLUE));
        //draw the low median points ONLY 
        let segments = split_segments(med_lo_vec.clone());
        for seg in segments {
            chartm.draw_series(PointSeries::of_element(
                seg,
                1,
                &RGBColor(0,62,120), //dark ocean blue
                &|c, _s, _st| {
                    return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
                    + Rectangle::new([(-8,0),(10,1)], RGBColor(0,62,120)) 
                    + Text::new(format!("{:?}", c.1), (-4, -12), ("sans-serif", 12).into_font()) 
                ;},
            )).unwrap();
        }
        //draw the low median legend
        let fake_low_median = vec![(624, 0)];
        chartm.draw_series(LineSeries::new(
            fake_low_median,
            &RGBColor(0,62,120), //dark ocean blue
            )).unwrap().label("Low Median Temps").legend(|(x,y)| PathElement::new(vec![(x,y),(x+20,y)], &RGBColor(0,62,120)));
        // The labels in the legend are set when the series are drawn, not when legend is configured. The chart.configure_series_labels() must be called after all series
        //     that you want included in the legend are drawn, not before. 
        // The labels in legend can be arbitrarily long and the legend will automatically adjust to fit the labels.
        // **margin** should be called padding because it sets the distance between legend elements & legend border and NOT the distance between legend and chart edge
        // **legend_area_size** is the size of the area reserved for the legend example (line), not the size of the legend text itself. If legend_area_size is set too small,
        //      the line will overlap the legend text, even if the margins are set to a large value.
        chartm.configure_series_labels().position(SeriesLabelPosition::UpperRight).margin(8) 
            .legend_area_size(25).border_style(BLUE).background_style(WHITE).label_font(("Calibri", 20)).draw().unwrap(); 
        // draw the circle on the legend for high temps - has to come after or legend covers it
        root.draw(&Circle::new((1098,80), 2, RGBColor(126,12,35).filled())).unwrap();
        // draw the circle on the legend for lowtemps
        root.draw(&Circle::new((1098,130), 2, RGBColor(0,62,120).filled())).unwrap();
        
        root.present().unwrap();
    }
    Ok(())
}
async fn make_fortly_charts(city: &str, first_year: i32, last_year: i32, city_low: i32, city_high: i32) -> Result<(), sqlx::Error> {
    info!("Fortnightly Line Charts for {city} from {first_year} to {last_year}, with low of {city_low} and high of {city_high}");
    for the_year in first_year..=last_year {
        let file_name = format!("line_charts/{}_{}_fort.svg", city, the_year);
        let root = SVGBackend::new(&file_name, (1280, 960)).into_drawing_area();
        let _ = root.fill(&WHITE);
        let title_city = city.to_string().replace("_", " ");
        let mut chartf = ChartBuilder::on(&root)
            .caption(format!("{} {} Average Fortnight Temperatures", the_year, title_city), ("sans-serif", 36).into_font())
            .margin_top(20)
            .margin_bottom(10)
            .margin_left(10)
            .margin_right(25)
            .y_label_area_size(54)
            .x_label_area_size(54)
            .build_cartesian_2d((0..624).with_key_points(vec![0,52,104,156,208,260,312,364,416,468,520,572,624]), //624 = 24 * 26 or 12 * 52 
            city_low..city_high
            )
            .unwrap();
        let x_labels = ["|","\u{1F870} Jan | Feb \u{2013}","- Feb | Mar -","- Mar | Apr -","- Apr | May -","- May | Jun -","- Jun | Jul -","\u{2014} Jul | Aug -","- Aug | Sep -","- Sep | Oct -","- Oct | Nov -","\u{2014} Nov | Dec \u{1F872}","|"];
        chartf.configure_mesh()
            .y_max_light_lines(5)// it still makes best guess at optimum number of minor lines, but it won't exceed 5
            .y_label_style(("sans-serif", 20).into_font())
            .x_label_style(("sans-serif", 20).into_font())
            .x_label_formatter(&|x: &i32| x_labels[(x / 52).min(12) as usize].to_string())
            .axis_desc_style(("sans-serif", 24).into_font())
            //.x_desc("Months of the Year")
            .y_desc("Temperature (°F)")
            .draw().unwrap();

        let city_period = format!("{}_fort", city);
        let fort_positions  = [ 14,  38,  62,  86, 110, 134, 158, 182, 206, 230, 254, 278, 302, 
                                        326, 350, 374, 398, 422, 446, 470, 494, 518, 542, 566, 590, 614];
        let temp_result = get_temps("tfort", &city_period, the_year).await?;

        let hi_temps: Vec<i32> = temp_result.iter().map(|row| row.try_get("tmax").unwrap_or(0)).collect();
        let lo_temps: Vec<i32> = temp_result.iter().map(|row| row.try_get("tmin").unwrap_or(0)).collect();
        let med_hi: Vec<i32> = temp_result.iter().map(|row| row.try_get("mmax").unwrap_or(0)).collect();
        let med_lo: Vec<i32> = temp_result.iter().map(|row| row.try_get("mmin").unwrap_or(0)).collect();

        let high_vec: Vec<(i32, i32)> = fort_positions.iter().zip(hi_temps.iter()).map(|(&a, &b)| (a, b)).collect();
        let low_vec: Vec<(i32, i32)> = fort_positions.iter().zip(lo_temps.iter()).map(|(&a, &b)| (a, b)).collect();
        let med_hi_vec: Vec<(i32, i32)> = fort_positions.iter().zip(med_hi.iter()).map(|(&a, &b)| (a, b)).collect();
        let med_lo_vec: Vec<(i32, i32)> = fort_positions.iter().zip(med_lo.iter()).map(|(&a, &b)| (a, b)).collect();

        let segments = split_segments(high_vec.clone());
        for seg in segments {
            chartf.draw_series(LineSeries::new(
                seg.clone(),
                &RED,
            )).unwrap();
            chartf.draw_series(PointSeries::of_element(
                seg, 2, &RGBColor(126,12,35), //dark red
                &|c, s, st| {
                    return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
                    + Circle::new((0,0),s,st.filled()) // At this point, the new pixel coordinate is established
                    + Text::new(format!("{:?}", c.1), (-4, -12), ("sans-serif", 12).into_font()); //-4,-12 are fudges to position the text better
                },
            )).unwrap();
        }
        //draw the average high legend
        let fake_high_avgs = vec![(0, 0)]; //have to do it separately to avoid multiple entries for various segments
        chartf.draw_series(LineSeries::new(
            fake_high_avgs,
            &RED,
            )).unwrap().label("Avg High Temps").legend(|(x,y)| PathElement::new(vec![(x,y),(x+20,y)], &RED));
        //draw the high median points ONLY 
        let segments = split_segments(med_hi_vec.clone());
        for seg in segments {
            chartf.draw_series(PointSeries::of_element(
                seg,
                1,
                &RGBColor(126,12,35),
                &|c, _s, _st| {
                    return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
                    + Rectangle::new([(-8,0),(10,1)], RGBColor(126,12,35)) 
                    + Text::new(format!("{:?}", c.1), (-4, -12), ("sans-serif", 12).into_font()) 
                ;},
            )).unwrap();
        }
        //draw median legend
        let fake_high_med = vec![(0, 0)]; //have to do it separately to avoid multiple entries for various segments
        chartf.draw_series(LineSeries::new(
            fake_high_med,
            &RGBColor(126,12,35),
            )).unwrap().label("Median High Temps").legend(|(x,y)| PathElement::new(vec![(x,y),(x+20,y)], &RGBColor(126,12,35)));
        // draw low temps
        let segments = split_segments(low_vec.clone());
        for seg in segments {
            chartf.draw_series(LineSeries::new(
                seg.clone(),
                &BLUE,
            )).unwrap();
            chartf.draw_series(PointSeries::of_element(
                seg, 2, &RGBColor(0,62,120), //dark ocean blue
                &|c, s, st| {
                    return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
                    + Circle::new((0,0),s,st.filled()) // At this point, the new pixel coordinate is established
                    + Text::new(format!("{:?}", c.1), (-4, -12), ("sans-serif", 12).into_font()); //-4,-12 are fudges to position the text better
                },
            )).unwrap();
        }
        let fake_low_avgs = vec![(624, 0)];
        chartf.draw_series(LineSeries::new(
            fake_low_avgs,
            &BLUE,
            )).unwrap().label("Avg Low Temps").legend(|(x,y)| PathElement::new(vec![(x,y),(x+20,y)], &BLUE));
        //draw the low median points ONLY 
        let segments = split_segments(med_lo_vec.clone());
        for seg in segments {
            chartf.draw_series(PointSeries::of_element(
                seg,
                1,
                &RGBColor(0,62,120),
                &|c, _s, _st| {
                    return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
                    + Rectangle::new([(-8,0),(10,1)], RGBColor(0,62,120)) 
                    + Text::new(format!("{:?}", c.1), (-4, -12), ("sans-serif", 12).into_font()) 
                ;},
            )).unwrap();
        }
        let fake_low_med = vec![(624, 0)];
        chartf.draw_series(LineSeries::new(
            fake_low_med,
            &RGBColor(0,62,120),
            )).unwrap().label("Median Low Temps").legend(|(x,y)| PathElement::new(vec![(x,y),(x+20,y)], &RGBColor(0,62,120)));
        chartf.configure_series_labels().position(SeriesLabelPosition::UpperRight).margin(8) 
            .legend_area_size(25).border_style(BLUE).background_style(WHITE).label_font(("Calibri", 20)).draw().unwrap(); 
        // draw the circle on the legend for high temps - has to come after or legend covers it
        root.draw(&Circle::new((1098,80), 2, RGBColor(126,12,35).filled())).unwrap();
        // draw the circle on the legend for high temps
        root.draw(&Circle::new((1098,130), 2, RGBColor(0,62,120).filled())).unwrap();

        root.present().unwrap();
    }
    Ok(())
}
async fn make_weekly_charts(city: &str, first_year: i32, last_year: i32, city_low: i32, city_high: i32) -> Result<(), sqlx::Error> {
    info!("Weekly Line Charts for {city} from {first_year} to {last_year}, with low of {city_low} and high of {city_high}");
    for the_year in first_year..=last_year {
        let file_name = format!("line_charts/{}_{}_week.svg", city,the_year);
        let root = SVGBackend::new(&file_name, (1280, 960)).into_drawing_area();
        let _ = root.fill(&WHITE);
        let title_city = city.to_string().replace("_", " ");
        let mut chartw = ChartBuilder::on(&root)
            .caption(format!("{} {} Average Weekly Temperatures", the_year, title_city), ("sans-serif", 36).into_font())
            .margin_top(20)
            .margin_bottom(10)
            .margin_left(10)
            .margin_right(25)
            .y_label_area_size(54)
            .x_label_area_size(54)
            .build_cartesian_2d((0..624).with_key_points(vec![0,52,104,156,208,260,312,364,416,468,520,572,624]), //624 = 24 * 26 or 12 * 52 
            city_low..city_high
            )
            .unwrap();
        let x_labels = ["|","\u{1F870} Jan| Feb \u{2013}","- Feb | Mar -","- Mar | Apr -","- Apr | May -","- May | Jun -","- Jun | Jul -","\u{2014} Jul | Aug -","- Aug | Sep -","- Sep | Oct -","- Oct | Nov -","\u{2014} Nov | Dec \u{1F872}","|"];
        chartw.configure_mesh()
            .y_max_light_lines(5)// it still makes best guess at optimum number of minor lines, but it won't exceed 5
            .y_label_style(("sans-serif", 20).into_font())
            .x_label_style(("sans-serif", 20).into_font())
            .x_label_formatter(&|x: &i32| x_labels[(x / 52).min(12) as usize].to_string())
            .axis_desc_style(("sans-serif", 24).into_font())
            //.x_desc("Months of the Year")
            .y_desc("Temperature (°F)")
            .draw().unwrap();

        let city_period = format!("{}_week", city);
        let week_positions  = [  4,  14,  26,  38,  50,  62,  74,  86,  98, 110, 122, 134, 146, 158, 170, 182, 194, 206, 218, 230, 242, 254, 266, 278, 290, 302, 
                                        314, 326, 338, 350, 362, 374, 386, 398, 410, 422, 434, 446, 458, 470, 482, 494, 506, 518, 530, 542, 554, 566, 578, 590, 602, 614]; 
        let temp_result = get_temps("tweek", &city_period, the_year).await?;

        let hi_temps: Vec<i32> = temp_result.iter().map(|row| row.try_get("tmax").unwrap_or(0)).collect();
        let lo_temps: Vec<i32> = temp_result.iter().map(|row| row.try_get("tmin").unwrap_or(0)).collect();
        let med_hi: Vec<i32> = temp_result.iter().map(|row| row.try_get("mmax").unwrap_or(0)).collect();
        let med_lo: Vec<i32> = temp_result.iter().map(|row| row.try_get("mmin").unwrap_or(0)).collect();

        let high_vec: Vec<(i32, i32)> = week_positions.iter().zip(hi_temps.iter()).map(|(&a, &b)| (a, b)).collect();
        let low_vec: Vec<(i32, i32)> = week_positions.iter().zip(lo_temps.iter()).map(|(&a, &b)| (a, b)).collect();
        let med_hi_vec: Vec<(i32, i32)> = week_positions.iter().zip(med_hi.iter()).map(|(&a, &b)| (a, b)).collect();
        let med_lo_vec: Vec<(i32, i32)> = week_positions.iter().zip(med_lo.iter()).map(|(&a, &b)| (a, b)).collect();

        let segments = split_segments(high_vec.clone());
        for seg in segments {
            chartw.draw_series(LineSeries::new(
                seg.clone(),
                &RED,
            )).unwrap();
            chartw.draw_series(PointSeries::of_element(
                seg,
                2,
                &RGBColor(126,12,35),
                &|c, s, st| {
                    return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
                    + Circle::new((0,0),s,st.filled()) // At this point, the new pixel coordinate is established
                    + Text::new(format!("{:?}", c.1), (-4, -12), ("sans-serif", 12).into_font()); //-4,-12 are fudges to position the text better
                },
            )).unwrap();
        }
        let fake_high_avg = vec![(0, 0)]; //have to do it separately to avoid multiple entries for multiple segments
        chartw.draw_series(LineSeries::new(
            fake_high_avg,
            &RED,
            )).unwrap().label("High Avg Temps").legend(|(x,y)| PathElement::new(vec![(x,y),(x+20,y)], &RED));        
        //draw the high median points ONLY 
        let segments = split_segments(med_hi_vec.clone());
        for seg in segments {
            chartw.draw_series(PointSeries::of_element(
                seg,
                1,
                &RGBColor(126,12,35),
                &|c, _s, _st| {
                    return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
                    + Rectangle::new([(-6,0),(8,1)], RGBColor(126,12,35)) 
                    + Text::new(format!("{:?}", c.1), (-8, -12), ("sans-serif", 12).into_font()) 
                ;},
            )).unwrap();
        }
        // draw high median temps legend - have to do it separately to avoid multiple entries for multiple segments
        let fake_high_med = vec![(0, 0)]; //have to do it separately to avoid multiple entries for multiple segments
        chartw.draw_series(LineSeries::new(
            fake_high_med,
            &RGBColor(126,12,35),
            )).unwrap().label("High Median Temps").legend(|(x,y)| PathElement::new(vec![(x,y),(x+20,y)], &RGBColor(126,12,35)));
        // draw the average low temps
        let segments = split_segments(low_vec.clone());
        for seg in segments {
            chartw.draw_series(LineSeries::new(
                seg.clone(),
                &BLUE,
            )).unwrap();
            chartw.draw_series(PointSeries::of_element(
                seg,
                2,
                &RGBColor(0,62,120),
                &|c, s, st| {
                    return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
                    + Circle::new((0,0),s,st.filled()) // At this point, the new pixel coordinate is established
                    + Text::new(format!("{:?}", c.1), (-4, -12), ("sans-serif", 12).into_font()); //-4,-12 are fudges to position the text better
                },
            )).unwrap();
        }
        // draw the legend
        let fake_low_avg = vec![(624, 0)];
        chartw.draw_series(LineSeries::new(
            fake_low_avg,
            &BLUE,
            )).unwrap().label("Avg Low Temps").legend(|(x,y)| PathElement::new(vec![(x,y),(x+20,y)], &BLUE));
        //draw the low median points ONLY 
        let segments = split_segments(med_lo_vec.clone());
        for seg in segments {
            chartw.draw_series(PointSeries::of_element(
                seg,
                1,
                &RGBColor(0,62,120),
                &|c, _s, _st| {
                    return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
                    + Rectangle::new([(-6,0),(8,1)], &RGBColor(0,62,120)) 
                    + Text::new(format!("{:?}", c.1), (-4, -12), ("sans-serif", 12).into_font()) 
                ;},
            )).unwrap();
        }
        // draw the low median legend
        let fake_low_med = vec![(624, 0)];
        chartw.draw_series(LineSeries::new(
            fake_low_med,
            &RGBColor(0,62,120),
            )).unwrap().label("Median Low Temps").legend(|(x,y)| PathElement::new(vec![(x,y),(x+20,y)], &RGBColor(0,62,120)));
        chartw.configure_series_labels().position(SeriesLabelPosition::UpperRight).margin(8) 
            .legend_area_size(25).border_style(BLUE).background_style(WHITE).label_font(("Calibri", 20)).draw().unwrap(); 
        // draw the circle on the legend for high temps - has to come after or legend covers it
        root.draw(&Circle::new((1098,80), 2, RGBColor(126,12,35).filled())).unwrap();
        // draw the circle on the legend for high temps
        root.draw(&Circle::new((1098,130), 2, RGBColor(0,62,120).filled())).unwrap();

        root.present().unwrap();
    }
    Ok(())
}
fn split_segments(data: Vec<(i32, i32)>) -> Vec<Vec<(i32, i32)>>
{
    let mut segments = Vec::new();
    let mut current_segment = Vec::new();

    for point in data {
        let (_point_x, point_y) = point;
        match point_y {
            point_y if point_y > 200 => {
                debug!("Point_y > 200 found"); //verify that this condition is being hit for the expected points
                if !current_segment.is_empty() {
                    segments.push(current_segment);
                    current_segment = Vec::new();   
                } 
            }  
            _ =>    current_segment.push(point)  // None should never happen
        }
    }
    // Push the last segment if not empty
    if !current_segment.is_empty() {
        segments.push(current_segment);
    }
    segments
}
fn f_to_c(num: f64) -> f64 {
    let result = (num - 32.0) * 5.0 / 9.0;
    debug!("\nC = {}", result);
    result
}

fn find_dup_dates(rows: &Vec<sqlx::mysql::MySqlRow>) -> HashMap<&str, (f64, i32)> {
    let mut dup_dates: HashMap<&str, (f64, i32)> = HashMap::new();
    let mut tdate = "1800-01-01";
    let mut count = 1;

    for row in 0..rows.len()-1 {
        let current_row = &rows[row]; //should be a tdate and a temp (either tmax or tmin depending on the chart)
        let next_row = &rows[row + 1];
        let current_temp: i32 = current_row.get(1);
        let next_temp: i32 = next_row.get(1);
        //let next_date: &str = next_row.get(0);
        if current_temp == next_temp { // ONLY process days with matching temps
            let days_apart = dates_diff_in_days(current_row.get(0), next_row.get(0));
            if days_apart < 45 {
                count += 1;
                if tdate == "1800-01-01" { //this check preserves original tdate with subsequent readings that will appear in the same dot-space
                    tdate = current_row.get(0);
                }
                dup_dates.insert(tdate, (current_temp as f64, count));
                // FIX WHEN NEEDED debug!("Temp = {}: Dates {} and {} are {} days apart. Count: {}. tdate: {}", current_temp, tdate, next_date, days_apart, count, tdate);
            } else {
                count = 1;
                tdate = "1800-01-01";
            }
        } // else if count > 1 push the dup date info into the hashmap and reset count and tdate
         else if count > 1 {
            dup_dates.insert(tdate, (current_temp as f64, count));
            debug!("Non-matching temp found. Temp = {}: Count: {}. tdate: {}", current_temp, count, tdate);
            count = 1;
            tdate = "1800-01-01";
         }
    }
    dup_dates
}
fn dates_diff_in_days(a: &str, b: &str) -> i64 {
    let a_date = NaiveDate::parse_from_str(a, "%Y-%m-%d").unwrap();
    let b_date = NaiveDate::parse_from_str(b, "%Y-%m-%d").unwrap();
    (a_date - b_date).num_days().abs()
}
