use plotters::prelude::*;
//use plotters::BitMapBackend::with_buffer;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let mut buffer = vec![0; 1024 * 768 * 3];
    let root = SVGBackend::new("plotters-doc-data/61-Fairb-30pt.svg", (1280, 960)).into_drawing_area();
        let _ = root.fill(&WHITE);
    let root = root.margin(10, 10, 10, 20);
    let mut chart = ChartBuilder::on(&root)
        .caption("1906 Fairbanks AK Fortnights", ("sans-serif", 30).into_font())
        .y_label_area_size(40)
        .x_label_area_size(40)
        .build_cartesian_2d((0..260).with_key_points(vec![0,65,130,195,260]),
         -65..99
        )
        .unwrap();
    let x_labels = ["", "Q1", "Q2", "Q3", "Q4"];
    chart.configure_mesh()
        .y_max_light_lines(5)// it still makes best guess at optimum number of minor lines, but it won't exceed 5
        .x_max_light_lines(5)
        .x_label_formatter(&|x: &i32| x_labels[(x / 65).min(4) as usize].to_string())
        //.y_label_formatter(&|x| format!("{:.3} ", x))
        .x_desc("X Axis Description")
        .draw().unwrap();

    chart.draw_series(LineSeries::new(
        vec![(5, -29),  (15,-36),   (25,12),   (35, 11),  (45,24), (55, 39),   (65,29),  (75, 40),  (85, 50),  (95, 57), (105, 73), (115, 74), (125, 72),
                   (135, 73), (145, 72), (155, 70), (165, 66), (175, 63), (185, 60), (195, 49), (205, 41), (215, 34), (225, 14), (235, -5), (245, -22), (255, 7)],
        &RED,
    )).unwrap();
    chart.draw_series(PointSeries::of_element(
        vec![ (5, -29),  (15,-36),   (25,12),   (35, 11),  (45,24), (55, 39),   (65,29),  (75, 40),  (85, 50),  (95, 57), (105, 73), (115, 74), (125, 72),
                    (135, 73), (145, 72), (155, 70), (165, 66), (175, 63), (185, 60), (195, 49), (205, 41), (215, 34), (225, 14), (235, -5), (245, -22), (255, 7)],
        2,
        &RED,
        &|c, s, st| {
            return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
            + Circle::new((0,0),s,st.filled()) // At this point, the new pixel coordinate is established
            + Text::new(format!("{:?}", c.1), (-4, -12), ("sans-serif", 10).into_font());
        },
    ))?;
    chart.draw_series(LineSeries::new(
        vec![ (5, -44),  (15, -46),  (25, -9),   (35, -15),  (45, -10), (55, 6),  (65, 3),  (75, 7),  (85, 29),  (95, 35), (105, 45), (115, 49), (125, 47),
                    (135, 47), (145, 50), (155, 44), (165, 43), (175, 38), (185, 30), (195, 25), (205, 24), (215, 14), (225, -8), (235, -24), (245, -37), (255, -10)],
        &BLUE,
    )).unwrap();
    chart.draw_series(PointSeries::of_element(
        vec![ (5, -44),  (15, -46),  (25, -9),   (35, -15),  (45, -10), (55, 6),  (65, 3),  (75, 7),  (85, 29),  (95, 35), (105, 45), (115, 49), (125, 47),
                    (135, 47), (145, 50), (155, 44), (165, 43), (175, 38), (185, 30), (195, 25), (205, 24), (215, 14), (225, -8), (235, -24), (245, -37), (255, -10)],
        2,
        &BLUE,
        &|c, s, st| {
            return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
            + Circle::new((0,0),s,st.filled()) // At this point, the new pixel coordinate is established
            + Text::new(format!("{:?}", c.1), (-4, -12), ("sans-serif", 10).into_font());
        },
    ))?;

    root.present().unwrap();
    Ok(())
}