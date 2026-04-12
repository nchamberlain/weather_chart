use plotters::prelude::*;

pub struct ChartConfig {
    pub dwg_width: i32 = 1280,
    pub dwg_height: i32 = 800,
    pub top_margin: i32 = 50,
    pub bottom_margin: i32 = 50,
    pub left_margin: i32 = 50,
    pub right_margin: i32 = 50,
    pub axis_width: i32 = dwg_width - left_margin - right_margin,
    pub axis_height: i32 = dwg_height - top_margin - bottom_margin,
    pub h_tick_width: i32 = axis_width / 4,
    pub v_tick_height: i32 = axis_height / 10,
}


pub fn base_chart (chart_config: &ChartConfig) -> Result<(), Box<dyn std::error::Error>> {
        let base_dwg = BitMapBackend::new(&mfile_name, (DWG_WIDTH as u32, DWG_HEIGHT as u32)).into_drawing_area();
        mdwg.fill(&WHITE).expect("Failed to fill dwg"); //this automatically makes a rectangle size of drawing area and fills it with white

        // Draw axis lines on the drawing area
        draw_axes(&mdwg).expect("Failed to draw axes");
        
        // Draw horizontal and verticlal grid lines with tick marks
        draw_grids(&mdwg).expect("Failed to draw grids");

        // Draw title
        draw_title(&mdwg, &mtitle_text, title_style.clone()).expect("Failed to draw title");

        // Draw axis labels
        draw_axis_labels(&mdwg, x_axis_style.clone(), y_axis_style.clone(), mperiod, y_lowest, y_highest, y_range).expect("Failed to draw axis labels");

}


async fn generate_city_bar_charts(the_city: &str) -> Result<(), sqlx::Error> {
    let city  = the_city;
    let city_low: i32;
    let city_high: i32;   
    let fn_get_min_max: Result<(i32, i32), sqlx::Error> = get_city_min_max(city).await;
    match fn_get_min_max { // city_low & city_high here must be initialized in this block to make compiler happy
        Ok(_) => {let min_max: &(i32, i32) = &fn_get_min_max.unwrap();
                city_low = min_max.0;  city_high = min_max.1; /*println!("Low: {city_low}  High: {city_high}")*/},
        Err(e) =>  {city_low = 0; city_high = 0; eprintln!("Error getting City min max: {}",e)}
    }  
    //let mut first_year: i32 = 0; 
    let first_year: i32 = get_1st_year(&the_city).await;
    let last_year: i32 = get_end_year(&the_city).await;

    println!("City: {city} from {first_year} to {last_year}"); 
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

        let fn_result: Result<Vec<sqlx::mysql::MySqlRow>, sqlx::Error> = get_temps(tmperiod, &mcity_period, the_year).await;
        match fn_result {
            Ok(_) => { 
                //print_avgs(period, &city_period, the_year, &fn_result.as_ref().unwrap());
                draw_hi_temps(&mdwg, mperiod, zero_line_offset, pixel_per_degree, &fn_result.as_ref().unwrap()).expect("Draw Hi Temps Failed"); 
                draw_low_temps(&mdwg, mperiod, zero_line_offset, pixel_per_degree, &fn_result.as_ref().unwrap()).expect("Draw Low Temps Failed");
            }
            Err(e) => eprintln!("Error getting temperatures from db: {}", e),
        }
        mdwg.present().expect("Failed monthly Chart drawing");
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

        let fn_result: Result<Vec<sqlx::mysql::MySqlRow>, sqlx::Error> = get_temps(twperiod, &wcity_period, the_year).await;
        match fn_result {
            Ok(_) => { 
                //print_avgs(period, &city_period, the_year, &fn_result.as_ref().unwrap());
                draw_hi_temps(&wdwg, wperiod, zero_line_offset, pixel_per_degree, &fn_result.as_ref().unwrap()).expect("Draw Hi Temps Failed"); 
                draw_low_temps(&wdwg, wperiod, zero_line_offset, pixel_per_degree, &fn_result.as_ref().unwrap()).expect("Draw Low Temps Failed");
            }
            Err(e) => eprintln!("Error getting temperatures from db: {}", e),
        }
        wdwg.present().expect("Failed weekly Chart drawing");
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

        let fn_result: Result<Vec<sqlx::mysql::MySqlRow>, sqlx::Error> = get_temps(tfperiod, &fcity_period, the_year).await;
        match fn_result {
            Ok(_) => { 
                //print_avgs(period, &city_period, the_year, &fn_result.as_ref().unwrap());
                draw_hi_temps(&fdwg, fperiod, zero_line_offset, pixel_per_degree, &fn_result.as_ref().unwrap()).expect("Draw Hi Temps Failed"); 
                draw_low_temps(&fdwg, fperiod, zero_line_offset, pixel_per_degree, &fn_result.as_ref().unwrap()).expect("Draw Low Temps Failed");
            }
            Err(e) => eprintln!("Error getting temperatures from db: {}", e),
        }
        fdwg.present().expect("Failed fortnight Chart drawing");
    }
    Ok(())
}