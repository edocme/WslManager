#![windows_subsystem = "windows"]

mod app;
mod wsl;
mod monitor;
mod i18n;

use app::WslManager;

const ICON_16: &[u8] = include_bytes!("..\\favicon48_ico\\favicon16.ico");
const ICON_32: &[u8] = include_bytes!("..\\favicon48_ico\\favicon32.ico");
const ICON_48: &[u8] = include_bytes!("..\\favicon48_ico\\favicon48.ico");
const ICON_64: &[u8] = include_bytes!("..\\favicon48_ico\\favicon64.ico");
const ICON_128: &[u8] = include_bytes!("..\\favicon48_ico\\favicon128.ico");
const ICON_256: &[u8] = include_bytes!("..\\favicon48_ico\\favicon256.ico");

fn load_best_icon() -> iced::window::Icon {
    let sources = [ICON_256, ICON_128, ICON_64, ICON_48, ICON_32, ICON_16];
    for &src in &sources {
        if let Ok(icon) = iced::window::icon::from_file_data(src, None) {
            return icon;
        }
    }
    panic!("Failed to load any icon");
}

fn main() -> iced::Result {
    let icon = load_best_icon();

    iced::application("WSL 管理器", WslManager::update, WslManager::view)
        .theme(WslManager::theme)
        .default_font(iced::Font::with_name("Microsoft YaHei"))
        .window(iced::window::Settings {
            size: iced::Size::new(1100.0, 700.0),
            icon: Some(icon),
            ..Default::default()
        })
        .run_with(WslManager::new)
}
