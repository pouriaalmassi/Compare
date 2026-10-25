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
use crate::whitespace::WhitespaceMode;

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
    pub change_starts: Vec<usize>,
    pub current_change: Option<usize>,
    pub wrap_lines: bool,
    pub is_hovering: bool,
    pub is_new_drop_sequence: bool,
    pub status_message: Option<String>,
    pub theme: AppTheme,
    pub is_os_dark: bool,
    pub font_size: f32,
    pub base_font_size: f32,
    pub diff_font: iced::Font,
    pub whitespace: WhitespaceMode,
    pub tab_width: usize,
}

impl Default for App {
    fn default() -> Self {
        let is_os_dark = theme::is_system_dark();
        let (config, app_theme) = theme::load_settings(is_os_dark);
        let base_font_size = config
            .font_size
            .unwrap_or(DEFAULT_FONT_SIZE)
            .clamp(MIN_FONT_SIZE, MAX_FONT_SIZE);

        Self {
            file_1: None,
            file_2: None,
            diff_data: None,
            change_starts: Vec::new(),
            current_change: None,
            wrap_lines: config.wrap_lines.unwrap_or(true),
            is_hovering: false,
            is_new_drop_sequence: false,
            status_message: None,
            theme: app_theme,
            is_os_dark,
            font_size: base_font_size,
            base_font_size,
            diff_font: configured_font(config.font_family.as_deref()),
            whitespace: config.whitespace.unwrap_or_default(),
            tab_width: config.tab_width.unwrap_or(4).clamp(1, 16),
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
    FilesChosen(Vec<PathBuf>),
    ClearAll,
    SwapSides,
    ToggleWrapLines(bool),
    SetWhitespace(WhitespaceMode),
    ZoomIn,
    ZoomOut,
    ResetZoom,
    CheckSystemTheme,
    NextChange,
    PreviousChange,
    DismissConfigWarning,
}

fn configured_font(family: Option<&str>) -> iced::Font {
    match family.map(str::trim) {
        None | Some("") => iced::Font::MONOSPACE,
        Some(name) if name.eq_ignore_ascii_case("monospace") => iced::Font::MONOSPACE,
        Some(name) => {
            // Iced requires a static family name. Retain this one startup setting
            // for the application lifetime, rather than allocating during rendering.
            iced::Font::with_name(Box::leak(name.to_owned().into_boxed_str()))
        }
    }
}

fn shortcut(key: Key, modifiers: keyboard::Modifiers) -> Option<Message> {
    if modifiers.command() {
        match key.as_ref() {
            Key::Character("+" | "=") => Some(Message::ZoomIn),
            Key::Character("-" | "_") => Some(Message::ZoomOut),
            Key::Character("0") => Some(Message::ResetZoom),
            _ => None,
        }
    } else if modifiers.is_empty() {
        match key.as_ref() {
            Key::Character("j") => Some(Message::NextChange),
            Key::Character("k") => Some(Message::PreviousChange),
            Key::Character("s") => Some(Message::SwapSides),
            _ => None,
        }
    } else {
        None
    }
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
            Message::DockFilesDropped(paths) | Message::FilesChosen(paths) => {
                self.is_hovering = false;
                self.is_new_drop_sequence = false;
                if paths.len() >= 2 {
                    self.file_1 = None;
                    self.file_2 = None;
                    self.diff_data = None;
                    self.change_starts.clear();
                    self.current_change = None;
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
                    self.change_starts.clear();
                    self.current_change = None;
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
                    return self.update(Message::FilesChosen(paths));
                }
                if self.file_1.is_some() || self.file_2.is_some() {
                    Self::maximize_window()
                } else {
                    Task::none()
                }
            }
            Message::NextChange => self.navigate_change(true),
            Message::PreviousChange => self.navigate_change(false),
            Message::DismissConfigWarning => {
                self.theme.warning = None;
                Task::none()
            }
            Message::ClearAll => {
                self.file_1 = None;
                self.file_2 = None;
                self.diff_data = None;
                self.change_starts.clear();
                self.current_change = None;
                self.status_message = None;
                Self::restore_window()
            }
            Message::SwapSides => {
                std::mem::swap(&mut self.file_1, &mut self.file_2);
                self.update_diff();
                Task::none()
            }
            Message::SetWhitespace(mode) => {
                self.whitespace = mode;
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

    fn navigate_change(&mut self, forward: bool) -> Task<Message> {
        if self.change_starts.is_empty() {
            return Task::none();
        }
        let last = self.change_starts.len() - 1;
        let index = match (self.current_change, forward) {
            (None, true) => 0,
            (None, false) => last,
            (Some(index), true) => (index + 1) % self.change_starts.len(),
            (Some(0), false) => last,
            (Some(index), false) => index - 1,
        };
        self.current_change = Some(index);
        crate::ui::navigation::scroll_to_row(self.change_starts[index])
    }

    pub fn update_diff(&mut self) {
        self.current_change = None;
        self.change_starts.clear();
        if let (Some(f1), Some(f2)) = (&self.file_1, &self.file_2) {
            let diff = compute_diff(&f1.content, &f2.content);
            self.change_starts = diff.change_starts();
            self.diff_data = Some(diff);
        } else {
            self.diff_data = None;
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let window_events = event::listen_with(|event, _status, _window| match event {
            Event::Window(window::Event::FileDropped(path)) => Some(Message::FileDropped(path)),
            Event::Window(window::Event::FileHovered(_)) => Some(Message::FileHovered),
            Event::Window(window::Event::FilesHoveredLeft) => Some(Message::FilesHoveredLeft),
            Event::Window(window::Event::Focused) => Some(Message::CheckSystemTheme),
            _ => None,
        });

        // Cross-platform zoom: `modifiers.command()` maps to ⌘ Command on macOS
        // and Ctrl on Windows/Linux.
        let key_events = keyboard::on_key_press(shortcut);

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

        let content = if let Some(diff) = &self.diff_data {
            view_diff(
                diff,
                file_1_name,
                file_2_name,
                self.wrap_lines,
                self.is_hovering,
                self.status_message.as_deref(),
                &self.theme.diff,
                self.font_size,
                self.diff_font,
                &self.change_starts,
                self.current_change,
                self.whitespace,
                self.tab_width,
            )
        } else {
            view_empty_state(
                file_1_name,
                self.is_hovering,
                self.status_message.as_deref(),
                &self.theme.diff,
                self.theme.is_dark,
            )
        };
        if let Some(warning) = &self.theme.warning {
            use iced::widget::{button, column, container, row, scrollable, text};
            let banner = container(
                row![
                    scrollable(
                        text(format!("{warning}\nUsing fallback settings for this file.")).size(13)
                    )
                    .height(iced::Length::Shrink),
                    button("Dismiss").on_press(Message::DismissConfigWarning),
                ]
                .spacing(12),
            )
            .padding(12)
            .max_height(140);
            column![banner, content].into()
        } else {
            content
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn comparison(left: &str, right: &str) -> App {
        let file = |content: &str| TextFile {
            path: PathBuf::from("test.txt"),
            name: "test.txt".to_string(),
            content: content.to_string(),
        };
        let mut app = App {
            file_1: Some(file(left)),
            file_2: Some(file(right)),
            ..Default::default()
        };
        app.update_diff();
        app
    }

    #[test]
    fn whitespace_mode_does_not_change_comparison_data() {
        let mut app = comparison("a \tb\n", "a\tb \n");
        let count = app.diff_data.as_ref().unwrap().differ_count;
        for mode in WhitespaceMode::ALL {
            let _ = app.update(Message::SetWhitespace(mode));
            assert_eq!(app.whitespace, mode);
            assert_eq!(app.file_1.as_ref().unwrap().content, "a \tb\n");
            assert_eq!(app.file_2.as_ref().unwrap().content, "a\tb \n");
            assert_eq!(app.diff_data.as_ref().unwrap().differ_count, count);
        }
    }

    #[test]
    fn configured_font_defaults_and_named_family() {
        for family in [
            None,
            Some(""),
            Some("  "),
            Some("monospace"),
            Some(" Monospace "),
        ] {
            assert_eq!(configured_font(family), iced::Font::MONOSPACE);
        }
        let config: theme::ConfigFile =
            toml::from_str("font_family = \" Menlo \"\nfont_size = 15").unwrap();
        assert_eq!(
            configured_font(config.font_family.as_deref()),
            iced::Font::with_name("Menlo")
        );
        assert_eq!(config.font_size, Some(15.0));
        assert!(toml::from_str::<theme::ConfigFile>("font_family = 123").is_err());
    }

    #[test]
    fn navigation_visits_blocks_and_cycles_at_boundaries() {
        let mut app = comparison("a\nold\nkeep\nremoved\n", "a\nnew\nkeep\nadded\n");
        assert_eq!(app.change_starts, vec![1, 3]);
        let _ = app.update(Message::NextChange);
        assert_eq!(app.current_change, Some(0));
        let _ = app.update(Message::NextChange);
        assert_eq!(app.current_change, Some(1));
        let _ = app.update(Message::NextChange);
        assert_eq!(app.current_change, Some(0));
        let _ = app.update(Message::PreviousChange);
        assert_eq!(app.current_change, Some(1));
        let _ = app.update(Message::PreviousChange);
        assert_eq!(app.current_change, Some(0));

        app.current_change = None;
        let _ = app.update(Message::PreviousChange);
        assert_eq!(app.current_change, Some(1));
        let _ = app.update(Message::SwapSides);
        assert_eq!(app.current_change, None);
        let _ = app.update(Message::NextChange);
        let _ = app.update(Message::ClearAll);
        assert!(app.change_starts.is_empty());
        assert_eq!(app.current_change, None);
    }

    #[test]
    fn navigation_cycles_with_one_change() {
        let mut app = comparison("old\n", "new\n");
        for message in [
            Message::NextChange,
            Message::NextChange,
            Message::PreviousChange,
        ] {
            let _ = app.update(message);
            assert_eq!(app.current_change, Some(0));
        }
    }

    #[test]
    fn navigation_is_safe_without_changes() {
        for mut app in [App::default(), comparison("same\n", "same\n")] {
            let _ = app.update(Message::NextChange);
            let _ = app.update(Message::PreviousChange);
            assert_eq!(app.current_change, None);
        }
    }

    #[test]
    fn vim_shortcuts_require_unmodified_keys() {
        assert_eq!(
            shortcut(Key::Character("j".into()), keyboard::Modifiers::empty()),
            Some(Message::NextChange)
        );
        assert_eq!(
            shortcut(Key::Character("k".into()), keyboard::Modifiers::empty()),
            Some(Message::PreviousChange)
        );
        assert_eq!(
            shortcut(Key::Character("s".into()), keyboard::Modifiers::empty()),
            Some(Message::SwapSides)
        );
        for modifiers in [
            keyboard::Modifiers::CTRL,
            keyboard::Modifiers::ALT,
            keyboard::Modifiers::SHIFT,
            keyboard::Modifiers::LOGO,
        ] {
            assert_eq!(shortcut(Key::Character("j".into()), modifiers), None);
            assert_eq!(shortcut(Key::Character("k".into()), modifiers), None);
            assert_eq!(shortcut(Key::Character("s".into()), modifiers), None);
        }
    }

    #[test]
    fn config_warning_survives_clear_until_dismissed() {
        let mut app = App::default();
        app.theme.warning = Some("Invalid configuration".to_string());
        let _ = app.update(Message::ClearAll);
        assert!(app.theme.warning.is_some());
        let _ = app.update(Message::DismissConfigWarning);
        assert!(app.theme.warning.is_none());
    }

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
    fn test_files_chosen_replaces_existing_pair() {
        let mut app = App::default();
        let _ = app.update(Message::DockFilesDropped(vec![
            PathBuf::from("Cargo.toml"),
            PathBuf::from("Cargo.lock"),
        ]));

        let _ = app.update(Message::FilesChosen(vec![
            PathBuf::from("README.md"),
            PathBuf::from("config.example.toml"),
        ]));

        assert_eq!(
            app.file_1.as_ref().unwrap().path,
            PathBuf::from("README.md")
        );
        assert_eq!(
            app.file_2.as_ref().unwrap().path,
            PathBuf::from("config.example.toml")
        );
        let expected = compute_diff(
            &app.file_1.as_ref().unwrap().content,
            &app.file_2.as_ref().unwrap().content,
        );
        assert_eq!(
            app.diff_data.as_ref().unwrap().differ_count,
            expected.differ_count
        );
        assert_eq!(
            app.diff_data.as_ref().unwrap().rows.len(),
            expected.rows.len()
        );
    }

    #[test]
    fn test_single_file_chosen_preserves_left_file() {
        let mut app = App::default();
        let _ = app.update(Message::FilesChosen(vec![PathBuf::from("Cargo.toml")]));
        assert!(app.file_2.is_none());

        let _ = app.update(Message::FilesChosen(vec![PathBuf::from("Cargo.lock")]));
        assert!(app.diff_data.is_some());

        let _ = app.update(Message::FilesChosen(vec![PathBuf::from("README.md")]));
        assert_eq!(
            app.file_1.as_ref().unwrap().path,
            PathBuf::from("Cargo.toml")
        );
        assert_eq!(
            app.file_2.as_ref().unwrap().path,
            PathBuf::from("README.md")
        );
        assert!(app.diff_data.is_some());
    }

    #[test]
    fn test_files_chosen_uses_first_two_files() {
        let mut app = App::default();
        let _ = app.update(Message::FilesChosen(vec![
            PathBuf::from("Cargo.toml"),
            PathBuf::from("Cargo.lock"),
            PathBuf::from("README.md"),
        ]));
        assert_eq!(
            app.file_1.as_ref().unwrap().path,
            PathBuf::from("Cargo.toml")
        );
        assert_eq!(
            app.file_2.as_ref().unwrap().path,
            PathBuf::from("Cargo.lock")
        );
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
