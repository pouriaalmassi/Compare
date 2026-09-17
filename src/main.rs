#![allow(unexpected_cfgs)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod app;
pub mod diff;
pub mod platform;
pub mod theme;
pub mod ui;

use app::App;
use iced::window;
use iced::Size;
use std::path::PathBuf;

fn main() -> iced::Result {
    platform::init();

    let mut initial_paths = Vec::new();
    for arg in std::env::args().skip(1) {
        if arg.starts_with("-psn_") {
            continue;
        }
        let p = PathBuf::from(arg);
        if p.exists() {
            initial_paths.push(p);
        }
    }

    if !initial_paths.is_empty() {
        platform::send_paths(initial_paths);
    }

    let icon = load_app_icon();

    iced::application("Compare", App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .window(window::Settings {
            size: Size::new(400.0, 400.0),
            position: window::Position::Centered,
            icon,
            ..Default::default()
        })
        .run()
}

fn load_app_icon() -> Option<window::Icon> {
    const ICON_BYTES: &[u8] = include_bytes!("../assets/icon-64.rgba");
    window::icon::from_rgba(ICON_BYTES.to_vec(), 64, 64).ok()
}
