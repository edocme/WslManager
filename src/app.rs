use iced::widget::{
    button, column, container, horizontal_rule, row, scrollable, text, text_editor, text_input,
};
use iced::{Element, Length, Task, Theme};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::monitor::MonitorData;
use crate::wsl;
use crate::i18n::{Language, t};

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Overview,
    DistroDetail,
    ConfigEditor,
    WslConfEditor,
    Monitor,
    Log,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigType {
    WslConfig,
    WslConf,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstallMode {
    Online,
    Local,
}

#[derive(Debug, Clone)]
pub struct DistroInfo {
    pub name: String,
    pub is_default: bool,
    pub state: String,
    pub wsl_version: u8,
}

pub struct WslManager {
    pub distros: Vec<DistroInfo>,
    pub selected_distro: Option<String>,
    pub active_tab: Tab,
    pub loading: bool,
    pub error: Option<String>,
    pub log_output: Vec<String>,

    pub config_content: text_editor::Content,
    pub config_modified: bool,
    pub config_type: ConfigType,

    pub rename_input: String,
    pub show_rename_dialog: bool,
    pub show_delete_confirm: bool,
    pub show_import_dialog: bool,
    pub show_install_dialog: bool,
    pub show_export_dialog: bool,
    pub import_path: String,
    pub import_name: String,
    pub import_vhdx_path: String,
    pub export_distro_name: String,
    pub export_path: String,
    pub install_distro_name: String,
    pub install_mode: InstallMode,
    pub install_tar_path: String,
    pub install_vhdx_path: String,

    pub monitor_data: MonitorData,
    pub current_lang: Language,
    pub loading_operation: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SwitchTab(Tab),
    RefreshDistros,
    DistrosLoaded(String),
    SelectDistro(String),
    StartDistro(String),
    StopDistro(String),
    RestartDistro(String),
    DeleteDistro(String),
    DeleteDistroConfirmed(String),
    SetDefault(String),
    ShowRenameDialog(String),
    RenameInputChanged(String),
    ConfirmRename,
    ShowExportDialog(String),
    DismissExportDialog,
    ExportPathChanged(String),
    ConfirmExport,
    ImportDistro,
    DismissImportDialog,
    ImportPathChanged(String),
    ImportNameChanged(String),
    ImportVhdxPathChanged(String),
    ConfirmImport,
    OpenTerminal(String),
    OpenExplorer(String),
    OpenVSCode(String),
    LoadConfig(ConfigType),
    ConfigLoaded(String),
    ConfigEditorAction(text_editor::Action),
    SaveConfig,
    ConfigSaved(String),
    ShowInstallDialog,
    DismissInstallDialog,
    InstallDistroNameChanged(String),
    SwitchInstallMode(InstallMode),
    InstallTarPathChanged(String),
    InstallVhdxPathChanged(String),
    ConfirmInstallOnline,
    ConfirmInstallLocal,
    RefreshMonitor,
    MonitorDataLoaded(String),
    LogMessage(String),
    Error(String),
    DismissError,
    DismissDeleteConfirm,
    DismissRenameDialog,
    SwitchWslVersion(String, u8),
    SwitchLanguage(Language),
}

pub fn bg(r: f32, g: f32, b: f32) -> Option<iced::Background> {
    Some(iced::Background::Color(iced::Color::from_rgb(r, g, b)))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DistroSource {
    pub source: String,
    pub tar_path: Option<String>,
    pub vhdx_path: Option<String>,
}

fn sources_file_path() -> PathBuf {
    let home = std::env::var("USERPROFILE").unwrap_or_default();
    std::path::PathBuf::from(home).join(".wsl-manager").join("sources.json")
}

pub fn load_sources() -> HashMap<String, DistroSource> {
    let path = sources_file_path();
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}

pub fn save_sources(sources: &HashMap<String, DistroSource>) {
    let path = sources_file_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(sources) {
        let _ = std::fs::write(&path, json);
    }
}

pub fn record_source(name: &str, source: DistroSource) {
    let mut sources = load_sources();
    sources.insert(name.to_string(), source);
    save_sources(&sources);
}

pub fn get_source(name: &str) -> Option<DistroSource> {
    load_sources().get(name).cloned()
}

impl WslManager {
    pub fn new() -> (Self, Task<Message>) {
        let app = Self {
            distros: Vec::new(),
            selected_distro: None,
            active_tab: Tab::Overview,
            loading: true,
            error: None,
            log_output: Vec::new(),
            config_content: text_editor::Content::default(),
            config_modified: false,
            config_type: ConfigType::WslConfig,
            rename_input: String::new(),
            show_rename_dialog: false,
            show_delete_confirm: false,
            show_import_dialog: false,
            show_install_dialog: false,
            show_export_dialog: false,
            import_path: String::new(),
            import_name: String::new(),
            import_vhdx_path: String::new(),
            export_distro_name: String::new(),
            export_path: String::new(),
            install_distro_name: String::new(),
            install_mode: InstallMode::Online,
            install_tar_path: String::new(),
            install_vhdx_path: String::new(),
            monitor_data: MonitorData::default(),
            current_lang: Language::Chinese,
            loading_operation: None,
        };
        (app, Task::perform(wsl::refresh_distros(), Message::DistrosLoaded))
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        let s = t(self.current_lang);
        match message {
            Message::SwitchTab(tab) => {
                self.active_tab = tab.clone();
                match tab {
                    Tab::ConfigEditor => {
                        self.config_type = ConfigType::WslConfig;
                        Task::perform(wsl::load_config(ConfigType::WslConfig), Message::ConfigLoaded)
                    }
                    Tab::WslConfEditor => {
                        self.config_type = ConfigType::WslConf;
                        let name = self.selected_distro.clone().or_else(|| {
                            self.distros.iter().find(|d| d.is_default).map(|d| d.name.clone())
                        }).or_else(|| {
                            self.distros.first().map(|d| d.name.clone())
                        });
                        if let Some(n) = name {
                            self.selected_distro = Some(n.clone());
                            Task::perform(wsl::load_wsl_conf(n), Message::ConfigLoaded)
                        } else {
                            self.error = Some(s.select_a_distro_first.into());
                            Task::none()
                        }
                    }
                    Tab::Monitor => {
                        self.loading = true;
                        Task::perform(crate::monitor::collect_monitor_data(), Message::MonitorDataLoaded)
                    }
                    _ => Task::none(),
                }
            }
            Message::RefreshDistros => {
                self.loading = true;
                Task::perform(wsl::refresh_distros(), Message::DistrosLoaded)
            }
            Message::DistrosLoaded(output) => {
                self.loading = false;
                self.loading_operation = None;
                let distros = wsl::parse_distros(&output);
                self.log(&format!("Parsed {} distro(s) from output", distros.len()));
                if distros.is_empty() && !output.starts_with("ERROR:") {
                    self.log(&format!("Raw output preview: {}", 
                        output.chars().take(200).collect::<String>()));
                }
                if self.selected_distro.is_none() {
                    if let Some(d) = distros.iter().find(|d| d.is_default) {
                        self.selected_distro = Some(d.name.clone());
                    } else if let Some(d) = distros.first() {
                        self.selected_distro = Some(d.name.clone());
                    }
                }
                self.distros = distros;
                Task::none()
            }
            Message::SelectDistro(name) => {
                self.selected_distro = Some(name);
                self.active_tab = Tab::DistroDetail;
                Task::none()
            }
            Message::StartDistro(name) => {
                self.loading_operation = Some(format!("{} {}...", s.start, name));
                self.log(&format!("Starting {}...", name));
                Task::perform(wsl::start_distro(name), |r| match r {
                    Ok(m) => Message::LogMessage(m),
                    Err(e) => Message::Error(e),
                })
            }
            Message::StopDistro(name) => {
                self.loading_operation = Some(format!("{} {}...", s.stop, name));
                self.log(&format!("Stopping {}...", name));
                Task::perform(wsl::stop_distro(name), |r| match r {
                    Ok(m) => Message::LogMessage(m),
                    Err(e) => Message::Error(e),
                })
            }
            Message::RestartDistro(name) => {
                self.loading_operation = Some(format!("{} {}...", s.restart, name));
                self.log(&format!("Restarting {}...", name));
                Task::perform(wsl::restart_distro(name), |r| match r {
                    Ok(m) => Message::LogMessage(m),
                    Err(e) => Message::Error(e),
                })
            }
            Message::DeleteDistro(name) => {
                self.show_delete_confirm = true;
                self.selected_distro = Some(name);
                Task::none()
            }
            Message::DeleteDistroConfirmed(name) => {
                self.show_delete_confirm = false;
                self.loading_operation = Some(format!("{} {}...", s.delete, name));
                self.log(&format!("Deleting {}...", name));
                {
                    let mut sources = load_sources();
                    sources.remove(&name);
                    save_sources(&sources);
                }
                Task::perform(wsl::delete_distro(name), |r| match r {
                    Ok(m) => Message::LogMessage(m),
                    Err(e) => Message::Error(e),
                })
            }
            Message::SetDefault(name) => {
                self.loading_operation = Some(format!("{} {}...", s.set_default, name));
                self.log(&format!("Setting {} as default", name));
                Task::perform(wsl::set_default_distro(name), |r| match r {
                    Ok(m) => Message::LogMessage(m),
                    Err(e) => Message::Error(e),
                })
            }
            Message::ShowRenameDialog(name) => {
                self.show_rename_dialog = !name.is_empty();
                if !name.is_empty() {
                    self.rename_input = name;
                }
                Task::none()
            }
            Message::RenameInputChanged(input) => {
                self.rename_input = input;
                Task::none()
            }
            Message::ConfirmRename => {
                let old = self.selected_distro.clone().unwrap_or_default();
                let new_name = self.rename_input.clone();
                self.show_rename_dialog = false;
                self.loading_operation = Some(format!("{}...", s.rename));
                if let Some(src) = get_source(&old) {
                    record_source(&new_name, src);
                    let mut sources = load_sources();
                    sources.remove(&old);
                    save_sources(&sources);
                }
                Task::perform(wsl::rename_distro(old, new_name), |r| match r {
                    Ok(m) => Message::LogMessage(m),
                    Err(e) => Message::Error(e),
                })
            }
            Message::ShowExportDialog(name) => {
                self.show_export_dialog = true;
                self.export_distro_name = name;
                self.export_path = String::new();
                Task::none()
            }
            Message::DismissExportDialog => {
                self.show_export_dialog = false;
                Task::none()
            }
            Message::ExportPathChanged(path) => {
                self.export_path = path;
                Task::none()
            }
            Message::ConfirmExport => {
                let name = self.export_distro_name.clone();
                let path = if self.export_path.trim().is_empty() {
                    let temp = std::env::temp_dir();
                    format!("{}\\wsl-export\\{}.tar", temp.to_string_lossy(), name)
                } else {
                    self.export_path.clone()
                };
                self.show_export_dialog = false;
                self.loading_operation = Some(format!("{} {}...", s.export, name));
                self.log(&format!("Exporting {} to {}...", name, path));
                Task::perform(wsl::export_distro_to(name, path), |r| match r {
                    Ok(m) => Message::LogMessage(m),
                    Err(e) => Message::Error(e),
                })
            }
            Message::ImportDistro => {
                self.show_import_dialog = true;
                self.import_path = String::new();
                self.import_name = self.selected_distro.clone().unwrap_or_default();
                self.import_vhdx_path = String::new();
                Task::none()
            }
            Message::DismissImportDialog => {
                self.show_import_dialog = false;
                Task::none()
            }
            Message::ImportPathChanged(path) => {
                self.import_path = path;
                Task::none()
            }
            Message::ImportNameChanged(name) => {
                self.import_name = name;
                Task::none()
            }
            Message::ImportVhdxPathChanged(path) => {
                self.import_vhdx_path = path;
                Task::none()
            }
            Message::ConfirmImport => {
                let path = self.import_path.clone();
                let name = self.import_name.clone();
                let vhdx = self.import_vhdx_path.clone();
                if name.trim().is_empty() {
                    self.error = Some(s.empty_name_error.into());
                    return Task::none();
                }
                if path.trim().is_empty() {
                    self.error = Some(s.empty_path_error.into());
                    return Task::none();
                }
                self.show_import_dialog = false;
                self.loading_operation = Some(format!("{} {}...", s.import, name));
                self.log(&format!("Importing {} from {}...", name, path));
                record_source(&name, DistroSource {
                    source: "Local".to_string(),
                    tar_path: Some(path.clone()),
                    vhdx_path: if vhdx.trim().is_empty() { None } else { Some(vhdx.clone()) },
                });
                Task::perform(wsl::import_distro_to(name, path, vhdx), |r| match r {
                    Ok(m) => Message::LogMessage(m),
                    Err(e) => Message::Error(e),
                })
            }
            Message::OpenTerminal(name) => {
                self.log(&format!("Opening terminal for {}", name));
                Task::perform(wsl::open_terminal(name), |r| match r {
                    Ok(m) => Message::LogMessage(m),
                    Err(e) => Message::Error(e),
                })
            }
            Message::OpenExplorer(name) => {
                self.log(&format!("Opening explorer for {}", name));
                Task::perform(wsl::open_explorer(name), |r| match r {
                    Ok(m) => Message::LogMessage(m),
                    Err(e) => Message::Error(e),
                })
            }
            Message::OpenVSCode(name) => {
                self.log(&format!("Opening VS Code for {}", name));
                Task::perform(wsl::open_vscode(name), |r| match r {
                    Ok(m) => Message::LogMessage(m),
                    Err(e) => Message::Error(e),
                })
            }
            Message::LoadConfig(config_type) => {
                self.config_type = config_type.clone();
                match config_type {
                    ConfigType::WslConfig => {
                        Task::perform(wsl::load_config(ConfigType::WslConfig), Message::ConfigLoaded)
                    }
                    ConfigType::WslConf => {
                        let name = self.selected_distro.clone().or_else(|| {
                            self.distros.iter().find(|d| d.is_default).map(|d| d.name.clone())
                        }).or_else(|| {
                            self.distros.first().map(|d| d.name.clone())
                        });
                        if let Some(n) = name {
                            self.selected_distro = Some(n.clone());
                            Task::perform(wsl::load_wsl_conf(n), Message::ConfigLoaded)
                        } else {
                            self.error = Some(s.no_distro_selected.into());
                            Task::none()
                        }
                    }
                }
            }
            Message::ConfigLoaded(content) => {
                self.config_content = text_editor::Content::with_text(&content);
                self.config_modified = false;
                Task::none()
            }
            Message::ConfigEditorAction(action) => {
                if matches!(action, text_editor::Action::Edit(_)) {
                    self.config_modified = true;
                }
                self.config_content.perform(action);
                Task::none()
            }
            Message::SaveConfig => {
                let text = self.config_content.text();
                let config_type = self.config_type.clone();
                let distro = self.selected_distro.clone();
                Task::perform(
                    wsl::save_config(config_type, distro, text),
                    |r| match r {
                        Ok(m) => Message::ConfigSaved(m),
                        Err(e) => Message::Error(e),
                    },
                )
            }
            Message::ConfigSaved(msg) => {
                self.config_modified = false;
                self.log_output.push(msg);
                Task::none()
            }
            Message::ShowInstallDialog => {
                self.show_install_dialog = true;
                self.install_distro_name = String::new();
                self.install_mode = InstallMode::Online;
                self.install_tar_path = String::new();
                self.install_vhdx_path = String::new();
                Task::none()
            }
            Message::DismissInstallDialog => {
                self.show_install_dialog = false;
                Task::none()
            }
            Message::InstallDistroNameChanged(name) => {
                self.install_distro_name = name;
                Task::none()
            }
            Message::SwitchInstallMode(mode) => {
                self.install_mode = mode;
                Task::none()
            }
            Message::InstallTarPathChanged(path) => {
                self.install_tar_path = path;
                Task::none()
            }
            Message::InstallVhdxPathChanged(path) => {
                self.install_vhdx_path = path;
                Task::none()
            }
            Message::ConfirmInstallOnline => {
                let name = self.install_distro_name.clone();
                if name.trim().is_empty() {
                    self.error = Some(s.empty_name_error.into());
                    return Task::none();
                }
                self.show_install_dialog = false;
                self.loading_operation = Some(format!("{} {}...", s.install_distro, name));
                self.log(&format!("Installing {} from Store...", name));
                record_source(&name, DistroSource {
                    source: "Microsoft Store".to_string(),
                    tar_path: None,
                    vhdx_path: None,
                });
                Task::perform(wsl::install_distro_from_store(name), |r| match r {
                    Ok(m) => Message::LogMessage(m),
                    Err(e) => Message::Error(e),
                })
            }
            Message::ConfirmInstallLocal => {
                let name = self.install_distro_name.clone();
                let tar = self.install_tar_path.clone();
                let vhdx = self.install_vhdx_path.clone();
                if name.trim().is_empty() {
                    self.error = Some(s.empty_name_error.into());
                    return Task::none();
                }
                if tar.trim().is_empty() {
                    self.error = Some(s.empty_path_error.into());
                    return Task::none();
                }
                self.show_install_dialog = false;
                self.loading_operation = Some(format!("{} {}...", s.install_distro, name));
                self.log(&format!("Installing {} from local tar...", name));
                record_source(&name, DistroSource {
                    source: "Local".to_string(),
                    tar_path: Some(tar.clone()),
                    vhdx_path: if vhdx.trim().is_empty() { None } else { Some(vhdx.clone()) },
                });
                Task::perform(wsl::import_distro_to(name, tar, vhdx), |r| match r {
                    Ok(m) => Message::LogMessage(m),
                    Err(e) => Message::Error(e),
                })
            }
            Message::RefreshMonitor => {
                self.loading = true;
                Task::perform(crate::monitor::collect_monitor_data(), Message::MonitorDataLoaded)
            }
            Message::MonitorDataLoaded(data) => {
                self.loading = false;
                self.loading_operation = None;
                if let Ok(parsed) = serde_json::from_str::<MonitorData>(&data) {
                    self.monitor_data = parsed;
                }
                Task::none()
            }
            Message::LogMessage(msg) => {
                self.log_output.push(msg);
                self.loading = false;
                self.loading_operation = None;
                Task::perform(wsl::refresh_distros(), Message::DistrosLoaded)
            }
            Message::Error(e) => {
                self.error = Some(e);
                self.loading = false;
                self.loading_operation = None;
                Task::none()
            }
            Message::DismissError => {
                self.error = None;
                Task::none()
            }
            Message::DismissDeleteConfirm => {
                self.show_delete_confirm = false;
                Task::none()
            }
            Message::DismissRenameDialog => {
                self.show_rename_dialog = false;
                Task::none()
            }
            Message::SwitchWslVersion(name, ver) => {
                self.loading_operation = Some(format!("WSL{}...", ver));
                self.log(&format!("Switching {} to WSL{}...", name, ver));
                Task::perform(wsl::set_wsl_version(name, ver), |r| match r {
                    Ok(m) => Message::LogMessage(m),
                    Err(e) => Message::Error(e),
                })
            }
            Message::SwitchLanguage(lang) => {
                self.current_lang = lang;
                Task::none()
            }
        }
    }

    pub fn log(&mut self, msg: &str) {
        self.log_output
            .push(format!("[{}] {}", chrono::Local::now().format("%H:%M:%S"), msg));
    }

    pub fn view(&self) -> Element<'_, Message> {
        let sidebar = self.view_sidebar();
        let content = self.view_content();

        let main_layout = row![sidebar, content]
            .spacing(0)
            .height(Length::Fill);

        let mut top = column![].spacing(5).width(Length::Fill).height(Length::Fill);

        if let Some(ref error) = self.error {
            let s = t(self.current_lang);
            let error_bar = container(
                row![
                    text(format!("{}{}", s.error_prefix, error)),
                    button(s.dismiss).on_press(Message::DismissError)
                ]
                .spacing(10),
            )
            .padding(8)
            .style(styles::error_bar);
            top = top.push(error_bar);
        }

        if let Some(ref op) = self.loading_operation {
            let s = t(self.current_lang);
            let status_bar = container(
                row![
                    text("  ").size(12),
                    text(format!("{} {}", op, s.please_wait))
                        .size(13)
                        .color(iced::Color::from_rgb(0.9, 0.8, 0.3)),
                ]
                .spacing(5),
            )
            .padding(6)
            .style(styles::status_bar);
            top = top.push(status_bar);
        }

        top = top.push(main_layout);
        container(top)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_sidebar(&self) -> Element<'_, Message> {
        let s = t(self.current_lang);
        let title = text(s.app_title).size(20).color(iced::Color::from_rgb(0.4, 0.8, 1.0));

        let refresh_btn = button(s.refresh).on_press(Message::RefreshDistros).width(Length::Fill);
        let install_btn = button(s.install_distro).on_press(Message::ShowInstallDialog).width(Length::Fill);

        let mut distro_list = column![].spacing(4);
        for distro in &self.distros {
            let state_icon = if distro.state == "Running" { "[R] " } else { "[S] " };
            let default_marker = if distro.is_default { " *" } else { "" };
            let label = format!(
                "{}{} WSL{}{}",
                state_icon, distro.name, distro.wsl_version, default_marker
            );

            let is_selected = self.selected_distro.as_deref() == Some(&distro.name);
            let btn = if is_selected {
                button(text(label).size(14).color(iced::Color::WHITE))
                    .on_press(Message::SelectDistro(distro.name.clone()))
                    .width(Length::Fill)
                    .style(styles::sidebar_button_selected)
            } else {
                button(text(label).size(14).color(iced::Color::WHITE))
                    .on_press(Message::SelectDistro(distro.name.clone()))
                    .width(Length::Fill)
                    .style(styles::sidebar_button)
            };

            distro_list = distro_list.push(btn);
        }

        let mut lang_row = row![].spacing(4);
        for &lang in Language::all() {
            let label = if lang == self.current_lang {
                text(lang.label()).color(iced::Color::WHITE).size(13)
            } else {
                text(lang.label()).size(13).color(iced::Color::from_rgb(0.6, 0.6, 0.7))
            };
            let btn = button(label)
                .on_press(Message::SwitchLanguage(lang))
                .padding([4, 8]);
            lang_row = lang_row.push(btn);
        }

        let distro_scroll = scrollable(distro_list).height(Length::Fill);

        let tab_overview = button(s.overview)
            .on_press(Message::SwitchTab(Tab::Overview))
            .width(Length::Fill);
        let tab_monitor = button(s.monitor)
            .on_press(Message::SwitchTab(Tab::Monitor))
            .width(Length::Fill);
        let tab_config = button(s.wslconfig)
            .on_press(Message::SwitchTab(Tab::ConfigEditor))
            .width(Length::Fill);
        let tab_wslconf = button(s.wslconf)
            .on_press(Message::SwitchTab(Tab::WslConfEditor))
            .width(Length::Fill);
        let tab_log = button(s.log)
            .on_press(Message::SwitchTab(Tab::Log))
            .width(Length::Fill);

        let nav_tabs = column![tab_overview, tab_monitor, tab_config, tab_wslconf, tab_log].spacing(4);

        column![
            title,
            horizontal_rule(1),
            nav_tabs,
            horizontal_rule(1),
            install_btn,
            horizontal_rule(1),
            text(s.distros)
                .size(14)
                .color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            distro_scroll,
            horizontal_rule(1),
            text(s.language)
                .size(12)
                .color(iced::Color::from_rgb(0.5, 0.5, 0.6)),
            lang_row,
            horizontal_rule(1),
            refresh_btn,
        ]
        .spacing(8)
        .padding(10)
        .width(240)
        .height(Length::Fill)
        .into()
    }

    fn view_content(&self) -> Element<'_, Message> {
        let content: Element<'_, Message> = match self.active_tab {
            Tab::Overview => self.view_overview(),
            Tab::DistroDetail => self.view_distro_detail(),
            Tab::ConfigEditor => self.view_config_editor(),
            Tab::WslConfEditor => self.view_wslconf_editor(),
            Tab::Monitor => self.view_monitor(),
            Tab::Log => self.view_log(),
        };

        let mut wrapped = column![content].padding(15).height(Length::Fill).width(Length::Fill);

        if self.show_rename_dialog {
            wrapped = wrapped.push(self.view_rename_dialog());
        }
        if self.show_delete_confirm {
            wrapped = wrapped.push(self.view_delete_confirm());
        }
        if self.show_import_dialog {
            wrapped = wrapped.push(self.view_import_dialog());
        }
        if self.show_install_dialog {
            wrapped = wrapped.push(self.view_install_dialog());
        }
        if self.show_export_dialog {
            wrapped = wrapped.push(self.view_export_dialog());
        }

        container(wrapped)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_overview(&self) -> Element<'_, Message> {
        let s = t(self.current_lang);
        let title = text(s.overview).size(22);
        let count = self.distros.len();
        let running = self.distros.iter().filter(|d| d.state == "Running").count();
        let stopped = count - running;

        let stats = row![
            stat_card(s.total, &count.to_string(), iced::Color::from_rgb(0.3, 0.6, 1.0)),
            stat_card(s.running, &running.to_string(), iced::Color::from_rgb(0.2, 0.8, 0.3)),
            stat_card(s.stopped, &stopped.to_string(), iced::Color::from_rgb(0.8, 0.3, 0.2)),
        ]
        .spacing(15);

        let mut distro_cards = column![].spacing(8);
        for distro in &self.distros {
            let state_color = if distro.state == "Running" {
                iced::Color::from_rgb(0.2, 0.8, 0.3)
            } else {
                iced::Color::from_rgb(0.8, 0.3, 0.2)
            };
            let default_marker = if distro.is_default { " *Default*" } else { "" };

            let card_content = column![
                row![
                    container(
                        text("")
                            .size(8)
                    )
                    .padding(4)
                    .center_y(12)
                    .width(12)
                    .height(12)
                    .style(move |_: &Theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(state_color)),
                        border: iced::Border::default().rounded(6),
                        ..Default::default()
                    }),
                    text(format!("{}{}", distro.name, default_marker))
                        .size(16)
                        .color(iced::Color::WHITE),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                text(format!("WSL{} | {}", distro.wsl_version, distro.state))
                    .size(12)
                    .color(iced::Color::from_rgb(0.6, 0.6, 0.7)),
            ]
            .spacing(4);

            let card = container(card_content)
                .padding(12)
                .width(Length::Fill)
                .style(styles::card);

            distro_cards = distro_cards.push(card);
        }

        let distro_scroll = scrollable(distro_cards).height(Length::Fill);

        column![title, stats, horizontal_rule(1), distro_scroll]
            .spacing(15)
            .width(Length::Fill)
            .into()
    }

    fn view_distro_detail(&self) -> Element<'_, Message> {
        let s = t(self.current_lang);
        let name = match &self.selected_distro {
            Some(n) => n.clone(),
            None => return text(s.select_distro_hint).into(),
        };

        let distro = self.distros.iter().find(|d| d.name == name);
        let (state, wsl_version) = match distro {
            Some(d) => (d.state.clone(), d.wsl_version),
            None => ("Unknown".to_string(), 0),
        };

        let title = text(format!("Distro: {}", name)).size(22);
        let info = text(format!("State: {} | WSL Version: WSL{}", state, wsl_version))
            .size(14)
            .color(iced::Color::from_rgb(0.7, 0.7, 0.8));

        let start_btn = button(text(s.start).color(iced::Color::WHITE))
            .on_press(Message::StartDistro(name.clone()))
            .style(styles::green_button);
        let stop_btn = button(text(s.stop).color(iced::Color::WHITE))
            .on_press(Message::StopDistro(name.clone()))
            .style(styles::red_button);
        let restart_btn = button(text(s.restart).color(iced::Color::WHITE))
            .on_press(Message::RestartDistro(name.clone()));
        let terminal_btn = button(s.terminal).on_press(Message::OpenTerminal(name.clone()));
        let explorer_btn = button(s.explorer).on_press(Message::OpenExplorer(name.clone()));
        let vscode_btn = button(s.vscode).on_press(Message::OpenVSCode(name.clone()));
        let export_btn = button(s.export).on_press(Message::ShowExportDialog(name.clone()));
        let import_btn = button(s.import).on_press(Message::ImportDistro);
        let rename_btn = button(s.rename).on_press(Message::ShowRenameDialog(name.clone()));
        let set_default_btn = button(s.set_default).on_press(Message::SetDefault(name.clone()));
        let delete_btn = button(text(s.delete).color(iced::Color::from_rgb(1.0, 0.5, 0.5)))
            .on_press(Message::DeleteDistro(name.clone()))
            .style(styles::danger_button);

        let actions_row1 = row![start_btn, stop_btn, restart_btn].spacing(10);
        let actions_row2 = row![terminal_btn, explorer_btn, vscode_btn].spacing(10);
        let actions_row3 = row![export_btn, import_btn].spacing(10);
        let actions_row4 = row![rename_btn, set_default_btn, delete_btn].spacing(10);

        let wsl1_btn = if wsl_version == 1 {
            button(text("WSL1 Active").color(iced::Color::WHITE)).style(styles::active_version)
        } else {
            button("WSL1").on_press(Message::SwitchWslVersion(name.clone(), 1))
        };
        let wsl2_btn = if wsl_version == 2 {
            button(text("WSL2 Active").color(iced::Color::WHITE)).style(styles::active_version)
        } else {
            button("WSL2").on_press(Message::SwitchWslVersion(name.clone(), 2))
        };
        let version_row = row![text(s.version).size(14), wsl1_btn, wsl2_btn].spacing(8);

        let source_info = {
            let source = get_source(&name);
            match source {
                Some(src) => {
                    let mut lines = vec![format!("{}: {}", s.install_source, src.source)];
                    if let Some(ref tar) = src.tar_path {
                        lines.push(format!("{}: {}", s.tar_file_path, tar));
                    }
                    if let Some(ref vhdx) = src.vhdx_path {
                        lines.push(format!("{}: {}", s.vhdx_path, vhdx));
                    }
                    column![
                        text(lines.join("\n"))
                            .size(13)
                            .color(iced::Color::from_rgb(0.6, 0.7, 0.8)),
                    ]
                    .spacing(2)
                }
                None => {
                    column![
                        text(s.unknown_source)
                            .size(13)
                            .color(iced::Color::from_rgb(0.5, 0.5, 0.6)),
                    ]
                }
            }
        };

        let scrollable_content = scrollable(
            column![
                title,
                info,
                horizontal_rule(1),
                text(s.lifecycle).size(16),
                actions_row1,
                text(s.quick_actions).size(16),
                actions_row2,
                text(s.import_export).size(16),
                actions_row3,
                text(s.advanced).size(16),
                actions_row4,
                horizontal_rule(1),
                version_row,
                horizontal_rule(1),
                text(s.install_source).size(16),
                source_info,
            ]
            .spacing(12)
            .width(Length::Fill)
        )
        .height(Length::Fill);

        scrollable_content.into()
    }

    fn view_config_editor(&self) -> Element<'_, Message> {
        let s = t(self.current_lang);
        let title = text(s.wsl_global_config).size(22);

        let save_btn = if self.config_modified {
            button(text(s.save_modified).color(iced::Color::WHITE))
                .on_press(Message::SaveConfig)
                .style(styles::green_button)
        } else {
            button(s.save)
        };
        let reload_btn = button(s.reload).on_press(Message::LoadConfig(ConfigType::WslConfig));
        let toolbar = row![save_btn, reload_btn].spacing(10);

        let editor = text_editor(&self.config_content)
            .on_action(Message::ConfigEditorAction)
            .height(Length::Fill);

        let help_text = text("Location: C:\\Users\\<user>\\.wslconfig\nSettings: memory, processors, swap, nestedVirtualization, firewall, autoProxy")
            .size(12)
            .color(iced::Color::from_rgb(0.5, 0.5, 0.6));

        column![title, toolbar, editor, help_text]
            .spacing(10)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    fn view_wslconf_editor(&self) -> Element<'_, Message> {
        let s = t(self.current_lang);
        let distro_label = self.selected_distro.as_deref().unwrap_or("?");
        let title = text(format!("{} ({})", s.distro_config, distro_label)).size(22);

        let save_btn = if self.config_modified {
            button(text(s.save_modified).color(iced::Color::WHITE))
                .on_press(Message::SaveConfig)
                .style(styles::green_button)
        } else {
            button(s.save)
        };
        let reload_btn = button(s.reload).on_press(Message::LoadConfig(ConfigType::WslConf));
        let toolbar = row![save_btn, reload_btn].spacing(10);

        let editor = text_editor(&self.config_content)
            .on_action(Message::ConfigEditorAction)
            .height(Length::Fill);

        let help_text = text("Location: /etc/wsl.conf (inside distro)\nSettings: [boot] systemd, [automount] enabled/options, [network] generateHosts/generateResolvConf, [interop] enabled")
            .size(12)
            .color(iced::Color::from_rgb(0.5, 0.5, 0.6));

        column![title, toolbar, editor, help_text]
            .spacing(10)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    fn view_monitor(&self) -> Element<'_, Message> {
        let s = t(self.current_lang);
        let title = text(s.system_monitor).size(22);
        let refresh_btn = button(s.refresh).on_press(Message::RefreshMonitor);

        let wsl_version = &self.monitor_data.wsl_version;
        let wsl_info = text(format!("WSL Version: {}", wsl_version))
            .size(14)
            .color(iced::Color::from_rgb(0.7, 0.7, 0.8));

        let mut distro_stats = column![].spacing(8);
        for dm in &self.monitor_data.distros {
            let mem_display = if dm.memory_mb > 0 {
                format!("{} MB", dm.memory_mb)
            } else {
                "N/A".to_string()
            };
            let disk_display = if !dm.disk_usage.is_empty() {
                dm.disk_usage.clone()
            } else {
                "N/A".to_string()
            };

            let card = container(
                column![
                    text(&dm.name).size(16).color(iced::Color::WHITE),
                    text(format!("Memory: {} | Disk: {}", mem_display, disk_display))
                        .size(12)
                        .color(iced::Color::from_rgb(0.6, 0.6, 0.7)),
                    text(format!("Processes: {}", dm.process_count))
                        .size(12)
                        .color(iced::Color::from_rgb(0.6, 0.6, 0.7)),
                ]
                .spacing(4),
            )
            .padding(12)
            .width(Length::Fill)
            .style(styles::card);

            distro_stats = distro_stats.push(card);
        }

        let distro_scroll = scrollable(distro_stats).height(Length::Fill);

        column![title, wsl_info, refresh_btn, horizontal_rule(1), distro_scroll]
            .spacing(12)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    fn view_log(&self) -> Element<'_, Message> {
        let s = t(self.current_lang);
        let title = text(s.operation_log).size(22);

        let mut log_list = column![].spacing(2);
        for line in &self.log_output {
            log_list = log_list.push(
                text(line)
                    .size(13)
                    .color(iced::Color::from_rgb(0.7, 0.8, 0.9)),
            );
        }

        let log_scroll = scrollable(log_list).height(Length::Fill).anchor_bottom();

        column![title, horizontal_rule(1), log_scroll]
            .spacing(10)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    fn view_rename_dialog(&self) -> Element<'_, Message> {
        let s = t(self.current_lang);
        let title = text(s.rename_distro).size(18);
        let input = text_input(s.new_name, &self.rename_input)
            .on_input(Message::RenameInputChanged)
            .size(14)
            .width(Length::Fill);

        let confirm_btn = button(text(s.confirm).color(iced::Color::WHITE))
            .on_press(Message::ConfirmRename)
            .style(styles::green_button);
        let cancel_btn = button(s.cancel).on_press(Message::DismissRenameDialog);

        let content = column![
            title,
            input,
            row![confirm_btn, cancel_btn].spacing(10)
        ]
        .spacing(10)
        .padding(20)
        .width(350);

        container(scrollable(content).height(Length::Shrink))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(styles::dialog)
            .into()
    }

    fn view_delete_confirm(&self) -> Element<'_, Message> {
        let s = t(self.current_lang);
        let name = self.selected_distro.clone().unwrap_or_default();
        let title = text(format!("{}: {}", s.confirm_delete, name)).size(18);
        let warning = text(s.confirm_delete_warning)
            .size(14)
            .color(iced::Color::from_rgb(1.0, 0.4, 0.3));

        let confirm_btn = button(text(s.delete).color(iced::Color::from_rgb(1.0, 0.5, 0.5)))
            .on_press(Message::DeleteDistroConfirmed(name))
            .style(styles::danger_button);
        let cancel_btn = button(s.cancel).on_press(Message::DismissDeleteConfirm);

        let content = column![
            title,
            warning,
            row![confirm_btn, cancel_btn].spacing(10)
        ]
        .spacing(10)
        .padding(20)
        .width(350);

        container(scrollable(content).height(Length::Shrink))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(styles::dialog)
            .into()
    }

    fn view_import_dialog(&self) -> Element<'_, Message> {
        let s = t(self.current_lang);
        let title = text(s.import_distro).size(18);
        let path_input = text_input(s.tar_file_path, &self.import_path)
            .on_input(Message::ImportPathChanged)
            .size(14)
            .width(Length::Fill);
        let name_input = text_input(s.new_distro_name, &self.import_name)
            .on_input(Message::ImportNameChanged)
            .size(14)
            .width(Length::Fill);
        let vhdx_hint = text(s.vhdx_path_hint)
            .size(11)
            .color(iced::Color::from_rgb(0.5, 0.5, 0.6));
        let vhdx_input = text_input(s.vhdx_path, &self.import_vhdx_path)
            .on_input(Message::ImportVhdxPathChanged)
            .size(14)
            .width(Length::Fill);

        let confirm_btn = button(text(s.import).color(iced::Color::WHITE))
            .on_press(Message::ConfirmImport)
            .style(styles::green_button);
        let cancel_btn = button(s.cancel).on_press(Message::DismissImportDialog);

        let content = column![
            title,
            path_input,
            name_input,
            vhdx_input,
            vhdx_hint,
            row![confirm_btn, cancel_btn].spacing(10)
        ]
        .spacing(8)
        .padding(20)
        .width(450);

        container(scrollable(content).height(Length::Shrink))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(styles::dialog)
            .into()
    }

    fn view_install_dialog(&self) -> Element<'_, Message> {
        let s = t(self.current_lang);
        let title = text(s.install_distro).size(18);

        let online_btn = if self.install_mode == InstallMode::Online {
            button(text(s.online_install).color(iced::Color::WHITE))
                .on_press(Message::SwitchInstallMode(InstallMode::Online))
                .style(styles::active_version)
        } else {
            button(s.online_install)
                .on_press(Message::SwitchInstallMode(InstallMode::Online))
        };
        let local_btn = if self.install_mode == InstallMode::Local {
            button(text(s.local_install).color(iced::Color::WHITE))
                .on_press(Message::SwitchInstallMode(InstallMode::Local))
                .style(styles::active_version)
        } else {
            button(s.local_install)
                .on_press(Message::SwitchInstallMode(InstallMode::Local))
        };
        let mode_row = row![online_btn, local_btn].spacing(8);

        let name_input = text_input(s.distro_name_hint, &self.install_distro_name)
            .on_input(Message::InstallDistroNameChanged)
            .size(14)
            .width(Length::Fill);

        let mut fields = column![title, mode_row, name_input].spacing(8);

        if self.install_mode == InstallMode::Online {
            let hint = text(s.install_hint)
                .size(12)
                .color(iced::Color::from_rgb(0.6, 0.6, 0.7));
            fields = fields.push(hint);
        } else {
            let tar_input = text_input(s.tar_file_path, &self.install_tar_path)
                .on_input(Message::InstallTarPathChanged)
                .size(14)
                .width(Length::Fill);
            let vhdx_hint = text(s.vhdx_path_hint)
                .size(11)
                .color(iced::Color::from_rgb(0.5, 0.5, 0.6));
            let vhdx_input = text_input(s.vhdx_path, &self.install_vhdx_path)
                .on_input(Message::InstallVhdxPathChanged)
                .size(14)
                .width(Length::Fill);
            fields = fields.push(tar_input).push(vhdx_input).push(vhdx_hint);
        }

        let confirm_btn = if self.install_mode == InstallMode::Online {
            button(text(s.confirm).color(iced::Color::WHITE))
                .on_press(Message::ConfirmInstallOnline)
                .style(styles::green_button)
        } else {
            button(text(s.confirm).color(iced::Color::WHITE))
                .on_press(Message::ConfirmInstallLocal)
                .style(styles::green_button)
        };
        let cancel_btn = button(s.cancel).on_press(Message::DismissInstallDialog);

        fields = fields.push(row![confirm_btn, cancel_btn].spacing(10));

        container(scrollable(fields).height(Length::Shrink))
            .padding(20)
            .width(450)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(styles::dialog)
            .into()
    }

    fn view_export_dialog(&self) -> Element<'_, Message> {
        let s = t(self.current_lang);
        let title = text(format!("{}: {}", s.export_distro_title, self.export_distro_name))
            .size(18);
        let hint = text(s.default_export_path)
            .size(11)
            .color(iced::Color::from_rgb(0.5, 0.5, 0.6));
        let path_input = text_input(s.export_path_hint, &self.export_path)
            .on_input(Message::ExportPathChanged)
            .size(14)
            .width(Length::Fill);

        let confirm_btn = button(text(s.export).color(iced::Color::WHITE))
            .on_press(Message::ConfirmExport)
            .style(styles::green_button);
        let cancel_btn = button(s.cancel).on_press(Message::DismissExportDialog);

        let content = column![
            title,
            path_input,
            hint,
            row![confirm_btn, cancel_btn].spacing(10)
        ]
        .spacing(8)
        .padding(20)
        .width(450);

        container(scrollable(content).height(Length::Shrink))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(styles::dialog)
            .into()
    }
}

