use std::{
    io::BufRead,
    process::{Command, Stdio},
};

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Volume,
    Symbol,
}

fn main() {
    match Cli::parse().command {
        Commands::Volume => {
            println!("{}", get_curr_volume_str());
            on_device_change(get_curr_volume_str);
        }
        Commands::Symbol => {
            println!("{}", get_curr_symbol());
            on_device_change(get_curr_symbol);
        }
    }
}

fn on_device_change(cb: fn() -> String) {
    let mut pactl_sub = Command::new("pactl")
        .arg("subscribe")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute pactl subscribe");
    let stdout = pactl_sub.stdout.take().expect("Failed to get stdout");
    let reader = std::io::BufReader::new(stdout);
    let mut last_val = String::new();
    for line in reader.lines() {
        match line {
            Ok(line) if line.contains("Event 'change' on sink") => {
                let val = cb();
                if val != last_val {
                    println!("{val}");
                    last_val = val;
                }
            }
            Err(e) => eprintln!("Error reading line: {e}"),
            _ => continue,
        }
    }
    pactl_sub
        .wait()
        .expect("Failed to wait for pactl subscribe");
}

fn get_mute() -> bool {
    let output = Command::new("pamixer")
        .arg("--get-mute")
        .output()
        .expect("Failed to execute command");
    let is_mute = String::from_utf8_lossy(&output.stdout);
    is_mute.trim().parse().expect("Failed to parse mute state")
}
fn get_curr_symbol() -> String {
    let mute = get_mute();
    if mute {
        ""
    } else {
        match get_curr_volume() {
            0 => "",
            1..=49 => "",
            50..=100 => "",
            _ => "",
        }
    }
    .to_string()
}

fn get_curr_volume_str() -> String {
    get_curr_volume().to_string()
}

fn get_curr_volume() -> u8 {
    let output = Command::new("pamixer")
        .arg("--get-volume")
        .output()
        .expect("Failed to execute command");
    let vol = String::from_utf8_lossy(&output.stdout);
    vol.trim().parse().expect("Failed to parse volume")
}
