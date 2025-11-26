#![windows_subsystem = "windows"]

use anyhow::Result;
use arboard::Clipboard;
use directories::ProjectDirs;
use eframe::egui;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct CreateLinkRequest {
    #[serde(rename = "originalURL")]
    original_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cloaking: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "passwordContact")]
    password_contact: Option<bool>,
    #[serde(rename = "allowDuplicates")]
    allow_duplicates: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "clicksLimit")]
    clicks_limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "redirectType")]
    redirect_type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,
}

#[derive(Deserialize, Clone)]
struct LinkResponse {
    #[serde(rename = "shortURL")]
    short_url: String,
    #[serde(rename = "originalURL")]
    original_url: String,
}

struct Config {
    api_key: String,
    domain: String,
}

impl Config {
    fn config_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "shortyio", "shortyio")
            .map(|proj_dirs| proj_dirs.config_dir().join("config.json"))
    }

    fn load() -> Option<Self> {
        let path = Self::config_path()?;
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    fn save(&self) -> Result<()> {
        let path = Self::config_path().ok_or_else(|| anyhow::anyhow!("Cannot determine config path"))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}

impl Serialize for Config {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Config", 2)?;
        state.serialize_field("api_key", &self.api_key)?;
        state.serialize_field("domain", &self.domain)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Config {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ConfigHelper {
            api_key: String,
            domain: String,
        }
        let helper = ConfigHelper::deserialize(deserializer)?;
        Ok(Config {
            api_key: helper.api_key,
            domain: helper.domain,
        })
    }
}

struct ShortyApp {
    api_key: String,
    domain: String,
    original_url: String,
    custom_path: String,
    cloaking: bool,
    password: String,
    password_contact: bool,
    clicks_limit: String,
    redirect_type: i32,
    result: Option<LinkResponse>,
    error: Option<String>,
    loading: bool,
    show_settings: bool,
}

impl Default for ShortyApp {
    fn default() -> Self {
        let config = Config::load();

        let original_url = Clipboard::new()
            .ok()
            .and_then(|mut clipboard| clipboard.get_text().ok())
            .filter(|text| {
                text.starts_with("http://") || text.starts_with("https://")
            })
            .unwrap_or_default();

        Self {
            api_key: config.as_ref().map(|c| c.api_key.clone()).unwrap_or_default(),
            domain: config.as_ref().map(|c| c.domain.clone()).unwrap_or_default(),
            original_url,
            custom_path: String::new(),
            cloaking: false,
            password: String::new(),
            password_contact: false,
            clicks_limit: String::new(),
            redirect_type: 301,
            result: None,
            error: None,
            loading: false,
            show_settings: false,
        }
    }
}

impl ShortyApp {
    fn create_short_link(&mut self, ctx: egui::Context) {
        if self.api_key.is_empty() {
            self.error = Some("API key is required. Click settings (⚙) to configure.".to_string());
            return;
        }

        if self.original_url.is_empty() {
            self.error = Some("Original URL is required".to_string());
            return;
        }

        let api_key = self.api_key.clone();
        let domain = if self.domain.is_empty() {
            None
        } else {
            Some(self.domain.clone())
        };

        let clicks_limit = if self.clicks_limit.is_empty() {
            None
        } else {
            self.clicks_limit.parse::<i32>().ok()
        };

        let request = CreateLinkRequest {
            original_url: self.original_url.clone(),
            path: if self.custom_path.is_empty() {
                None
            } else {
                Some(self.custom_path.clone())
            },
            domain,
            cloaking: if self.cloaking { Some(true) } else { None },
            password: if self.password.is_empty() {
                None
            } else {
                Some(self.password.clone())
            },
            password_contact: if self.password_contact { Some(true) } else { None },
            allow_duplicates: false,
            clicks_limit,
            redirect_type: Some(self.redirect_type),
            tags: Some(vec!["shortyio".to_string()]),
        };

        self.loading = true;
        self.error = None;
        self.result = None;

        std::thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let client = reqwest::Client::new();
                let response = client
                    .post("https://api.short.io/links")
                    .header("authorization", api_key)
                    .json(&request)
                    .send()
                    .await;

                ctx.request_repaint();

                match response {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            match resp.json::<LinkResponse>().await {
                                Ok(link) => {
                                    ctx.data_mut(|data| {
                                        data.insert_temp(egui::Id::new("result"), Some(link));
                                        data.insert_temp(egui::Id::new("error"), None::<String>);
                                        data.insert_temp(egui::Id::new("loading"), false);
                                    });
                                }
                                Err(e) => {
                                    ctx.data_mut(|data| {
                                        data.insert_temp(egui::Id::new("result"), None::<LinkResponse>);
                                        data.insert_temp(
                                            egui::Id::new("error"),
                                            Some(format!("Failed to parse response: {}", e)),
                                        );
                                        data.insert_temp(egui::Id::new("loading"), false);
                                    });
                                }
                            }
                        } else {
                            let status = resp.status();
                            let error_text = resp.text().await.unwrap_or_default();
                            ctx.data_mut(|data| {
                                data.insert_temp(egui::Id::new("result"), None::<LinkResponse>);
                                data.insert_temp(
                                    egui::Id::new("error"),
                                    Some(format!("API error {}: {}", status, error_text)),
                                );
                                data.insert_temp(egui::Id::new("loading"), false);
                            });
                        }
                    }
                    Err(e) => {
                        ctx.data_mut(|data| {
                            data.insert_temp(egui::Id::new("result"), None::<LinkResponse>);
                            data.insert_temp(egui::Id::new("error"), Some(format!("Request failed: {}", e)));
                            data.insert_temp(egui::Id::new("loading"), false);
                        });
                    }
                }
            });
        });
    }
}

