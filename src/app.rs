use iced::event::{self, Event};
use iced::keyboard::{self, Key};
use iced::window;
use iced::{Element, Size, Subscription, Task, Theme};
use std::fs;
use std::path::PathBuf;

use crate::diff::{compute_diff, DiffData};
use crate::platform;
use crate::theme::{self, AppTheme};
use crate::ui::{view_diff, view_empty_state};

pub const DEFAULT_FONT_SIZE: f32 = 13.0;
pub const MIN_FONT_SIZE: f32 = 8.0;
pub const MAX_FONT_SIZE: f32 = 32.0;

#[derive(Debug, Clone)]
pub struct TextFile {
    #[allow(dead_code)]
    pub path: PathBuf,
    pub name: String,
    pub content: String,
}

pub struct App {
    pub file_1: Option<TextFile>,
    pub file_2: Option<TextFile>,
    pub diff_data: Option<DiffData>,
    pub wrap_lines: bool,
    pub is_hovering: bool,
    pub is_new_drop_sequence: bool,
    pub status_message: Option<String>,
    pub theme: AppTheme,
    pub is_os_dark: bool,
    pub font_size: f32,
    pub base_font_size: f32,
}

impl Default for App {
    fn default() -> Self {
        let config = theme::load_config();
        let is_os_dark = theme::is_system_dark();
        let app_theme = theme::load_theme_for_mode(is_os_dark);
        let base_font_size = config
            .font_size
            .unwrap_or(DEFAULT_FONT_SIZE)
            .clamp(MIN_FONT_SIZE, MAX_FONT_SIZE);

        Self {
            file_1: None,
            file_2: None,
            diff_data: None,
            wrap_lines: config.wrap_lines.unwrap_or(true),
            is_hovering: false,
            is_new_drop_sequence: false,
            status_message: None,
            theme: app_theme,
            is_os_dark,
            font_size: base_font_size,
            base_font_size,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    FileDropped(PathBuf),
    DockFilesDropped(Vec<PathBuf>),
    FileHovered,
    FilesHoveredLeft,
    ChooseFiles,
    ClearAll,
    SwapSides,
    ToggleWrapLines(bool),
    ZoomIn,
    ZoomOut,
    ResetZoom,
    CheckSystemTheme,
}

impl App {
    pub fn theme(&self) -> Theme {
        self.theme.iced_theme.clone()
    }

    pub fn maximize_window() -> Task<Message> {
        window::get_latest().then(|id| {
            if let Some(id) = id {
                window::maximize(id, true)
            } else {
                Task::none()
            }
        })
    }

    pub fn restore_window() -> Task<Message> {
        window::get_latest().then(|id| {
            if let Some(id) = id {
                Task::batch([
                    window::maximize(id, false),
                    window::resize(id, Size::new(400.0, 400.0)),
                ])
            } else {
                Task::none()
            }
        })
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DockFilesDropped(paths) => {
                self.is_hovering = false;
                self.is_new_drop_sequence = false;
                if paths.len() >= 2 {
                    self.file_1 = None;
                    self.file_2 = None;
                    self.diff_data = None;
                    self.status_message = None;
                    self.load_file(paths[0].clone());
                    self.load_file(paths[1].clone());
                } else if let Some(first) = paths.into_iter().next() {
                    self.load_file(first);
                }
                if self.file_1.is_some() || self.file_2.is_some() {
                    Self::maximize_window()
                } else {
                    Task::none()
                }
            }
            Message::FileHovered => {
                self.is_hovering = true;
                self.is_new_drop_sequence = true;
                Task::none()
            }
            Message::FilesHoveredLeft => {
                self.is_hovering = false;
                self.is_new_drop_sequence = false;
                Task::none()
            }
            Message::FileDropped(path) => {
                self.is_hovering = false;
                if self.is_new_drop_sequence && self.file_1.is_some() && self.file_2.is_some() {
                    self.file_1 = None;
                    self.file_2 = None;
                    self.diff_data = None;
                }
                self.is_new_drop_sequence = false;
                self.load_file(path);
                if self.file_1.is_some() || self.file_2.is_some() {
                    Self::maximize_window()
                } else {
                    Task::none()
                }
            }
            Message::ChooseFiles => {
                if let Some(paths) = rfd::FileDialog::new()
                    .set_title("Choose text files to compare")
                    .pick_files()
                {
                    for path in paths {
                        self.load_file(path);
                    }
                }
                if self.file_1.is_some() || self.file_2.is_some() {
                    Self::maximize_window()
                } else {
                    Task::none()
                }
            }
            Message::ClearAll => {
                self.file_1 = None;
                self.file_2 = None;
                self.diff_data = None;
                self.status_message = None;
                Self::restore_window()
            }
            Message::SwapSides => {
                std::mem::swap(&mut self.file_1, &mut self.file_2);
                self.update_diff();
                Task::none()
            }
            Message::ToggleWrapLines(enabled) => {
                self.wrap_lines = enabled;
                Task::none()
            }
            Message::ZoomIn => {
                self.font_size = (self.font_size + 1.0).min(MAX_FONT_SIZE);
                Task::none()
            }
            Message::ZoomOut => {
                self.font_size = (self.font_size - 1.0).max(MIN_FONT_SIZE);
                Task::none()
            }
            Message::ResetZoom => {
                self.font_size = self.base_font_size;
                Task::none()
            }
            Message::CheckSystemTheme => {
                let current_os_dark = theme::is_system_dark();
                if self.is_os_dark != current_os_dark {
                    self.is_os_dark = current_os_dark;
                    self.theme = theme::load_theme_for_mode(current_os_dark);
                }
                Task::none()
            }
        }
    }

    pub fn load_file(&mut self, path: PathBuf) {
        if path.is_dir() {
            self.status_message = Some(format!(
                "\"{}\" is a directory. Please choose plain text files.",
                path.display()
            ));
            return;
        }

        match fs::read_to_string(&path) {
            Ok(content) => {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                let text_file = TextFile {
                    path,
                    name,
                    content,
                };

                if self.file_1.is_none() {
                    self.file_1 = Some(text_file);
                } else {
                    self.file_2 = Some(text_file);
                }

                self.status_message = None;
                self.update_diff();
            }
            Err(err) => {
                let file_name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.display().to_string());
                self.status_message = Some(format!(
                    "Could not read \"{}\" as plain text: {}",
                    file_name, err
                ));
            }
        }
    }

