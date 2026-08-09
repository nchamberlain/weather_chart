use plotters::{prelude::*, style::RGBAColor};
use plotters::coord::Shift;
use log::{error, warn, info, debug, trace, log_enabled, Level};
use sqlx::mysql::MySqlRow;
use sqlx::Row;

pub fn draw_freq_chart(the_city: &str, first_year: i32, last_year: i32, rows: Vec<MySqlRow>) -> Result<(), Box<dyn std::error::Error>>{
    info!("Drawing frequency chart for {the_city} from {first_year} to {last_year}");
    // column width should also be based on how best to fit the chart width 
    //let number of rows returned determine the number of years
    let years: u32 = rows.len() as u32;
    let width: u32 = (years * 60) + 80 + 20;
    info!("Freq Chart for {the_city}: width = {width} years = {years}");

    let city_path = format!("freq_charts/{}_range_freq.svg", the_city);
    let root = SVGBackend::new(&city_path, (width, 1920)).into_drawing_area();
    root.fill(&WHITE).expect("Failed to fill the drawing area");

    let mut x: i32 = 80;
    let y: i32 = 30;

    for row in rows {
        let _ = draw_column(&root, x, y, row);
        x += 60;
    }

    root.present().expect("Failed to draw the drawing area");    
    if log_enabled!(Level::Trace) {
        //probably don't need to see this very often
        trace!("Trace level is active (most detailed)");
        debug!("debug level is active (lots of details)");
        info!("info is active (std setting)");
        warn!("warning level is set (typical for production)");
        error!("only errors reported (for mature production onlt)");
    }
    Ok(())
}
fn draw_column(root: &DrawingArea<SVGBackend, Shift>, x: i32, mut y: i32, row: MySqlRow) -> Result<(), Box<dyn std::error::Error>> {
    let frz: i32 = row.get("count_freezing");
    let c30s: i32 = row.get("count_30s");
    let c40s: i32 = row.get("count_40s");
    let c50s: i32 = row.get("count_50s");
    let c60s: i32 = row.get("count_60s");
    let c70s: i32 = row.get("count_70s");
    let c80s: i32 = row.get("count_80s");
    let c90s: i32 = row.get("count_90s");
    let c100s: i32 = row.get("count_100s");

    draw_rectangle(&root, x, y, x + 30, y + 200, STYLE_100S, c100s)?;
    y += 200;
    draw_rectangle(&root, x, y, x + 30, y + 200, STYLE_90S, c90s)?;
    y += 200;
    draw_rectangle(&root, x, y, x + 30, y + 200, STYLE_80S, c80s)?;
    y += 200;
    draw_rectangle(&root, x, y, x + 30, y + 200, STYLE_70S, c70s)?;
    y += 200;
    draw_rectangle(&root, x, y, x + 30, y + 200, STYLE_60S, c60s)?;
    y += 200;
    draw_rectangle(&root, x, y, x + 30, y + 200, STYLE_50S, c50s)?;
    y += 200;
    draw_rectangle(&root, x, y, x + 30, y + 200, STYLE_40S, c40s)?;
    y += 200;
    draw_rectangle(&root, x, y, x + 30, y + 200, STYLE_30S, c30s)?;
    y += 200;
    draw_rectangle(&root, x, y, x + 30, y + 200, STYLE_FREEZING, frz)?;
    Ok(())
}
fn draw_rectangle(root: &DrawingArea<SVGBackend, Shift>, x: i32, y: i32, xr: i32, yr: i32, style: ShapeStyle, freq: i32) -> Result<(), Box<dyn std::error::Error>> {
    let freq_offset: i32;
    let bars: i32;
    match freq {
        f if f < 21 => {freq_offset = 200 - freq * 10; bars = 0;} ,
        f if f < 41 => {freq_offset = 200 - (freq-20) * 10; bars = 1;} ,
        f if f < 51 => {freq_offset = 200 - (freq-40) * 10; bars = 2;} ,
        f if f < 52 => {freq_offset = 200 - (freq-41) * 10; bars = 2;} ,
        f if f < 53 => {freq_offset = 200 - (freq-42) * 10; bars = 2;} ,
        _ => {freq_offset = 0; bars = 0;}
    }
    root.draw(&Rectangle::new([(x, y + freq_offset), (xr, yr)], style.clone()))?;
    root.draw(&Rectangle::new([(x, y), (xr, yr)], STYLE_BLACK))?;
    let text_style = ("sans-serif", 16).into_font().color(&BLACK);
    root.draw_text(&freq.to_string(), &text_style, (x + 32, y + 98))?;
    match bars {
        0 => {},
        1 => {root.draw(&Rectangle::new([(x + 31, y + 110), (x + 48, y + 200)], style.clone()))?;
              root.draw(&Rectangle::new([(x + 31, y + 110), (x + 48, y + 200)], STYLE_BLACK))?;
        },
        2 => {root.draw(&Rectangle::new([(x + 31, y ), (x + 48, y + 90)], style.clone()))?;
              root.draw(&Rectangle::new([(x + 31, y ), (x + 48, y + 90)], STYLE_BLACK))?;
              root.draw(&Rectangle::new([(x + 31, y + 110), (x + 48, y + 200)], style.clone()))?;
              root.draw(&Rectangle::new([(x + 31, y + 110), (x + 48, y + 200)], STYLE_BLACK))?;
            },
        _ => {}
    }

    Ok(())
}
const STYLE_100S: ShapeStyle = ShapeStyle { // RED
    color: RGBAColor(255, 0, 0, 0.9),
    filled: true,
    stroke_width: 1,
};
const STYLE_90S: ShapeStyle = ShapeStyle { // crimson
    color: RGBAColor(220, 20, 60, 0.88),
    filled: true,
    stroke_width: 1,
};
const STYLE_80S: ShapeStyle = ShapeStyle { //orange red
    color: RGBAColor(255, 69, 0, 0.7),
    filled: true,
    stroke_width: 1,
};
const STYLE_70S: ShapeStyle = ShapeStyle { //light green
    color: RGBAColor(50, 205, 51, 0.7),
    filled: true,
    stroke_width: 1,
};
const STYLE_60S: ShapeStyle = ShapeStyle { //green 
    color: RGBAColor(0,128, 0, 0.9),
    filled: true,
    stroke_width: 1,
};
const STYLE_50S: ShapeStyle = ShapeStyle { //Teal
    color: RGBAColor(0, 139, 128, 0.6),
    filled: true,
    stroke_width: 1,
};
const STYLE_40S: ShapeStyle = ShapeStyle { //steel blue
    color: RGBAColor(70, 130, 132, 0.95),
    filled: true,
    stroke_width: 1,
};
const STYLE_30S: ShapeStyle = ShapeStyle { //medium blue
    color: RGBAColor(0, 0, 205, 0.7),
    filled: true,
    stroke_width: 1,
};
const STYLE_FREEZING: ShapeStyle = ShapeStyle { // navy
    color: RGBAColor(0, 0, 128, 0.95),
    filled: true,
    stroke_width: 1,
};
const STYLE_BLACK: ShapeStyle = ShapeStyle { // black
    color: RGBAColor(0, 0, 0, 0.95),
    filled: false,
    stroke_width: 1,
};
