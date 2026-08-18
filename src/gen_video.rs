use execute::Execute;
use std::process::Command;
use crate::db_ops::get_first_year;
use log::{error, warn, info, debug, trace, log_enabled, Level};

pub async fn generate_videos_from_charts(the_city: &str) -> Result<(), sqlx::Error> {
    let first_year: i32 = get_first_year(&the_city).await;
    info!("Generating videos for the city of {0} starting in {1}", the_city, first_year);
    
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

    if log_enabled!(Level::Trace) {
        trace!("Trace level is active (most detailed)");
        debug!("debug level is active (lots of details)");
        info!("info is active (std setting)");
        warn!("warning level is set (typical for production)");
        error!("only errors reported (for mature production only)");
    }
    
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

    info!("Executing command: {:?}", cmd);
    let _ = cmd.execute_output().unwrap();

    period = "week";
    //let frames_per_second = fps.to_string();
    let mut cmd2 = Command::new(FFMPEG_PATH);
    cmd2.arg(HIDE_BANNER).arg(OVERWRITE).arg(START_NUMBER).arg(start_year.to_string())
    .arg(FRAMERATE).arg(&frames_per_second);
    let input_arg = format!("{}{}_%4d_{}.png", PNG_FOLDER, city, period);
    cmd2.arg("-i").arg(&input_arg).arg("-c:v").arg("libx264");
    output_arg = format!("{}{}_{}_{}.mp4", VIDEO_FOLDER, city, period, &frames_per_second);
    cmd2.arg(&output_arg);

    info!("Executing command: {:?}", cmd2);
    let _ = cmd2.execute_output().unwrap();

    period = "fort";
    //let frames_per_second = fps.to_string();
    let mut cmd3 = Command::new(FFMPEG_PATH);
    cmd3.arg(HIDE_BANNER).arg(OVERWRITE).arg(START_NUMBER).arg(start_year.to_string())
    .arg(FRAMERATE).arg(&frames_per_second);
    let input_arg = format!("{}{}_%4d_{}.png", PNG_FOLDER, city, period);
    cmd3.arg("-i").arg(&input_arg).arg("-c:v").arg("libx264");
    output_arg = format!("{}{}_{}_{}.mp4", VIDEO_FOLDER, city, period, &frames_per_second);
    cmd3.arg(&output_arg);

    info!("Executing command: {:?}", cmd3);
    let output = cmd3.execute_output().unwrap();

    if let Some(exit_code) = output.status.code() {
        if exit_code == 0 {
            info!("The weekly exit code is 0 (Ok)");
        } else {
            error!("Error executing `{}` with in-file: {} and out-file: {}", FFMPEG_PATH, input_arg, output_arg);
        }
    } 

    Ok(())
}

// these are ffmpeg related consts
const FFMPEG_PATH: &str = "C:\\ProgramData\\chocolatey\\bin\\ffmpeg.exe";
const PNG_FOLDER: &str = "bar_pngs\\";
//const PNG_FOLDER: &str = "line_pngs\\"; 
const VIDEO_FOLDER: &str = "videos\\";
const HIDE_BANNER: &str = "-hide_banner";
const START_NUMBER: &str = "-start_number";  //start_year = first_year
const FRAMERATE: &str = "-framerate";
const OVERWRITE: &str = "-y";
