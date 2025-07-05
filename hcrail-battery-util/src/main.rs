use battery::{Manager, State};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Charge,
    Symbol,
}

fn main() {
    match Cli::parse().command {
        Commands::Charge => {
            on_poll(get_curr_charge);
        }
        Commands::Symbol => {
            on_poll(get_curr_symbol);
        }
    }
}

fn on_poll(cb: fn() -> String) {
    let mut last_val = String::new();
    loop {
        let val = cb();
        if val != last_val {
            println!("{val}");
            last_val = val;
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}

fn get_curr_symbol() -> String {
    match get_curr_state() {
        (0..10, State::Charging) => "󰢜",
        (10..20, State::Charging) => "󰂆",
        (20..30, State::Charging) => "󰂇",
        (30..40, State::Charging) => "󰂈",
        (40..50, State::Charging) => "󰢝",
        (50..60, State::Charging) => "󰂉",
        (60..70, State::Charging) => "󰢞",
        (70..80, State::Charging) => "󰂊",
        (80..90, State::Charging) => "󰂋",
        (90..100, State::Charging) => "󰂅",
        (0..10, _) => "󰁺",
        (10..20, _) => "󰁻",
        (20..30, _) => "󰁼",
        (30..40, _) => "󰁽",
        (40..50, _) => "󰁾",
        (50..60, _) => "󰁿",
        (60..70, _) => "󰂀",
        (70..80, _) => "󰂁",
        (80..90, _) => "󰂂",
        (90..=100, _) => "󰁹",
        _ => "󰂃",
    }
    .to_string()
}

fn get_curr_charge() -> String {
    let (charge, _) = get_curr_state();
    charge.to_string()
}

fn get_curr_state() -> (u8, State) {
    let mut batteries = Manager::new()
        .expect("Failed to create battery manager")
        .batteries()
        .expect("Failed to get batteries");
    let battery = batteries
        .next()
        .expect("No batteries found")
        .expect("Failed to get battery");
    (
        (battery.state_of_charge().value / 1.0 * 100.0).round() as u8,
        battery.state(),
    )
}
