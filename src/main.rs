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
    result: Option<LinkResponse>,
    error: Option<String>,
    loading: bool,
}

impl Default for ShortyApp {
    fn default() -> Self {
        let config = Config::load();
        println!("Starting up...");
        let original_url = Clipboard::new()
            .ok()
            .and_then(|mut clipboard| clipboard.get_text().ok())
            .filter(|text| {
                println!("{:?}", text);
                text.starts_with("http://") || text.starts_with("https://")
            })
            .unwrap_or_default();

        Self {
            api_key: config.as_ref().map(|c| c.api_key.clone()).unwrap_or_default(),
            domain: config.as_ref().map(|c| c.domain.clone()).unwrap_or_default(),
            original_url,
            custom_path: String::new(),
            result: None,
            error: None,
            loading: false,
        }
    }
}

impl ShortyApp {
    fn create_short_link(&mut self, ctx: egui::Context) {
        if self.api_key.is_empty() {
            self.error = Some("API key is required".to_string());
            return;
        }

        if self.original_url.is_empty() {
            self.error = Some("Original URL is required".to_string());
            return;
        }

        let config = Config {
            api_key: self.api_key.clone(),
            domain: self.domain.clone(),
        };
        if let Err(e) = config.save() {
            eprintln!("Failed to save config: {}", e);
        }

        let api_key = self.api_key.clone();
        let domain = if self.domain.is_empty() {
            None
        } else {
            Some(self.domain.clone())
        };
        let request = CreateLinkRequest {
            original_url: self.original_url.clone(),
            path: if self.custom_path.is_empty() {
                None
            } else {
                Some(self.custom_path.clone())
            },
            domain,
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

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Shortyio the Short.io URL Shortener");
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("API Key:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.api_key)
                        .password(true)
                        .desired_width(300.0),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Domain (optional):");
                ui.add(egui::TextEdit::singleline(&mut self.domain).desired_width(300.0));
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Original URL:");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.original_url)
                        .hint_text("https://example.com")
                        .desired_width(300.0),
                );
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.create_short_link(ctx.clone());
                }
            });

            ui.horizontal(|ui| {
                ui.label("Custom Path:");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.custom_path)
                        .hint_text("my-custom-link (optional)")
                        .desired_width(300.0),
                );
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.create_short_link(ctx.clone());
                }
            });

            ui.add_space(10.0);

            if ui
                .add_enabled(!self.loading, egui::Button::new("Create Short Link"))
                .clicked()
            {
                self.create_short_link(ctx.clone());
            }

            if self.loading {
                ui.add_space(10.0);
                ui.spinner();
                ui.label("Creating short link...");
            }

            if let Some(error) = &self.error {
                ui.add_space(10.0);
                ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
            }

            if let Some(result) = &self.result {
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
                ui.colored_label(egui::Color32::GREEN, "Success!");
                ui.horizontal(|ui| {
                    ui.label("Short URL:");
                    ui.add(
                        egui::TextEdit::singleline(&mut result.short_url.as_str())
                            .desired_width(300.0),
                    );
                    if ui.button("Copy").clicked() {
                        ui.output_mut(|o| o.copied_text = result.short_url.clone());
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Original:");
                    ui.label(&result.original_url);
                });
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 350.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "Shortyio",
        options,
        Box::new(|_cc| Ok(Box::new(ShortyApp::default()))),
    )
}
