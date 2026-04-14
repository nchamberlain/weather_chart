// this code came from \users\cofee\rust-plotters\src\main.rs where I was trying to learn
// how to use the build-in plotters features to make a line chart.

use plotters::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let mut buffer = vec![0; 1024 * 768 * 3];
    let month_positions = [ 26,  78, 130, 182, 234, 286, 338, 390, 442, 494, 546, 598];
    let month_highs = [ 74, 61, 61, 62, 65, 72, 72, 70, 74, 76, 75, 75];
    let month_lows  = [ 33, 30, 28, 36, 31, 38, 36, 43, 42, 50, 48, 45];

    let mut month_pos_hi = month_positions.iter().zip(month_highs.iter());
    let mut month_pos_lo = month_positions.iter().zip(month_lows.iter());
    for val in month_pos_hi.by_ref() {
         println!("MONTH positon {}, high {}", val.0, val.1);
    }
    for val in month_pos_lo.by_ref() {
         println!("month positon {}, low {}", val.0, val.1);
    }

    /*let fort_positions  = [ 14,  38,  62,  86, 110, 134, 158, 182, 206, 230, 254, 278, 302, 
                                      326, 350, 374, 398, 422, 446, 470, 944, 518, 542, 566, 590, 614];
    let fort_highs = [ 74, 61, 61, 62, 65, 72, 72, 70, 74, 76, 75, 75, 84, 82, 84, 83, 89, 91, 87, 92, 92, 91,106,108,106,107];
    let fort_lows =  [ 33, 30, 28, 36, 31, 38, 36, 43, 42, 50, 48, 45, 50, 46, 50, 48, 51, 57, 51, 53, 53, 52, 65, 75, 75, 75];

    let mut fort_pos_hi = fort_positions.iter().zip(fort_highs.iter());
    let mut fort_pos_lo = fort_positions.iter().zip(fort_lows.iter());
    for val in fort_pos_hi.by_ref() {
         println!("FORT positon {}, high {}", val.0, val.1);
    }
    for val in fort_pos_lo.by_ref() {
         println!("Fort positon {}, low {}", val.0, val.1);
    }*/

    let week_positions  = [  4,  14,  26,  38,  50,  62,  74,  86,  98, 110, 122, 134, 146, 158, 170, 182, 194, 206, 218, 230, 242, 254, 266, 278, 290, 302, 
                                      314, 326, 338, 350, 362, 374, 386, 398, 410, 422, 434, 446, 458, 470, 482, 494, 506, 518, 530, 542, 554, 566, 578, 590, 602, 614]; 
    /*let week_highs      = [ 74,  61,  61,  62,  65,  72,  72,  70,  74,  76,  75,  75,  84,  82,  84,  83,  89,  91,  87,  92,  92,  91, 106, 108, 106, 107,
                                      108, 106, 108,  90,  97, 101, 104,  97, 100, 100,  94,  81,  93,  83,  77,  83,  75,  78,  69,  57,  72,  64,  60,  58,  50,  48];
    let week_lows       = [ 33,  30,  28,  36,  31,  38,  36,  43,  42,  50,  48,  45,  50,  46,  50,  48,  51,  57,  51,  53,  53,  52,  65,  75,  75,  75, 
                                       75,  71,  72,  69,  75,  69,  55,  53,  44,  70,  70,  66,  65,  64,  57,  61,  57,  50,  46,  46,  37,  48,  40,  32,  38,  34];

    let mut week_pos_hi = week_positions.iter().zip(week_highs.iter());
    let mut week_pos_lo = week_positions.iter().zip(week_lows.iter());
    for val in week_pos_hi.by_ref() {
         println!("WEEK positon {}, high {}", val.0, val.1);
    }
    for val in week_pos_lo.by_ref() {
         println!("Week positon {}, low {}", val.0, val.1);
    }
*/
    let root = SVGBackend::new("plotters-doc-data/Phoenix-1918-week.svg", (1280, 960)).into_drawing_area();
    let _ = root.fill(&WHITE);
    let root = root.margin(10, 10, 10, 20);
    let mut chart = ChartBuilder::on(&root)
        .caption("1918 Phoenix Az Weekly", ("sans-serif", 30).into_font())
        .y_label_area_size(48)
        .x_label_area_size(64)
        //.build_cartesian_2d((0..260).with_key_points(vec![0,65,130,195,260]),
        .build_cartesian_2d((0..624).with_key_points(vec![0,52,104,156,208,260,312,364,416,468,520,572,624]), //624 = 24 * 26 or 12 * 52 
         17..122
        )
        .unwrap();
    let x_labels = ["","<- Jan | Feb -","- Feb | Mar -","- Mar | Apr -","- Apr | May -","- May | Jun -","- Jun | Jul -","- Jul | Aug -","- Aug | Sep -","- Sep | Oct -","- Oct | Nov -","- Nov | Dec -","- Dec | -"];
    chart.configure_mesh()
        .y_max_light_lines(5)// it still makes best guess at optimum number of minor lines, but it won't exceed 5
        .y_label_style(("sans-serif", 20).into_font())
        //.x_max_light_lines(5)
        .x_label_style(("sans-serif", 20).into_font())
        .x_label_formatter(&|x: &i32| x_labels[(x / 52).min(12) as usize].to_string())
        //.y_label_formatter(&|x| format!("{:.3} ", x))
        .axis_desc_style(("sans-serif", 24).into_font())
        .x_desc("Months of the Year")
        .y_desc("Temperature (°F)")
        .draw().unwrap();
    // HIGH TEMPS
    let high_vec = vec![
            (  4, 74), ( 14, 61), ( 26, 61), ( 38, 62), ( 50, 65), ( 62, 72), ( 74, 72), ( 86, 70), ( 98, 74), (110, 76), (122, 75), (134, 75), (146, 84),
            (158, 82), (170, 84), (182, 83), (194, 89), (206, 91), (218, 87), (230, 92), (242, 92), (254, 91), (266,106), (278,108), (290,106), (302,107),
            (314,104), (326,100), (338,109), (350,106), (362,108), (374, 90), (386, 97), (398,101), (410,104), (422, 97), (434,100), (446,100), (458, 94),
            (470, 81), (482, 93), (494, 83), (506, 77), (518, 83), (530, 75), (542, 78), (554, 69), (566, 57), (578, 72), (590, 64), (602, 60), (614, 58),
            ];

    /*let high_temps = [
             74,  61,  61,  62,  65, 72, 72,  70,  74, 76,  75,  75, 84, 82, 84, 83, 89, 91, 87, 92, 92, 91, 106, 108, 106, 107,
            104, 100, 109, 106, 108, 90, 97, 101, 104, 97, 100, 100, 94, 81, 93, 83, 77, 83, 75, 78, 69, 57,  72,  64,  60,  58
            ];
    let high_vec: Vec<(i32, i32)> = week_positions.iter().zip(high_temps.iter()).map(|(&a, &b)| (a, b)).collect();*/

    chart.draw_series(LineSeries::new(
        high_vec.clone(),
        &RED,
    )).unwrap();
    chart.draw_series(PointSeries::of_element(
        high_vec,
        2,
        &RED,
        &|c, s, st| {
            return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
            + Circle::new((0,0),s,st.filled()) // At this point, the new pixel coordinate is established
            + Text::new(format!("{:?}", c.1), (-4, -12), ("sans-serif", 12).into_font()); //-4,-12 are fudges to position the text better
        },
    ))?;
    // LOW TEMPS
    let low_vec = vec![
        (  4, 33), ( 14, 30), ( 26, 28), ( 38, 36), ( 50, 31), ( 62, 38), ( 74, 36), ( 86, 43), ( 98, 42), (110, 50), (122, 48), (134, 45), (146, 50),
        (158, 46), (170, 50), (182, 48), (194, 51), (206, 57), (218, 51), (230, 53), (242, 53), (254, 52), (266, 65), (278, 75), (290, 75), (302, 75),
        (314, 75), (326, 71), (338, 72), (350, 69), (362, 75), (374, 69), (386, 63), (398, 70), (410, 70), (422, 66), (434, 65), (446, 64), (458, 57),
        (470, 61), (482, 57), (494, 50), (506, 46), (518, 46), (530, 37), (542, 48), (554, 40), (566, 32), (578, 38), (590, 34), (602, 33), (614, 29),
        ];
    chart.draw_series(LineSeries::new(
        low_vec.clone(),
        &BLUE,
    )).unwrap();

    chart.draw_series(PointSeries::of_element(
        low_vec,
        2,
        &BLUE,
        &|c, s, st| {
            return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
            + Circle::new((0,0),s,st.filled()) // At this point, the new pixel coordinate is established
            + Text::new(format!("{:?}", c.1), (-4, -12), ("sans-serif", 12).into_font());
        },
    ))?;

    root.present().unwrap();
    Ok(())
}