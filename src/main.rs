use colorize::AnsiColor;
use std::process::Command;
use execute::Execute;
use sqlx::{mysql::{MySqlPoolOptions, MySqlRow}, MySql, Pool, Row};
use dotenvy::dotenv;
use std::env;
use std::io::{self, Write};
use plotters::prelude::*;
use plotters::coord::Shift;

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

fn main()  {
    let _ = get_db_action();
}

fn get_db_action() -> Result<(), sqlx::Error>{
    let choices = vec![
        "List all cities",
        "Generate Avgs Charts by CITY",
        "Print Avgs Charts by YEAR",
        "Generate Videos from Charts",
        "Exit",
    ];

    loop {
        let prompt_message = "Please select a database action".blue();
        let select = inquire::Select::new(&prompt_message, choices.clone())
            .prompt()
            .expect("Failed to select a database action");

        // each selected action calls a separate tokio runtime so the must each be marked with #[tokio::main]
        if select == "List all cities" {
            println!("Listing all cities...");
            list_all_cities().expect("Failed to list all cities");
        } 
          else if select == "Generate Avgs Charts by CITY" {
            generate_averages_by_city().expect("Failed to generate averages charts by city");
        }
          else if select == "Print Avgs Charts by YEAR" {
            println!("Generate Avgs Charts by YEAR - UNDER CONSTRUCTION");
        }
          else if select == "Generate Videos from Charts" {
            generate_videos_by_city().expect("Failed to generate videos from charts");
        }  else
        if select == "Exit" {
            println!("Exiting the program. Goodbye!");
            break;
        }
    }
    Ok(())
}
//    =======================================================================
#[tokio::main] 
async fn list_all_cities() -> Result<(), sqlx::Error> { 
    // get the db env info from .env file
    dotenv().ok();
    // Set up the database URL from environment variable
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    // Create a connection pool
    let pool: Pool<MySql> = MySqlPoolOptions::new()
        .max_connections(5) // Set the maximum number of connections
        .connect(&database_url)
        .await?;

    let city_list_result: Result<Vec<MySqlRow>, sqlx::Error> = list_cities(&pool).await;
    match city_list_result {
        Ok(_) => { //probably only returns Ok if it found something. otherwise it would return err, no empty check
            let city_list = city_list_result.unwrap();
            for a_city in city_list {
                let c_name: &str = a_city.get("name_of_city");
                println!("Available city: {c_name}");
            }
        },
        Err(e) => eprint!("Cities not found, {} ", e),
    }   
    Ok(())
}
async fn list_cities(pool: &Pool<MySql>) -> Result<Vec<MySqlRow>, sqlx::Error> {
    let query_string = format!("SELECT name_of_city FROM city_names ORDER by name_of_city asc;"); 
    let rows: Vec<sqlx::mysql::MySqlRow> = sqlx::query(&query_string)
        .fetch_all(pool)
        .await?; 
    Ok(rows)
}
//    =======================================================================
#[tokio::main]
async fn generate_averages_by_city() -> Result<(), sqlx::Error> {
    // get the db env info from .env file
    dotenv().ok();
    // Set up the database URL from environment variable
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    // Create a connection pool
    let pool: Pool<MySql> = MySqlPoolOptions::new()
        .max_connections(5) // Set the maximum number of connections
        .connect(&database_url)
        .await?;

    let city_list_result: Result<Vec<MySqlRow>, sqlx::Error> = list_cities(&pool).await;
    let mut cities: Vec<String> = Vec::new();
    
    match city_list_result {
        Ok(_) => { //probably only returns Ok if it found something. otherwise it would return err, no empty check
            let city_list = city_list_result.unwrap();
            for a_city in city_list {
                let c_name: String = a_city.get("name_of_city");
                cities.push(c_name);
            }
        },
        Err(e) => eprint!("Cities not found, {} ", e),
    }   

    let prompt_message = "Please select the cities to GENERATE averages charts".blue();
    let selected_cities = inquire::MultiSelect::new(&prompt_message, cities)
        .prompt()
        .expect("Failed to select cities");

    for the_city in selected_cities {
        println!("Generating averages charts for city of {0}", the_city.clone().red());
        generate_city_averages_charts(&the_city).await?;    
    }

// ------------------------------------------------------------------
    Ok(())
}

