#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(not(target_os = "macos"))]
mod non_macos {
    use iced::Subscription;
    use std::path::PathBuf;

    use crate::app::Message;

    pub fn init() {}

    pub fn send_paths(_paths: Vec<PathBuf>) {}

    pub fn subscription() -> Subscription<Message> {
        Subscription::none()
    }
}

#[cfg(not(target_os = "macos"))]
pub use non_macos::*;
