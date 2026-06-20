#![windows_subsystem = "windows"]

mod app;
mod wsl;
mod monitor;
mod i18n;

use app::WslManager;

const ICON_48: &[u8] = include_bytes!("..\\favicon48_ico\\favicon48.ico");

fn main() -> iced::Result {
    let icon = iced::window::icon::from_file_data(ICON_48, None)
        .expect("Failed to load icon");

    iced::application("WSL Manager", WslManager::update, WslManager::view)
        .theme(WslManager::theme)
        .default_font(iced::Font::with_name("Microsoft YaHei"))
        .window(iced::window::Settings {
            size: iced::Size::new(1100.0, 700.0),
            icon: Some(icon),
            ..Default::default()
        })
        .run_with(WslManager::new)
}
