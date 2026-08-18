use sqlx::Row;
use sqlx::mysql::MySqlRow;
use log::{error, warn, info, debug, trace, log_enabled, Level};
use plotters::{prelude::*, style::RGBColor};
use plotters::coord::Shift;
use crate::db_ops::*;
use std::io;
use std::io::Write;


pub async fn generate_one_city_bar_charts(the_city: &str) -> Result<(), sqlx::Error> {
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
    let first_year: i32 = get_first_year(&the_city).await;
    let last_year: i32 = get_last_year(&the_city).await;

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

    if log_enabled!(Level::Trace) {
        trace!("Trace level is active (most detailed)");
        debug!("debug level is active (lots of details)");
        info!("info is active (std setting)");
        warn!("warning level is set (typical for production)");
        error!("only errors reported (for mature production only)");
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

// constants used when generating bar charts
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