async fn generate_city_averages_charts(the_city: &str) -> Result<(), sqlx::Error> {
    // get the db env info from .env file
    dotenv().ok();
    // Set up the database URL from environment variable
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    // Create a connection pool
    let pool: Pool<MySql> = MySqlPoolOptions::new()
        .max_connections(5) // Set the maximum number of connections
        .connect(&database_url)
        .await?;

    let city  = the_city;

    let city_low: i32;
    let city_high: i32;   
    let fn_get_min_max: Result<(i32, i32), sqlx::Error> = get_city_min_max(&pool, city).await;
    match fn_get_min_max { // city_low & city_high here must be initialized in this block to make compiler happy
        Ok(_) => {let min_max: &(i32, i32) = &fn_get_min_max.unwrap();
                city_low = min_max.0;  city_high = min_max.1; println!("Low: {city_low}  High: {city_high}")},
        Err(e) =>  {city_low = 0; city_high = 0; eprintln!("Error getting City min max: {}",e)}
    }  

    let mut first_year: i32 = 0; 
    let first_year_result: Result<Vec<sqlx::mysql::MySqlRow>, sqlx::Error> = get_first_year(&pool, city).await;
    match first_year_result {
        Ok(_) => { 
            let first_year_row = &first_year_result.unwrap(); //unwrap the row
            let first_year_str: &str = first_year_row[0].get("tdate"); //get date string, for ex. 2020-09-05
            first_year = first_year_str[0..4].parse().unwrap();  //parse first 4 digits as an i32
            println!("First year for {}: {}", city, &first_year);
        },
        Err(e) => eprintln!("Error executing function: {}", e),
    } 
    println!("First year: {first_year}  City: {city}"); 
    let mut last_year: i32 = 0; 
    let last_year_result: Result<Vec<sqlx::mysql::MySqlRow>, sqlx::Error> = get_last_year(&pool, city).await;
    match last_year_result {
        Ok(_) => { 
            let last_year_row = &last_year_result.unwrap(); //unwrap the row
            let last_year_str: &str = last_year_row[0].get("tdate"); //get date string, for ex. 2020-11-21
            last_year = last_year_str[0..4].parse().unwrap();  //parse first 4 digits as an i32
            println!("Last year for {}: {}", city, &last_year);
        },
        Err(e) => eprintln!("Error executing function: {}", e),
    } 

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

    println!("Axis Height: {AXIS_HEIGHT} Y range: {y_range} degrees. Pixels per degree: {pixel_per_degree}. Zero offset: {zero_line_offset}");
 
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
        let mfile_name = format!("imgs/{city}_{the_year}_month.png");
        let wfile_name = format!("imgs/{city}_{the_year}_week.png");
        let ffile_name = format!("imgs/{city}_{the_year}_fort.png");

        let mtitle_text = format!("{the_year} {city}  Monthly Avg Temperatures");
        let wtitle_text = format!("{the_year} {city}  Weekly Avg Temperatures");
        let ftitle_text = format!("{the_year} {city}  Fortnightly Avg Temperatures");
       // ------ monthly chart ------------------------------------------------------
        let mdwg = BitMapBackend::new(&mfile_name, (DWG_WIDTH as u32, DWG_HEIGHT as u32)).into_drawing_area();
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

        let fn_result: Result<Vec<sqlx::mysql::MySqlRow>, sqlx::Error> = get_temps(&pool, tmperiod, &mcity_period, the_year).await;
        match fn_result {
            Ok(_) => { 
                //print_avgs(period, &city_period, the_year, &fn_result.as_ref().unwrap());
                draw_hi_temps(&mdwg, mperiod, zero_line_offset, pixel_per_degree, &fn_result.as_ref().unwrap()).expect("Draw Hi Temps Failed"); 
                draw_low_temps(&mdwg, mperiod, zero_line_offset, pixel_per_degree, &fn_result.as_ref().unwrap()).expect("Draw Low Temps Failed");
            }
            Err(e) => eprintln!("Error getting temperatures from db: {}", e),
        }
        mdwg.present().expect("Failed Chart drawing");
        // ----------- wweekly chart ------------------------------------------------------
        let wdwg = BitMapBackend::new(&wfile_name, (DWG_WIDTH as u32, DWG_HEIGHT as u32)).into_drawing_area();
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

        let fn_result: Result<Vec<sqlx::mysql::MySqlRow>, sqlx::Error> = get_temps(&pool, twperiod, &wcity_period, the_year).await;
        match fn_result {
            Ok(_) => { 
                //print_avgs(period, &city_period, the_year, &fn_result.as_ref().unwrap());
                draw_hi_temps(&wdwg, wperiod, zero_line_offset, pixel_per_degree, &fn_result.as_ref().unwrap()).expect("Draw Hi Temps Failed"); 
                draw_low_temps(&wdwg, wperiod, zero_line_offset, pixel_per_degree, &fn_result.as_ref().unwrap()).expect("Draw Low Temps Failed");
            }
            Err(e) => eprintln!("Error getting temperatures from db: {}", e),
        }
        wdwg.present().expect("Failed Chart drawing");
        // ----------- fortnightly chart ------------------------------------------------------
        let fdwg = BitMapBackend::new(&ffile_name, (DWG_WIDTH as u32, DWG_HEIGHT as u32)).into_drawing_area();
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

        let fn_result: Result<Vec<sqlx::mysql::MySqlRow>, sqlx::Error> = get_temps(&pool, tfperiod, &fcity_period, the_year).await;
        match fn_result {
            Ok(_) => { 
                //print_avgs(period, &city_period, the_year, &fn_result.as_ref().unwrap());
                draw_hi_temps(&fdwg, fperiod, zero_line_offset, pixel_per_degree, &fn_result.as_ref().unwrap()).expect("Draw Hi Temps Failed"); 
                draw_low_temps(&fdwg, fperiod, zero_line_offset, pixel_per_degree, &fn_result.as_ref().unwrap()).expect("Draw Low Temps Failed");
            }
            Err(e) => eprintln!("Error getting temperatures from db: {}", e),
        }
        fdwg.present().expect("Failed Chart drawing");
    }
    Ok(())
}

