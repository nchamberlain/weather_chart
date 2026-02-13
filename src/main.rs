use colorize::AnsiColor;
use inquire::{validator::Validation, CustomUserError};

fn main() {
    //prompt_text();
    //prompt_select();
    prompt_multiselect();
}

fn prompt_multiselect() {
    let cities = vec![
        "Billings_MT",
        "Bismarck_ND",
        "Chicago_IL",
        "Jacksonville_FL",
        "Los_Angeles_CA" ,
        "New_York_NY",
        "Phoenix_AZ",
        "San_Francisco_CA",
        "San_Diego_CA",
        "Seattle_WA",
        "Spokane_WA",
    ];

    let prompt_message = "Please select the desired cities";
    let selected_cities = inquire::MultiSelect::new(&prompt_message, cities)
        .prompt()
        .expect("Failed to select cities");
    for selected_item in selected_cities {
        println!("You selected the city of {0}", selected_item);
    }
}

fn prompt_select() {
    let choices = vec![
        "Billings_MT",
        "Chicago_IL",
        "Jacksonville_FL",
        "Los_Angeles_CA",
        "Spokane_WA",
    ];

    let prompt_message = "Please select your player class".blue();
    let select = inquire::Select::new(&prompt_message, choices)
    .prompt()
    .expect("Failed to select a city");

    println!("You selected the city of {0}", select.yellow());
}
fn prompt_text() {
        let name_validator = |i: &str| -> Result<Validation, CustomUserError> {
        let first_char = i.chars().next().unwrap() as u8; //get the first char
        match first_char {
            65..=90 => {
                return Ok(Validation::Valid);
            }
            _ => {
                return Ok(Validation::Invalid("First Character must b a capital".into()));
            }   
        }
    };

    let prompt_message = "What is your player name?".blue();
    let player_name = inquire::Text::new(&prompt_message)
    .with_validator(name_validator)
    .prompt()
    .expect("Failed to capture player name");

    println!("Your name is {player_name}");

}
fn prompt_boolean() {
    let proceed = inquire::prompt_confirmation("Are you ready?");

    if proceed.unwrap() {
        println!("{0}", "User selected to proceed".green());
    } else {
        println!("{0}","User doesn't want to proceed".red());
    }
}