fn stat_card(label: &str, value: &str, color: iced::Color) -> Element<'static, Message> {
    let val = text(value.to_owned()).size(28).color(color);
    let lbl = text(label.to_owned())
        .size(12)
        .color(iced::Color::from_rgb(0.6, 0.6, 0.7));

    container(column![val, lbl].spacing(4).align_x(iced::Alignment::Center))
        .padding(15)
        .width(Length::FillPortion(1))
        .style(styles::card)
        .into()
}

pub mod styles {
    use super::*;

    pub fn error_bar(_: &Theme) -> iced::widget::container::Style {
        iced::widget::container::Style {
            background: bg(0.8, 0.2, 0.2),
            text_color: Some(iced::Color::WHITE),
            border: iced::Border::default().rounded(4),
            ..Default::default()
        }
    }

    pub fn status_bar(_: &Theme) -> iced::widget::container::Style {
        iced::widget::container::Style {
            background: bg(0.15, 0.15, 0.05),
            text_color: Some(iced::Color::from_rgb(0.9, 0.8, 0.3)),
            border: iced::Border::default().rounded(4),
            ..Default::default()
        }
    }

    pub fn sidebar_button(
        _theme: &Theme,
        _status: iced::widget::button::Status,
    ) -> iced::widget::button::Style {
        iced::widget::button::Style {
            background: bg(0.15, 0.15, 0.2),
            text_color: iced::Color::WHITE,
            border: iced::Border::default().rounded(6),
            ..Default::default()
        }
    }