impl eframe::App for ShortyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.data_mut(|data| {
            if let Some(result) = data.get_temp::<Option<LinkResponse>>(egui::Id::new("result")) {
                self.result = result;
                data.remove::<Option<LinkResponse>>(egui::Id::new("result"));
            }
            if let Some(error) = data.get_temp::<Option<String>>(egui::Id::new("error")) {
                self.error = error;
                data.remove::<Option<String>>(egui::Id::new("error"));
            }
            if let Some(loading) = data.get_temp::<bool>(egui::Id::new("loading")) {
                self.loading = loading;
                data.remove::<bool>(egui::Id::new("loading"));
            }
        });

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        if self.show_settings {
            egui::Window::new("⚙ Settings")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.set_min_width(400.0);

                    ui.label("API Key:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.api_key)
                            .password(true)
                            .hint_text("Enter your short.io API key"),
                    );
                    ui.add_space(8.0);

                    ui.label("Domain (optional):");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.domain)
                            .hint_text("e.g., yourdomain.com"),
                    );
                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            let config = Config {
                                api_key: self.api_key.clone(),
                                domain: self.domain.clone(),
                            };
                            if let Err(e) = config.save() {
                                eprintln!("Failed to save config: {}", e);
                            }
                            self.show_settings = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_settings = false;
                        }
                    });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(16.0);
                ui.heading(egui::RichText::new("Shortyio").size(28.0).strong());
                ui.label(egui::RichText::new("Lightning-fast custom URL shortening").size(12.0).weak());
            });

            ui.add_space(20.0);

            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("URL").strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("⚙").on_hover_text("Settings").clicked() {
                            self.show_settings = true;
                        }
                    });
                });

                let url_response = ui.add(
                    egui::TextEdit::singleline(&mut self.original_url)
                        .hint_text("https://example.com/your-long-url")
                        .desired_width(f32::INFINITY),
                );
                if url_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.create_short_link(ctx.clone());
                }

                ui.add_space(8.0);
                ui.label(egui::RichText::new("Custom Path (optional)").strong());
                let path_response = ui.add(
                    egui::TextEdit::singleline(&mut self.custom_path)
                        .hint_text("my-custom-link")
                        .desired_width(f32::INFINITY),
                );
                if path_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.create_short_link(ctx.clone());
                }

                ui.add_space(8.0);

                ui.collapsing(egui::RichText::new("Advanced Options").strong(), |ui| {
                    ui.add_space(4.0);

                    ui.checkbox(&mut self.cloaking, "Enable cloaking")
                        .on_hover_text("Hide the redirect in an iframe");

                    ui.add_space(4.0);
                    ui.label("Password (optional):");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.password)
                            .password(true)
                            .hint_text("Protect link with password"),
                    );

                    ui.checkbox(&mut self.password_contact, "Show contact for password")
                        .on_hover_text("Provide email to users to get password");

                    ui.add_space(4.0);
                    ui.label("Clicks Limit (optional):");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.clicks_limit)
                            .hint_text("e.g., 100")
                            .desired_width(100.0),
                    ).on_hover_text("Disable link after this many clicks");

                    ui.add_space(4.0);
                    ui.label("Redirect Type:");
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.redirect_type, 301, "301 (Permanent)");
                        ui.radio_value(&mut self.redirect_type, 302, "302 (Temporary)");
                    });
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.redirect_type, 307, "307 (Temporary)");
                        ui.radio_value(&mut self.redirect_type, 308, "308 (Permanent)");
                    });
                });

                ui.add_space(12.0);

                ui.vertical_centered(|ui| {
                    let button = egui::Button::new(
                        egui::RichText::new("✨ Create Short Link").size(16.0)
                    ).min_size(egui::vec2(200.0, 36.0));

                    if ui.add_enabled(!self.loading, button).clicked() {
                        self.create_short_link(ctx.clone());
                    }
                });

                ui.add_space(8.0);
            });

            ui.add_space(8.0);

            if self.loading {
                ui.vertical_centered(|ui| {
                    ui.spinner();
                    ui.label("Creating short link...");
                });
            }

            if let Some(error) = &self.error {
                ui.add_space(8.0);
                ui.group(|ui| {
                    ui.set_min_width(ui.available_width());
                    ui.colored_label(egui::Color32::from_rgb(220, 60, 60), format!("❌ {}", error));
                });
            }

            if let Some(result) = &self.result {
                ui.add_space(8.0);
                ui.group(|ui| {
                    ui.set_min_width(ui.available_width());
                    ui.add_space(4.0);

                    ui.colored_label(egui::Color32::from_rgb(60, 179, 113),
                        egui::RichText::new("✅ Success!").size(14.0).strong());

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Short URL:").strong());
                        ui.add(
                            egui::TextEdit::singleline(&mut result.short_url.as_str())
                                .desired_width(ui.available_width() - 70.0),
                        );
                        if ui.button("📋 Copy").clicked() {
                            ui.output_mut(|o| o.copied_text = result.short_url.clone());
                        }
                    });

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Original:").weak().size(11.0));
                        ui.label(egui::RichText::new(&result.original_url).weak().size(11.0));
                    });

                    ui.add_space(4.0);
                });
            }

            ui.add_space(8.0);
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("Press ESC to exit").size(10.0).weak());
            });
        });
    }
}

fn load_icon() -> egui::IconData {
    let icon_bytes = include_bytes!("../icon.png");
    let image = image::load_from_memory(icon_bytes)
        .expect("Failed to load icon")
        .to_rgba8();
    let (width, height) = image.dimensions();
    egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    }
}

fn main() -> Result<(), eframe::Error> {
    let icon = load_icon();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 520.0])
            .with_resizable(true)
            .with_min_inner_size([480.0, 400.0])
            .with_icon(icon)
            .with_app_id("systems.weedmark.shortyio"),
        ..Default::default()
    };

    eframe::run_native(
        "Shorty",
        options,
        Box::new(|_cc| Ok(Box::new(ShortyApp::default()))),
    )
}
