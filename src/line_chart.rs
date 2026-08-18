use sqlx::Row;
use log::{error, warn, info, debug, trace, log_enabled, Level};
use plotters::{prelude::*, style::RGBColor};
use crate::db_ops::*;

pub async fn make_monthly_charts(city: &str, first_year: i32, last_year: i32, city_low: i32, city_high: i32) -> Result<(), sqlx::Error> {
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
    if log_enabled!(Level::Trace) {
        trace!("Trace level is active (most detailed)");
        debug!("debug level is active (lots of details)");
        info!("info is active (std setting)");
        warn!("warning level is set (typical for production)");
        error!("only errors reported (for mature production only)");
    }
    Ok(())
}
pub async fn make_fortly_charts(city: &str, first_year: i32, last_year: i32, city_low: i32, city_high: i32) -> Result<(), sqlx::Error> {
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
pub async fn make_weekly_charts(city: &str, first_year: i32, last_year: i32, city_low: i32, city_high: i32) -> Result<(), sqlx::Error> {
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