    pub fn sidebar_button_selected(
        _theme: &Theme,
        _status: iced::widget::button::Status,
    ) -> iced::widget::button::Style {
        iced::widget::button::Style {
            background: bg(0.15, 0.35, 0.65),
            text_color: iced::Color::WHITE,
            border: iced::Border::default()
                .rounded(6)
                .color(iced::Color::from_rgb(0.3, 0.6, 1.0))
                .width(2),
            ..Default::default()
        }
    }

    pub fn card(_: &Theme) -> iced::widget::container::Style {
        iced::widget::container::Style {
            background: bg(0.12, 0.12, 0.18),
            border: iced::Border::default()
                .rounded(8)
                .color(iced::Color::from_rgb(0.25, 0.25, 0.35)),
            ..Default::default()
        }
    }

    pub fn dialog(_: &Theme) -> iced::widget::container::Style {
        iced::widget::container::Style {
            background: bg(0.15, 0.15, 0.2),
            border: iced::Border::default()
                .rounded(12)
                .color(iced::Color::from_rgb(0.4, 0.4, 0.5))
                .width(2),
            ..Default::default()
        }
    }

    pub fn green_button(
        _theme: &Theme,
        _status: iced::widget::button::Status,
    ) -> iced::widget::button::Style {
        iced::widget::button::Style {
            background: bg(0.15, 0.5, 0.15),
            text_color: iced::Color::WHITE,
            border: iced::Border::default().rounded(6),
            ..Default::default()
        }
    }

    pub fn red_button(
        _theme: &Theme,
        _status: iced::widget::button::Status,
    ) -> iced::widget::button::Style {
        iced::widget::button::Style {
            background: bg(0.6, 0.15, 0.15),
            text_color: iced::Color::WHITE,
            border: iced::Border::default().rounded(6),
            ..Default::default()
        }
    }

    pub fn danger_button(
        _theme: &Theme,
        _status: iced::widget::button::Status,
    ) -> iced::widget::button::Style {
        iced::widget::button::Style {
            background: bg(0.6, 0.1, 0.1),
            text_color: iced::Color::from_rgb(1.0, 0.5, 0.5),
            border: iced::Border::default().rounded(6),
            ..Default::default()
        }
    }

    pub fn active_version(
        _theme: &Theme,
        _status: iced::widget::button::Status,
    ) -> iced::widget::button::Style {
        iced::widget::button::Style {
            background: bg(0.3, 0.6, 1.0),
            text_color: iced::Color::WHITE,
            border: iced::Border::default().rounded(6),
            ..Default::default()
        }
    }
}
