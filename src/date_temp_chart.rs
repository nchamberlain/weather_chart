use sqlx::Row;
//use sqlx::mysql::MySqlRow;
use std::collections::HashMap;
use log::{error, warn, info, debug, trace, log_enabled, Level};
use plotters::{prelude::*, style::RGBColor};
//use plotters::coord::Shift;
//use crate::db_ops::*;
use chrono::{NaiveDate, Months};

pub async fn generate_date_temp_charts(the_city: &str) -> Result<(), sqlx::Error> {
    let city  = String::from(the_city);
    for chart_type in 1..=4 { //1=high-Max, 2=hig-min, 3=low-max, 4=low-min
        make_date_time_charts(&city, chart_type).await?;    
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
async fn make_date_time_charts(city: &str, chart_type: i32) -> Result<(), sqlx::Error> {
    let mut chart_title = String::new();
    let mut out_file_name = String::new();
    let mut query_string = String::new();//format!("SELECT tdate, tmax, tmin FROM {} ORDER BY tdate", city);
    let mut the_color: RGBColor;
    let query_limit = 10000;
    //let city_fmt: String = city.replace("_", " ");
    match chart_type {
        1 => {debug!("Generating High-Max charts for {city}");
                chart_title = format!("{}: {} Hottest Daytime Temperatures", city.replace("_", " "), query_limit);
                query_string = format!("SELECT tdate, tmax FROM {} where tmax is not null ORDER BY tmax DESC, tdate ASC LIMIT {}", city, query_limit);
                out_file_name = format!("date_charts/{}_high_max.svg", city);
                the_color = COLOR_HOT;
            },
        2 => {debug!("Generating High-Min charts for {city}");
                chart_title = format!("{}: {} Coolest Daytime Temperatures", city.replace("_", " "), query_limit);
                query_string = format!("SELECT tdate, tmax FROM {} where tmax is not null ORDER BY tmax ASC, tdate ASC LIMIT {}", city, query_limit);
                out_file_name = format!("date_charts/{}_high_min.svg", city);
                the_color = COLOR_WARM;
            },
        3 => {debug!("Generating Low-Max charts for {city}");
                chart_title = format!("{}: {} Coldest Nighttime Temperatures", city.replace("_", " "), query_limit);
                query_string = format!("SELECT tdate, tmin FROM {} where tmin is not null ORDER BY tmin ASC, tdate ASC LIMIT {}", city, query_limit);
                out_file_name = format!("date_charts/{}_low_max.svg", city);
                the_color = COLOR_COLD;
            },
        4 => {debug!("Generating Low-Min charts for {city}");
                chart_title = format!("{}: {} Warmest Nighttime Temperatures", city.replace("_", " "), query_limit);
                query_string = format!("SELECT tdate, tmin FROM {} where tmin is not null ORDER BY tmin DESC, tdate ASC LIMIT {}", city, query_limit);
                out_file_name = format!("date_charts/{}_low_min.svg", city);
                the_color = COLOR_COOL;
            },
        _ => {the_color = COLOR_HOT; error!("Unknown chart type");},
    }
    // this db call should really be in db_ops but is so specialized it doesn't seem worth the effort to move
    let rows: Vec<sqlx::mysql::MySqlRow> = sqlx::query(&query_string)
        .fetch_all(crate::DB_POOL.get().expect("Database pool not initialized"))
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

    let years: u32 = latest_date.years_since(earliest_date).unwrap();
    let width = years * 60 + 180;
    info!("Width {width} for {earliest_date} to {latest_date}");

    let root = SVGBackend::new(&out_file_name, (width, 1920)).into_drawing_area();
    let _ = root.fill(&WHITE);
    let mut chart = ChartBuilder::on(&root)
        .margin(10)
        .caption(full_chart_title, ("sans-serif", 48),)
        .set_label_area_size(LabelAreaPosition::Left, 100)
        .set_label_area_size(LabelAreaPosition::Right, 100)
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
        .label_style(("sans-serif", 36))
        .max_light_lines(4)
        //.y_label_style(("sans-serif", 16))
        .y_desc("Average Temp (F)")
        .draw().unwrap();
    chart
        .configure_secondary_axes()
        .label_style(("sans-serif", 36))
        .y_desc("Average Temp (C)")
        .draw().unwrap();

    chart.draw_series(
        rows.iter()
            .map(|row: &sqlx::mysql::MySqlRow| {
                let date_str: &str = row.get(0);
                let temp: i32 = row.get(1);
                Circle::new(
                (NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap(), temp as f64),
                     2, the_color.filled())})
            ).expect("Error drawing series");
    // display dup data for diagnostic purposes
    if log_enabled!(Level::Debug) {
        for item in &dup_dates {
            debug!("Duplicate date found: {} for temp: {} count: {}", item.0, item.1.0, item.1.1);
        }
    }
    chart.draw_series(PointSeries::of_element(dup_dates, 4, the_color.mix(0.6).filled(),  
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
fn f_to_c(num: f64) -> f64 {
    let result = (num - 32.0) * 5.0 / 9.0;
    debug!("\nC = {}", result);
    result
}

const COLOR_HOT: RGBColor = RGBColor(220, 20, 60);
const COLOR_WARM: RGBColor = RGBColor(255, 69, 0);
const COLOR_COOL: RGBColor = RGBColor(0, 128, 0);
const COLOR_COLD: RGBColor = RGBColor(0, 0, 205);

//const COLOR_100S: RGBAColor = RGBAColor(255, 0, 0, 0.9);
//const COLOR_HOT: RGBAColor = RGBAColor(220, 20, 60, 0.88);
//const COLOR_WARM: RGBAColor = RGBAColor(255, 69, 0, 0.7);
//const COLOR_70S: RGBAColor = RGBAColor(50, 205, 51, 0.7);
//const COLOR_COOL: RGBAColor = RGBAColor(0, 128, 0, 0.9);
//const COLOR_50S: RGBAColor = RGBAColor(0, 139, 128, 0.6);
//const COLOR_COLD: RGBAColor = RGBAColor(70, 130, 132, 0.95);
//const COLOR_30S: RGBAColor = RGBAColor(0, 0, 205, 0.7);
//const COLOR_FREEZING: RGBAColor = RGBAColor(0, 0, 128, 0.95);
//const COLOR_WHITE: RGBAColor = RGBAColor(250, 250, 250, 0.2);
//const COLOR_BLACK: RGBAColor = RGBAColor(0, 0, 0, 0.99);