    pub fn update_diff(&mut self) {
        if let (Some(f1), Some(f2)) = (&self.file_1, &self.file_2) {
            self.diff_data = Some(compute_diff(&f1.content, &f2.content));
        } else {
            self.diff_data = None;
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let window_events = event::listen_with(|event, _status, _window| match event {
            Event::Window(window::Event::FileDropped(path)) => {
                Some(Message::FileDropped(path))
            }
            Event::Window(window::Event::FileHovered(_)) => Some(Message::FileHovered),
            Event::Window(window::Event::FilesHoveredLeft) => {
                Some(Message::FilesHoveredLeft)
            }
            Event::Window(window::Event::Focused) => {
                Some(Message::CheckSystemTheme)
            }
            _ => None,
        });

        // Cross-platform zoom: `modifiers.command()` maps to ⌘ Command on macOS
        // and Ctrl on Windows/Linux.
        let key_events = keyboard::on_key_press(|key, modifiers| {
            if modifiers.command() {
                match key.as_ref() {
                    Key::Character("+" | "=") => Some(Message::ZoomIn),
                    Key::Character("-" | "_") => Some(Message::ZoomOut),
                    Key::Character("0") => Some(Message::ResetZoom),
                    _ => None,
                }
            } else {
                None
            }
        });

        let theme_timer = iced::time::every(std::time::Duration::from_millis(500))
            .map(|_| Message::CheckSystemTheme);

        Subscription::batch([
            window_events,
            key_events,
            theme_timer,
            platform::subscription(),
        ])
    }

    pub fn view(&self) -> Element<'_, Message> {
        let file_1_name = self.file_1.as_ref().map(|f| f.name.as_str());
        let file_2_name = self.file_2.as_ref().map(|f| f.name.as_str());

        if let Some(diff) = &self.diff_data {
            view_diff(
                diff,
                file_1_name,
                file_2_name,
                self.wrap_lines,
                self.is_hovering,
                self.status_message.as_deref(),
                &self.theme.diff,
                self.font_size,
            )
        } else {
            view_empty_state(
                file_1_name,
                self.is_hovering,
                self.status_message.as_deref(),
                &self.theme.diff,
                self.theme.is_dark,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle_wrap_lines_message() {
        let mut app = App {
            wrap_lines: true,
            ..Default::default()
        };
        assert!(app.wrap_lines);
        let _ = app.update(Message::ToggleWrapLines(false));
        assert!(!app.wrap_lines);
        let _ = app.update(Message::ToggleWrapLines(true));
        assert!(app.wrap_lines);
    }

    #[test]
    fn test_zoom_messages() {
        let mut app = App {
            font_size: DEFAULT_FONT_SIZE,
            base_font_size: DEFAULT_FONT_SIZE,
            ..Default::default()
        };
        assert_eq!(app.font_size, DEFAULT_FONT_SIZE);

        let _ = app.update(Message::ZoomIn);
        assert_eq!(app.font_size, DEFAULT_FONT_SIZE + 1.0);

        let _ = app.update(Message::ZoomOut);
        assert_eq!(app.font_size, DEFAULT_FONT_SIZE);

        let _ = app.update(Message::ZoomOut);
        assert_eq!(app.font_size, DEFAULT_FONT_SIZE - 1.0);

        let _ = app.update(Message::ResetZoom);
        assert_eq!(app.font_size, DEFAULT_FONT_SIZE);

        // Test clamping max
        app.font_size = MAX_FONT_SIZE;
        let _ = app.update(Message::ZoomIn);
        assert_eq!(app.font_size, MAX_FONT_SIZE);

        // Test clamping min
        app.font_size = MIN_FONT_SIZE;
        let _ = app.update(Message::ZoomOut);
        assert_eq!(app.font_size, MIN_FONT_SIZE);
    }

    #[test]
    fn test_custom_font_size_reset_zoom() {
        let mut app = App {
            font_size: 16.0,
            base_font_size: 16.0,
            ..Default::default()
        };
        let _ = app.update(Message::ZoomIn);
        assert_eq!(app.font_size, 17.0);
        let _ = app.update(Message::ResetZoom);
        assert_eq!(app.font_size, 16.0);
    }

    #[test]
    fn test_dock_files_dropped_two_files() {
        let mut app = App::default();
        let p1 = PathBuf::from("Cargo.toml");
        let p2 = PathBuf::from("Cargo.lock");
        let _ = app.update(Message::DockFilesDropped(vec![p1, p2]));
        assert!(app.file_1.is_some());
        assert!(app.file_2.is_some());
        assert!(app.diff_data.is_some());
    }

    #[test]
    fn test_dock_files_dropped_replaces_existing() {
        let mut app = App::default();
        let p1 = PathBuf::from("Cargo.toml");
        let p2 = PathBuf::from("Cargo.lock");
        let _ = app.update(Message::DockFilesDropped(vec![p1, p2]));
        let original_name = app.file_1.as_ref().unwrap().name.clone();
        assert_eq!(original_name, "Cargo.toml");

        let _ = app.update(Message::DockFilesDropped(vec![
            PathBuf::from("Cargo.lock"),
            PathBuf::from("Cargo.toml"),
        ]));
        assert_eq!(app.file_1.as_ref().unwrap().name, "Cargo.lock");
        assert_ne!(app.file_1.as_ref().unwrap().name, original_name);
    }

    #[test]
    fn test_clear_all_resets_files() {
        let mut app = App::default();
        let _ = app.update(Message::DockFilesDropped(vec![
            PathBuf::from("Cargo.toml"),
            PathBuf::from("Cargo.lock"),
        ]));
        assert!(app.file_1.is_some());
        assert!(app.file_2.is_some());
        assert!(app.diff_data.is_some());

        let _ = app.update(Message::ClearAll);
        assert!(app.file_1.is_none());
        assert!(app.file_2.is_none());
        assert!(app.diff_data.is_none());
    }

    #[test]
    fn test_check_system_theme_updates_theme() {
        // Force the app to think the OS is the opposite mode
        let mut app = App {
            is_os_dark: !theme::is_system_dark(),
            ..Default::default()
        };
        let _ = app.update(Message::CheckSystemTheme);
        // It should synchronize with the actual OS mode
        assert_eq!(app.is_os_dark, theme::is_system_dark());
    }
}
