use clap::Parser;
use hyprland::{
    data::{Monitors, Workspaces},
    event_listener::EventListener,
    shared::HyprData,
};
use serde_json::{Value, json};

#[derive(Parser, Debug)]

struct Cli {
    monitor: String,
}

fn main() {
    let cli = Cli::parse();
    let monitor_name = cli.monitor;
    println!("{}", get_workspaces(&monitor_name));

    let mut listener = EventListener::new();
    listener.add_workspace_changed_handler(move |_| {
        println!("{}", get_workspaces(&monitor_name));
    });
    listener.start_listener().expect("Failed to start listener");
}

fn get_monitor(monitor_name: &str) -> hyprland::data::Monitor {
    Monitors::get()
        .expect("Failed to get monitors")
        .into_iter()
        .find(|mon| mon.name == monitor_name)
        .expect("Monitor not found")
}

fn get_workspaces(monitor_name: &str) -> Value {
    let monitor = get_monitor(monitor_name);
    let active_ws_id = monitor.active_workspace.id;
    let monitor_id = monitor.id;
    let wsps = Workspaces::get().expect("Failed to get workspaces");
    let mut wsps = wsps
        .iter()
        .filter(|ws| ws.monitor_id == Some(monitor_id))
        .collect::<Vec<_>>();
    wsps.sort_by_key(|ws| ws.id);
    json!(
        wsps.into_iter()
            .filter_map(|ws| match (ws.id, ws.id == active_ws_id) {
                (10, false) => Some("󰎣"),
                (1, false) => Some("󰎦"),
                (2, false) => Some("󰎩"),
                (3, false) => Some("󰎬"),
                (4, false) => Some("󰎮"),
                (5, false) => Some("󰎰"),
                (6, false) => Some("󰎵"),
                (7, false) => Some("󰎸"),
                (8, false) => Some("󰎻"),
                (9, false) => Some("󰎾"),
                (10, true) => Some("󰎡"),
                (1, true) => Some("󰎤"),
                (2, true) => Some("󰎧"),
                (3, true) => Some("󰎪"),
                (4, true) => Some("󰎭"),
                (5, true) => Some("󰎱"),
                (6, true) => Some("󰎳"),
                (7, true) => Some("󰎶"),
                (8, true) => Some("󰎹"),
                (9, true) => Some("󰎼"),
                _ => None,
            })
            .collect::<Vec<&str>>()
    )
}
