use plotters::{prelude::*, style::RGBAColor};
use plotters::coord::Shift;
use log::{error, warn, info, debug, trace, log_enabled, Level};
use sqlx::mysql::MySqlRow;
use sqlx::Row;
use chrono::NaiveDate;

// This funct will do either daytime or nighttime charts depending on the rows that are passed to it
pub fn draw_freq_chart(the_city: &str, first_year: i32, last_year: i32, rows: Vec<MySqlRow>, is_day: bool) -> Result<(), Box<dyn std::error::Error>>{
    info!("Drawing frequency chart for {the_city} from {first_year} to {last_year}");
    // column width should some day also be based on how best to fit the chart width 
    let years: i32 = rows.len() as i32;
    let width: i32 = (years * 60) + 80 + 80;
    let city_title = the_city.replace("_", " ");
    let city_path: String;
    info!("Freq Chart for {city_title}: width = {width} years = {years}");
    if is_day {
        city_path = format!("freq_charts/{}_range_freq.svg", the_city);
    } else {
        city_path = format!("freq_charts/{}_nite_range_freq.svg", the_city);
    }
    let root = SVGBackend::new(&city_path, (width as u32, 2160)).into_drawing_area();
    root.fill(&WHITE).expect("Failed to fill the drawing area");

    let areas = root.split_by_breakpoints(
        [100, (width - 100)],                                //makes 3 columns
        [100, 310, 520, 730, 940, 1150, 1360, 1570, 1780, 1990], //makes 11 rows
    );
    // title row areas[0] areas[1] areas[2]

    // temperature row 1
    areas[3].fill(&BGSTYLE_100S)?;
    areas[4].fill(&BGSTYLE_100S)?;
    areas[5].fill(&BGSTYLE_100S)?;
    // temperature row 2
    areas[6].fill(&BGSTYLE_90S)?;
    areas[7].fill(&BGSTYLE_90S)?;
    areas[8].fill(&BGSTYLE_90S)?;
    // temperature row 3
    areas[9].fill(&BGSTYLE_80S)?;
    areas[10].fill(&BGSTYLE_80S)?;
    areas[11].fill(&BGSTYLE_80S)?;
    // temperature row 4
    areas[12].fill(&BGSTYLE_70S)?;
    areas[13].fill(&BGSTYLE_70S)?;
    areas[14].fill(&BGSTYLE_70S)?;
    // temperature row 5
    areas[15].fill(&BGSTYLE_60S)?;
    areas[16].fill(&BGSTYLE_60S)?;
    areas[17].fill(&BGSTYLE_60S)?;
    // temperature row 6
    areas[18].fill(&BGSTYLE_50S)?;
    areas[19].fill(&BGSTYLE_50S)?;
    areas[20].fill(&BGSTYLE_50S)?;
    // temperature row 7
    areas[21].fill(&BGSTYLE_40S)?;
    areas[22].fill(&BGSTYLE_40S)?;
    areas[23].fill(&BGSTYLE_40S)?;
    // temperature row 8
    areas[24].fill(&BGSTYLE_30S)?;
    areas[25].fill(&BGSTYLE_30S)?;
    areas[26].fill(&BGSTYLE_30S)?;
    // temperature row 9
    areas[27].fill(&BGSTYLE_FREEZING)?;
    areas[28].fill(&BGSTYLE_FREEZING)?;
    areas[29].fill(&BGSTYLE_FREEZING)?;
    // footer row areas[30] areas[31] areas[32]

    // Title caption area
    root.draw(&Rectangle::new([(1, 98), (width, 101)], STYLE_H_LINE))?;
    // 100's area
    root.draw(&Rectangle::new([(1, 308), (width, 310)], STYLE_H_LINE))?;
    //90's area
    root.draw(&Rectangle::new([(1, 518), (width, 520)], STYLE_H_LINE))?;
    //80's area
    root.draw(&Rectangle::new([(1, 728), (width, 730)], STYLE_H_LINE))?;
    //70's area
    root.draw(&Rectangle::new([(1, 938), (width, 940)], STYLE_H_LINE))?;
    //60's area
    root.draw(&Rectangle::new([(1, 1148), (width, 1150)], STYLE_H_LINE))?;
    //50's area
    root.draw(&Rectangle::new([(1, 1358), (width, 1360)], STYLE_H_LINE))?;
    //40's area
    root.draw(&Rectangle::new([(1, 1568), (width, 1570)], STYLE_H_LINE))?;
    //30's area
    root.draw(&Rectangle::new([(1, 1778), (width, 1780)], STYLE_H_LINE))?;
    //freezing
    root.draw(&Rectangle::new([(1, 1988), (width, 1990)], STYLE_H_LINE))?;
    //footer year numbers area
    // draw vertical lines
    root.draw(&Rectangle::new([(100, 100), (101, 1990)], STYLE_BLACK))?;
    root.draw(&Rectangle::new(
        [(width - 99, 100), (width - 100, 1990)],
        STYLE_BLACK,
    ))?;

    let text_style = ("sans-serif", 24).into_font().color(&BLACK);
    //print the fahrenheit temps
    root.draw_text("100 \u{2191}", &text_style, (20, 200))?;
    root.draw_text("90 - 99", &text_style, (20, 410))?;
    root.draw_text("80 - 89", &text_style, (20, 620))?;
    root.draw_text("70 - 79", &text_style, (20, 830))?;
    root.draw_text("60 - 69", &text_style, (20, 1040))?;
    root.draw_text("50 - 59", &text_style, (20, 1250))?;
    root.draw_text("40 - 49", &text_style, (20, 1460))?;
    root.draw_text("33 - 39", &text_style, (20, 1670))?;
    root.draw_text("32 \u{2193}", &text_style, (20, 1880))?;
    //print the celcius temps
    root.draw_text("38 \u{2191}", &text_style, (width - 80, 200))?;
    root.draw_text("32 - 37", &text_style, (width - 80, 410))?;
    root.draw_text("27 - 32", &text_style, (width - 80, 620))?;
    root.draw_text("21 - 26", &text_style, (width - 80, 830))?;
    root.draw_text("16 - 20", &text_style, (width - 80, 1040))?;
    root.draw_text("10 - 15", &text_style, (width - 80, 1250))?;
    root.draw_text("4 - 9", &text_style, (width - 80, 1460))?;
    root.draw_text("0 - 4", &text_style, (width - 80, 1670))?;
    root.draw_text("0 \u{2193}", &text_style, (width - 80, 1880))?;
    // draw caption (Chart Title)
    let title = format!(
        "Daytime Temperature Range Frequency for {city_title} from {first_year} to {last_year}"
    );
    let mut chart_area1 = ChartBuilder::on(&areas[1])
        .margin(40)
        .caption(title, ("sans-serif", 40))
        .build_cartesian_2d(0i32..1i32, 0i32..1i32)?;
    chart_area1
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()?;

    // Draw temp range series for 100+ temps
    draw_freq_boxes(100, first_year, last_year, &rows, areas.clone())
        .expect("Failed to draw 100's boxes");

    // Draw temp range series for 90+ temps
    draw_freq_boxes(90, first_year, last_year, &rows, areas.clone())
        .expect("Failed to draw 90's boxes");

    // Draw temp range series for 80+ temps
    draw_freq_boxes(80, first_year, last_year, &rows, areas.clone())
        .expect("Failed to draw 80's boxes");

    // Draw temp range series for 70+ temps
    draw_freq_boxes(70, first_year, last_year, &rows, areas.clone())
        .expect("Failed to draw 70's boxes");

    // Draw temp range series for 60+ temps
    draw_freq_boxes(60, first_year, last_year, &rows, areas.clone())
        .expect("Failed to draw 60's boxes");

    // Draw temp range series for 50+ temps
    draw_freq_boxes(50, first_year, last_year, &rows, areas.clone())
        .expect("Failed to draw 50's boxes");

    // Draw temp range series for 40+ temps
    draw_freq_boxes(40, first_year, last_year, &rows, areas.clone())
        .expect("Failed to draw 40's boxes");

    // Draw temp range series for 30+ temps
    draw_freq_boxes(30, first_year, last_year, &rows, areas.clone())
        .expect("Failed to draw 30's boxes");

    // Draw temp range series for freezing temps and below
    draw_freq_boxes(-1, first_year, last_year, &rows, areas).expect("failed to draw freezing boxes");

    // Draw dates beneath columns in footer
    let x_label_style = ("sans-serif", 20).into_font().color(&BLACK);
    let dyn_col_width: f64 = (width as f64 - 260.0) / years as f64;
    debug!("Dyn Col Width: {} ", dyn_col_width);
    let mut j = 0;
    for i in (first_year..last_year).step_by(2) {
        let x_offset = 130.0 + ((dyn_col_width * 2.0) * j as f64);
        j += 1;
        let year_label = format!("{}", i);
        root.draw_text(&year_label, &x_label_style, (x_offset.round() as i32, 2020))?;
        debug!(
            "x_offset: {:.2}  x_offset.round(): {}",
            x_offset,
            x_offset.round()
        );
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
fn make_range(rows: &Vec<sqlx::mysql::MySqlRow>, temps: i32) -> Vec<(i32, i32)> {
    let out_vec: Vec<(i32, i32)>;
    let years: Vec<i32> = rows
        .iter()
        .map(|a_row: &MySqlRow| a_row.try_get("tyear").unwrap_or(0))
        .collect();
    let range_freqs: Vec<i32>;
    match temps {
        100 => {
            range_freqs = rows
                .iter()
                .map(|row: &MySqlRow| row.try_get("count_100s").unwrap_or(0))
                .collect();
        }
        90 => {
            range_freqs = rows
                .iter()
                .map(|row: &MySqlRow| row.try_get("count_90s").unwrap_or(0))
                .collect();
        }
        80 => {
            range_freqs = rows
                .iter()
                .map(|row: &MySqlRow| row.try_get("count_80s").unwrap_or(0))
                .collect();
        }
        70 => {
            range_freqs = rows
                .iter()
                .map(|row: &MySqlRow| row.try_get("count_70s").unwrap_or(0))
                .collect();
        }
        60 => {
            range_freqs = rows
                .iter()
                .map(|row: &MySqlRow| row.try_get("count_60s").unwrap_or(0))
                .collect();
        }
        50 => {
            range_freqs = rows
                .iter()
                .map(|row: &MySqlRow| row.try_get("count_50s").unwrap_or(0))
                .collect();
        }
        40 => {
            range_freqs = rows
                .iter()
                .map(|row: &MySqlRow| row.try_get("count_40s").unwrap_or(0))
                .collect();
        }
        30 => {
            range_freqs = rows
                .iter()
                .map(|row: &MySqlRow| row.try_get("count_30s").unwrap_or(0))
                .collect();
        }
        _ => {
            range_freqs = rows
                .iter()
                .map(|row: &MySqlRow| row.try_get("count_freezing").unwrap_or(0))
                .collect();
        }
    }
    out_vec = years
        .iter()
        .zip(range_freqs.iter())
        .map(|(&a, &b)| (a, b))
        .collect();
    debug!("out_vec {:?}", out_vec);
    out_vec
}

fn draw_freq_boxes(
    the_range: i32,
    first_year: i32,
    last_year: i32,
    rows: &Vec<sqlx::mysql::MySqlRow>,
    areas: Vec<DrawingArea<SVGBackend<'_>, Shift>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Draw temp range series for freezing temps and below
    let area_nbr: usize;
    let the_color: RGBAColor;
    let the_style: ShapeStyle;
    match the_range {
        100 => {
            area_nbr = 4;
            the_color = COLOR_100S;
            the_style = STYLE_100S;
        }
        90 => {
            area_nbr = 7;
            the_color = COLOR_90S;
            the_style = STYLE_90S;
        }
        80 => {
            area_nbr = 10;
            the_color = COLOR_80S;
            the_style = STYLE_80S;
        }
        70 => {
            area_nbr = 13;
            the_color = COLOR_70S;
            the_style = STYLE_70S;
        }
        60 => {
            area_nbr = 16;
            the_color = COLOR_60S;
            the_style = STYLE_60S;
        }
        50 => {
            area_nbr = 19;
            the_color = COLOR_50S;
            the_style = STYLE_50S;
        }
        40 => {
            area_nbr = 22;
            the_color = COLOR_40S;
            the_style = STYLE_40S;
        }
        30 => {
            area_nbr = 25;
            the_color = COLOR_30S;
            the_style = STYLE_30S;
        }
        _ => {
            area_nbr = 28;
            the_color = COLOR_FREEZING;
            the_style = STYLE_FREEZING
        }
    }
    let frequency: Vec<(i32, i32)> = make_range(&rows, the_range); //get the freezing data
    let mut the_chart_area = ChartBuilder::on(&areas[area_nbr]).build_cartesian_2d(
        (NaiveDate::from_ymd_opt(first_year - 1, 6, 1).unwrap()
            ..NaiveDate::from_ymd_opt(last_year + 1, 6, 1).unwrap())
            .yearly(),
        0..10,
    )?;
    the_chart_area
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()?;
    the_chart_area
        .draw_series(PointSeries::of_element(
            //draw points for high temps
            frequency,
            2, //size is not used for rectangle
            the_color,
            &|c, _s, _st| {
                let nd: NaiveDate = NaiveDate::from_ymd_opt(c.0, 1, 1).unwrap();
                let f: i32 = c.1;
                return EmptyElement::at((nd,0))    // We want to construct a composed element on-the-fly
            + Rectangle::new([(0,1), (30,-208)], STYLE_WHITE) 
            + Rectangle::new([(0,1), (30,-4 * f)], the_style) 
            + Rectangle::new([(0,1), (30,-208)], STYLE_BLACK) 
            + Text::new(format!("{:?}", f), (33, -80), ("sans-serif", 16).into_font());
            },
        ))
        .unwrap();
    Ok(())
}

const COLOR_100S: RGBAColor = RGBAColor(255, 0, 0, 0.9);
const STYLE_100S: ShapeStyle = ShapeStyle {
    // RED
    color: RGBAColor(255, 0, 0, 0.9),
    filled: true,
    stroke_width: 1,
};
const COLOR_90S: RGBAColor = RGBAColor(220, 20, 60, 0.88);
const STYLE_90S: ShapeStyle = ShapeStyle {
    // crimson
    color: RGBAColor(220, 20, 60, 0.88),
    filled: true,
    stroke_width: 1,
};
const COLOR_80S: RGBAColor = RGBAColor(255, 69, 0, 0.7);
const STYLE_80S: ShapeStyle = ShapeStyle {
    //orange red
    color: RGBAColor(255, 69, 0, 0.7),
    filled: true,
    stroke_width: 1,
};
const COLOR_70S: RGBAColor = RGBAColor(50, 205, 51, 0.7);
const STYLE_70S: ShapeStyle = ShapeStyle {
    //light green
    color: RGBAColor(50, 205, 51, 0.7),
    filled: true,
    stroke_width: 1,
};
const COLOR_60S: RGBAColor = RGBAColor(0, 128, 0, 0.9);
const STYLE_60S: ShapeStyle = ShapeStyle {
    //green
    color: RGBAColor(0, 128, 0, 0.9),
    filled: true,
    stroke_width: 1,
};
const COLOR_50S: RGBAColor = RGBAColor(0, 139, 128, 0.6);
const STYLE_50S: ShapeStyle = ShapeStyle {
    //Teal
    color: RGBAColor(0, 139, 128, 0.6),
    filled: true,
    stroke_width: 1,
};
const COLOR_40S: RGBAColor = RGBAColor(70, 130, 132, 0.95);
const STYLE_40S: ShapeStyle = ShapeStyle {
    //steel blue
    color: RGBAColor(70, 130, 132, 0.95),
    filled: true,
    stroke_width: 1,
};
const COLOR_30S: RGBAColor = RGBAColor(0, 0, 205, 0.7);
const STYLE_30S: ShapeStyle = ShapeStyle {
    //medium blue
    color: RGBAColor(0, 0, 205, 0.7),
    filled: true,
    stroke_width: 1,
};
const COLOR_FREEZING: RGBAColor = RGBAColor(0, 0, 128, 0.95);
const STYLE_FREEZING: ShapeStyle = ShapeStyle {
    // navy
    color: RGBAColor(0, 0, 128, 0.95),
    filled: true,
    stroke_width: 1,
};
//const COLOR_WHITE: RGBAColor = RGBAColor(250, 250, 250, 0.2);
const STYLE_WHITE: ShapeStyle = ShapeStyle {
    // black
    color: RGBAColor(250, 250, 250, 0.99),
    filled: true,
    stroke_width: 1,
};
//const COLOR_BLACK: RGBAColor = RGBAColor(0, 0, 0, 0.99);
const STYLE_BLACK: ShapeStyle = ShapeStyle {
    // black
    color: RGBAColor(0, 0, 0, 0.99),
    filled: false,
    stroke_width: 1,
};
const STYLE_H_LINE: ShapeStyle = ShapeStyle {
    // black
    color: RGBAColor(0, 0, 0, 0.25),
    filled: true,
    stroke_width: 3,
};
// ========= Background styles ================== \\
const BGSTYLE_100S: RGBAColor = RGBAColor(255, 0, 0, 0.15); //RED
const BGSTYLE_90S: RGBAColor = RGBAColor(220, 20, 60, 0.1); //CRIMSON
const BGSTYLE_80S: RGBAColor = RGBAColor(255, 69, 0, 0.1); //orange red
const BGSTYLE_70S: RGBAColor = RGBAColor(50, 205, 51, 0.15); //light green
const BGSTYLE_60S: RGBAColor = RGBAColor(0, 128, 0, 0.1); //green
const BGSTYLE_50S: RGBAColor = RGBAColor(0, 139, 128, 0.1); //Teal
const BGSTYLE_40S: RGBAColor = RGBAColor(70, 130, 132, 0.15); //steel blue
const BGSTYLE_30S: RGBAColor = RGBAColor(0, 0, 205, 0.1); //medium blue
const BGSTYLE_FREEZING: RGBAColor = RGBAColor(0, 0, 128, 0.15); // navy
