use plotters::{prelude::*, style::RGBAColor};
use plotters::coord::Shift;
use log::{error, warn, info, debug, trace, log_enabled, Level};

pub fn draw_freq_chart(the_city: &str, first_year: i32, last_year: i32) -> Result<(), Box<dyn std::error::Error>>{
    info!("Drawing frequency chart for {the_city} from {first_year} to {last_year}");
    // calc number of years to determine how wide to make the chart
    // column width should also be based on how best to fit the chart width 
    //let years = last_year - first_year + 1;
    //let width = years * 60 + 80;

    let city_path = format!("freq_charts/{}_range_freq.svg", the_city);
    let root = SVGBackend::new(&city_path, (9200, 960)).into_drawing_area();
    root.fill(&WHITE).expect("Failed to fill the drawing area");

    let mut x: i32 = 80;
    let y: i32 = 30;

    for _ in 0..150 {
        let _ = draw_column(&root, x, y);
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
fn draw_column(root: &DrawingArea<SVGBackend, Shift>, x: i32, mut y: i32) -> Result<(), Box<dyn std::error::Error>> {
    let mut freq = 46;
    draw_rectangle(&root, x, y, x + 30, y + 100, STYLE_100S, freq)?;
    y += 100;
    freq -= 5;
    draw_rectangle(&root, x, y, x + 30, y + 100, STYLE_90S, freq)?;
    y += 100;
    freq -= 5;
    draw_rectangle(&root, x, y, x + 30, y + 100, STYLE_80S, freq)?;
    y += 100;
    freq -= 5;
    draw_rectangle(&root, x, y, x + 30, y + 100, STYLE_70S, freq)?;
    y += 100;
    freq -= 5;
    draw_rectangle(&root, x, y, x + 30, y + 100, STYLE_60S, freq)?;
    y += 100;
    freq -= 5;
    draw_rectangle(&root, x, y, x + 30, y + 100, STYLE_50S, freq)?;
    y += 100;
    freq -= 5;
    draw_rectangle(&root, x, y, x + 30, y + 100, STYLE_40S, freq)?;
    y += 100;
    freq -= 5;
    draw_rectangle(&root, x, y, x + 30, y + 100, STYLE_30S, freq)?;
    y += 100;
    freq -= 5;
    draw_rectangle(&root, x, y, x + 30, y + 100, STYLE_FREEZING, freq)?;
    //println!("Drawing column at x: {}, final y: {}, final freq: {}", x, y, freq);
    Ok(())
}
fn draw_rectangle(root: &DrawingArea<SVGBackend, Shift>, x: i32, y: i32, xr: i32, yr: i32, style: ShapeStyle, freq: i32) -> Result<(), Box<dyn std::error::Error>> {
    let freq_offset: i32;
    let dots: i32;
    match freq {
        f if f < 11 => {freq_offset = 100 - freq * 10; dots = 0;} ,
        f if f < 21 => {freq_offset = 100 - (freq-10) * 10; dots = 1;} ,
        f if f < 31 => {freq_offset = 100 - (freq-20) * 10; dots = 2;} ,
        f if f < 41 => {freq_offset = 100 - (freq-30) * 10; dots = 3;} ,
        f if f < 51 => {freq_offset = 100 - (freq-40) * 10; dots = 4;} ,
        f if f < 52 => {freq_offset = 100 - (freq-41) * 10; dots = 4;} ,
        f if f < 53 => {freq_offset = 100 - (freq-42) * 10; dots = 4;} ,
        _ => {freq_offset = 0; dots = 0;}
    }
    root.draw(&Rectangle::new([(x, y + freq_offset), (xr, yr)], style.clone()))?;
    root.draw(&Rectangle::new([(x, y), (xr, yr)], STYLE_BLACK))?;
    let text_style = ("sans-serif", 16).into_font().color(&BLACK);
    root.draw_text(&freq.to_string(), &text_style, (x + 32, y + 48))?;
    match dots {
        0 => {},
        1 => {root.draw(&Circle::new((x + 35, y + 41), 5, style.clone()))?;root.draw(&Circle::new((x + 35, y + 41), 5, STYLE_BLACK))?;
        },
        2 => {root.draw(&Circle::new((x + 35, y + 68), 5, style.clone()))?;root.draw(&Circle::new((x + 35, y + 68), 5, STYLE_BLACK))?;
              root.draw(&Circle::new((x + 35, y + 41), 5, style.clone()))?;root.draw(&Circle::new((x + 35, y + 41), 5, STYLE_BLACK))?;
            },
        3 => {root.draw(&Circle::new((x + 35, y + 29), 5, style.clone()))?;root.draw(&Circle::new((x + 35, y + 29), 5, STYLE_BLACK))?;
              root.draw(&Circle::new((x + 35, y + 41), 5, style.clone()))?;root.draw(&Circle::new((x + 35, y + 41), 5, STYLE_BLACK))?;
              root.draw(&Circle::new((x + 35, y + 68), 5, style.clone()))?;root.draw(&Circle::new((x + 35, y + 68), 5, STYLE_BLACK))?;
            },
        4 => {root.draw(&Circle::new((x + 35, y + 29), 5, style.clone()))?;root.draw(&Circle::new((x + 35, y + 29), 5, STYLE_BLACK))?;
              root.draw(&Circle::new((x + 35, y + 41), 5, style.clone()))?;root.draw(&Circle::new((x + 35, y + 41), 5, STYLE_BLACK))?;
              root.draw(&Circle::new((x + 35, y + 68), 5, style.clone()))?;root.draw(&Circle::new((x + 35, y + 68), 5, STYLE_BLACK))?;
              root.draw(&Circle::new((x + 35, y + 80), 5, style.clone()))?;root.draw(&Circle::new((x + 35, y + 80), 5, STYLE_BLACK))?;
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