fn draw_legend(dwg: &DrawingArea<BitMapBackend, Shift>) -> Result<(), Box<dyn std::error::Error>> {
    let legend_x = LEFT_MARGIN + AXIS_WIDTH - 120;
    let legend_y = TOP_MARGIN + 12;
    let rect_size = 20;
    let spacing = 30;

    // Draw legend boxes and labels
    let warm_colors = vec![
        (get_warm_colors(110), "100+°F"),
        (get_warm_colors(90), "81-100°F"),
        (get_warm_colors(70), "61-80°F"),
        (get_warm_colors(50), "33-60°F"),
        (get_warm_colors(30), "<= 32°F"),
    ];

    // Draw legend background
    dwg.draw(&Rectangle::new(
        [(legend_x - 10, legend_y - 10), (legend_x + rect_size + 98, legend_y + (spacing * warm_colors.len() as i32))],
        Into::<ShapeStyle>::into(&WHITE).filled(),
    ))?;
    // Draw box around legend
    dwg.draw(&Rectangle::new(
        [(legend_x - 10, legend_y - 10), (legend_x + rect_size + 98, legend_y + (spacing * warm_colors.len() as i32))],
        Into::<ShapeStyle>::into(&BLACK).stroke_width(2),
    ))?;
    //draw the legend boxes and labels
    for (i, (color, label)) in warm_colors.iter().enumerate() {
        let y_offset = i as i32 * spacing;
        dwg.draw(&Rectangle::new(
            [(legend_x, legend_y + y_offset), (legend_x + rect_size, legend_y + rect_size + y_offset)],
            Into::<ShapeStyle>::into(color).filled(),
        ))?;
        dwg.draw_text(label, &("sans-serif", 16).into_font().color(&BLACK), (legend_x + rect_size + 10, legend_y + rect_size / 2 + y_offset-3))?;
    }
    Ok(())
}

fn draw_hi_temps(dwg: &DrawingArea<BitMapBackend, Shift>, period: &str, z_line_offset: f64,  pixel_per_degree: f64, rows: &Vec<MySqlRow>) -> Result<(), Box<dyn std::error::Error>> {
    let mut y_adj: i32;
    match period {
        "Week" => {    
            for i in 1..53 {
                let x = i * (AXIS_WIDTH / 52) + LEFT_MARGIN;
                let idx: usize = i.try_into().unwrap();
                let tmp: i32; // get ready to hold the hi_temp to display
                let hi_result = rows[idx-1].try_get("tmax");
                match hi_result {
                    Ok(_) => { tmp = hi_result.unwrap(); } //set tmp to hi_temp
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
                    Ok(_) => { tmp = hi_result.unwrap(); }
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
                    Ok(_) => { tmp = hi_result.unwrap(); }
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
        _ => println!("Unknown Period"),
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
fn draw_low_temps(dwg: &DrawingArea<BitMapBackend, Shift>, period: &str, z_line_offset: f64, pixel_per_degree: f64, rows: &Vec<MySqlRow>) -> Result<(), Box<dyn std::error::Error>>  {
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
                    Ok(_) =>  { tmp = low_result.unwrap(); }
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
                    Ok(_) =>  { tmp = low_result.unwrap(); }
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
                    Ok(_) =>  { tmp = low_result.unwrap(); }
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
        _ => println!("Unknown Period"),
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
    } else if temp > 80 && temp <= 100 { // RosyBrown
        RGBColor(188, 143, 143) 
    } else {
        RGBColor(50, 205, 50)  // HotPink
    }
}

fn draw_axes(dwg: &DrawingArea<BitMapBackend, Shift>) -> Result<(), Box<dyn std::error::Error>> {
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

fn draw_grids(dwg: &DrawingArea<BitMapBackend, Shift>) -> Result<(), Box<dyn std::error::Error>> {
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
                Into::<ShapeStyle>::into(RGBAColor(128, 128, 128, 0.5)).stroke_width(1),
            ))?;
            dwg.draw(&PathElement::new(
                vec![(LEFT_MARGIN+2, y_minor2), (AXIS_WIDTH + LEFT_MARGIN, y_minor2)],
                Into::<ShapeStyle>::into(RGBAColor(128, 128, 128, 0.5)).stroke_width(1),
            ))?;
            dwg.draw(&PathElement::new(
                vec![(LEFT_MARGIN+2, y_minor3), (AXIS_WIDTH + LEFT_MARGIN, y_minor3)],
                Into::<ShapeStyle>::into(RGBAColor(128, 128, 128, 0.5)).stroke_width(1),
            ))?;
        }
    }
    Ok(())
}

fn draw_title(dwg: &DrawingArea<BitMapBackend, Shift>, title_text: &str, title_style: TextStyle) -> Result<(), Box<dyn std::error::Error>> {
    let (title_width, title_height) = dwg.estimate_text_size(&title_text, &title_style)?;
    
    dwg.draw_text(&title_text, &title_style,
        ((DWG_WIDTH / 2) as i32 - (title_width as i32 / 2), title_height as i32 - 10),
    )?; 
    Ok(())
}

fn draw_axis_labels(dwg: &DrawingArea<BitMapBackend, Shift>,
                         x_axis_style: TextStyle, 
                         y_axis_style: TextStyle, 
                         period: &str,
                         _y_lowest: i32,
                         y_highest: i32,
                         y_range: i32) -> Result<(), Box<dyn std::error::Error>> {
    match period {
        "Week" => {
            let (_x_label_width, x_label_height) = dwg.estimate_text_size(&format!("55"), &x_axis_style)?;
            //println!("x_label_width: {}, x_label_height: {}", _x_label_width, x_label_height);
            for i in 1..53 {
                let x = i * (AXIS_WIDTH / 52) + LEFT_MARGIN;
                let i_str = i.to_string();
                dwg.draw_text(&i_str, &x_axis_style, (x - 1, AXIS_HEIGHT + TOP_MARGIN + (x_label_height / 2) as i32 + 5))?;
            }
        },
        "Fort" => {
            let (_x_label_width, x_label_height) = dwg.estimate_text_size(&format!("55"), &x_axis_style)?;
            //println!("x_label_width: {}, x_label_height: {}", _x_label_width, x_label_height);
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
        _ => println!("Unknown Period"),
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
async fn get_city_min_max(pool: &Pool<MySql>, city: &str) -> Result<(i32, i32), sqlx::Error> {
    let query_string = format!("SELECT min_temp, max_temp FROM city_names WHERE name_of_city = '{}'", city); // Adjust table name as needed
    let rows: Vec<sqlx::mysql::MySqlRow> = sqlx::query(&query_string)
        .fetch_all(pool)
        .await?; // had to make this function return a Result to use the ? operator

    let lo: i32 = rows[0].get(0);
    let hi: i32 = rows[0].get(1);

    Ok((lo, hi))
}

async fn get_temps(pool: &Pool<MySql>, tperiod: &str, city: &str, year: i32) -> Result<Vec<MySqlRow>, sqlx::Error> {
    let query_string = format!("SELECT tyear, {}, tmax, tmin FROM {} WHERE tyear = {}", tperiod, city, year ); // Adjust table name as needed
    let rows: Vec<sqlx::mysql::MySqlRow> = sqlx::query(&query_string)
        .fetch_all(pool)
        .await?; // had to make this function return a Result to use the ? operator
    Ok(rows)
}
async fn get_first_year(pool: &Pool<MySql>, city: &str) -> Result<Vec<MySqlRow>, sqlx::Error> {
    let query_string = format!("SELECT tdate FROM {} ORDER BY tdate ASC LIMIT 1", city); // Adjust table name as needed
    let rows: Vec<sqlx::mysql::MySqlRow> = sqlx::query(&query_string)
        .fetch_all(pool)
        .await?; // had to make this function return a Result to use the ? operator
    Ok(rows)
}
async fn get_last_year(pool: &Pool<MySql>, city: &str) -> Result<Vec<MySqlRow>, sqlx::Error> {
    let query_string = format!("SELECT tdate FROM {} ORDER BY tdate DESC LIMIT 1", city); // Adjust table name as needed
    let rows: Vec<sqlx::mysql::MySqlRow> = sqlx::query(&query_string)
        .fetch_all(pool)
        .await?; // had to make this function return a Result to use the ? operator
    Ok(rows)
}

#[tokio::main]
async fn generate_videos_by_city() -> Result<(), sqlx::Error> {
    // get the db env info from .env file
    dotenv().ok();
    // Set up the database URL from environment variable
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    // Create a connection pool
    let pool: Pool<MySql> = MySqlPoolOptions::new()
        .max_connections(5) // Set the maximum number of connections
        .connect(&database_url)
        .await?;

    let city_list_result: Result<Vec<MySqlRow>, sqlx::Error> = list_cities(&pool).await;
    let mut cities: Vec<String> = Vec::new();
    
    match city_list_result {
        Ok(_) => { //probably only returns Ok if it found something. otherwise it would return err, no empty check
            let city_list = city_list_result.unwrap();
            for a_city in city_list {
                let c_name: String = a_city.get("name_of_city");
                cities.push(c_name);
            }
        },
        Err(e) => eprint!("Cities not found, {} ", e),
    }   

    let prompt_message = "Please select the cities to GENERATE videos".blue();
    let selected_cities = inquire::MultiSelect::new(&prompt_message, cities)
        .prompt()
        .expect("Failed to select cities");

    for the_city in selected_cities {
        println!("Generating videos for city of {0}", the_city.clone().red());
        generate_videos_from_charts(&the_city).await?;    
    }
    Ok(())
}

async fn generate_videos_from_charts(the_city: &str) -> Result<(), sqlx::Error> {
    //let city  = the_city.to_string();
    println!("Generating videos for the city of {0}", the_city);
    // get the db env info from .env file
    dotenv().ok();
    // Set up the database URL from environment variable
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    // Create a connection pool
    let pool: Pool<MySql> = MySqlPoolOptions::new()
        .max_connections(5) // Set the maximum number of connections
        .connect(&database_url)
        .await?;

    let mut first_year: i32 = 0; 
    let first_year_result: Result<Vec<sqlx::mysql::MySqlRow>, sqlx::Error> = get_first_year(&pool, the_city).await;
    match first_year_result {
        Ok(_) => { 
            let first_year_row = &first_year_result.unwrap(); //unwrap the row
            let first_year_str: &str = first_year_row[0].get("tdate"); //get date string, for ex. 2020-09-05
            first_year = first_year_str[0..4].parse().unwrap();  //parse first 4 digits as an i32
            //println!("First year for {}: {}", the_city, &first_year);
        },
        Err(e) => eprintln!("Error executing function: {}", e),
    } 
    println!("First year: {first_year}  for City: {the_city}"); 
    // let mut last_year: i32 = 0; // videos will be gen'd for every year after first year so last not needed here
    
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

    println!("Executing command: {:?}", cmd);
    let mut output = cmd.execute_output().unwrap();

    period = "week";
    //let frames_per_second = fps.to_string();
    let mut cmd2 = Command::new(FFMPEG_PATH);
    cmd2.arg(HIDE_BANNER).arg(OVERWRITE).arg(START_NUMBER).arg(start_year.to_string())
    .arg(FRAMERATE).arg(&frames_per_second);
    let input_arg = format!("{}{}_%4d_{}.png", PNG_FOLDER, city, period);
    cmd2.arg("-i").arg(&input_arg).arg("-c:v").arg("libx264");
    output_arg = format!("{}{}_{}_{}.mp4", VIDEO_FOLDER, city, period, &frames_per_second);
    cmd2.arg(&output_arg);

    println!("Executing command: {:?}", cmd2);
    output = cmd2.execute_output().unwrap();

    period = "fort";
    //let frames_per_second = fps.to_string();
    let mut cmd3 = Command::new(FFMPEG_PATH);
    cmd3.arg(HIDE_BANNER).arg(OVERWRITE).arg(START_NUMBER).arg(start_year.to_string())
    .arg(FRAMERATE).arg(&frames_per_second);
    let input_arg = format!("{}{}_%4d_{}.png", PNG_FOLDER, city, period);
    cmd3.arg("-i").arg(&input_arg).arg("-c:v").arg("libx264");
    output_arg = format!("{}{}_{}_{}.mp4", VIDEO_FOLDER, city, period, &frames_per_second);
    cmd3.arg(&output_arg);

    println!("Executing command: {:?}", cmd3);
    output = cmd3.execute_output().unwrap();


    if let Some(exit_code) = output.status.code() {
        if exit_code == 0 {
            println!("The weekly exit code is 0 (Ok)");
        } else {
            eprintln!("Error executing `{}` with in-file: {} and out-file: {}", FFMPEG_PATH, input_arg, output_arg);
        }
    } 

    Ok(())
}
