use std::time::Duration;

use eframe::egui;
use raw_window_handle::HasWindowHandle;
use reqwest::Client;
use tokio::sync::mpsc;

use crate::config::AppConfig;
use crate::llm::claude::ClaudeClient;
use crate::llm::gemini::GeminiClient;
use crate::llm::openai::OpenAiClient;
use crate::llm::{LlmClient, PromptRequest};
use crate::platform::WindowManager;

#[derive(PartialEq)]
pub enum ActiveTab {
    Chat,
    Settings,
}

pub struct StealthApp {
    pub config: AppConfig,
    pub window_mgr: Option<WindowManager>,
    pub active_tab: ActiveTab,

    pub prompt_text: String,
    pub response_text: String,
    pub is_generating: bool,

    pub rx_stream: mpsc::UnboundedReceiver<String>,
    pub tx_stream: mpsc::UnboundedSender<String>,
}

impl StealthApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let config = AppConfig::load();
        let (tx, rx) = mpsc::unbounded_channel();

        set_modern_theme(&cc.egui_ctx);

        let window_mgr = cc.window_handle().ok().map(|handle| {
            let raw: raw_window_handle::RawWindowHandle = handle.into();
            crate::platform::apply_stealth(raw);
            WindowManager::new(raw)
        });

        Self {
            config,
            window_mgr,
            active_tab: ActiveTab::Chat,
            prompt_text: String::new(),
            response_text: String::new(),
            is_generating: false,
            rx_stream: rx,
            tx_stream: tx,
        }
    }

    fn render_top_bar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.active_tab, ActiveTab::Chat, "💬 Assistant");
            ui.selectable_value(&mut self.active_tab, ActiveTab::Settings, "⚙ Settings");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(ref mut wm) = self.window_mgr {
                    let mut click_through = wm.is_click_through;
                    ui.add_space(8.0);
                    if ui.checkbox(&mut click_through, "Click-Through").changed() {
                        wm.set_click_through(click_through);
                        self.config.is_click_through = click_through;
                    }
                }

                ui.label(
                    egui::RichText::new(format!("{}", self.config.selected_provider))
                        .color(ui.visuals().weak_text_color()),
                );
            });
        });
        ui.add_space(6.0);
        ui.separator();
        ui.add_space(4.0);
    }

    fn render_chat_tab(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("chat_input_panel").show(ui, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let edit = egui::TextEdit::multiline(&mut self.prompt_text)
                    .hint_text("Ask something... (Ctrl+Enter to send)")
                    .desired_rows(2)
                    .font(egui::TextStyle::Body);
                let response = ui.add_sized([ui.available_width() - 84.0, 56.0], edit);

                if response.has_focus()
                    && ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Enter))
                {
                    self.send_prompt(ui.ctx().clone());
                }

                ui.add_space(4.0);
                ui.vertical(|ui| {
                    let send_btn = egui::Button::new(
                        egui::RichText::new("Send").size(14.0).strong(),
                    )
                    .min_size(egui::vec2(72.0, 30.0))
                    .fill(ui.visuals().selection.bg_fill)
                    .corner_radius(6.0);

                    if ui.add_enabled(!self.is_generating, send_btn).clicked() {
                        self.send_prompt(ui.ctx().clone());
                    }
                    if ui.button("Clear").clicked() {
                        self.response_text.clear();
                    }
                });
            });
            ui.add_space(8.0);
        });

        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(4.0);
            let frame = egui::Frame::default()
                .corner_radius(8.0)
                .inner_margin(egui::Margin::symmetric(10, 10))
                .fill(
                    egui::Color32::from_black_alpha(
                        (255.0 * (self.config.opacity * 0.35)) as u8,
                    ),
                );
            frame.show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if self.response_text.is_empty() {
                            ui.add_space(30.0);
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    egui::RichText::new("Ready for input...")
                                        .size(16.0)
                                        .color(ui.visuals().weak_text_color()),
                                );
                            });
                        } else {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(&self.response_text)
                                        .size(14.0)
                                        .line_height(Some(18.0))
                                        .color(egui::Color32::from_rgb(230, 235, 245)),
                                )
                                .wrap(),
                            );
                        }
                    });
            });
        });
    }

    fn render_settings_tab(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(6.0);

            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("API Configuration")
                    .size(15.0)
                    .strong()
                    .color(ui.visuals().selection.bg_fill),
            );
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Active Provider:");
                egui::ComboBox::from_id_salt("provider_select")
                    .selected_text(&self.config.selected_provider)
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.config.selected_provider,
                            "Gemini".into(),
                            "Gemini",
                        );
                        ui.selectable_value(
                            &mut self.config.selected_provider,
                            "OpenAI".into(),
                            "OpenAI",
                        );
                        ui.selectable_value(
                            &mut self.config.selected_provider,
                            "DeepSeek".into(),
                            "DeepSeek",
                        );
                        ui.selectable_value(
                            &mut self.config.selected_provider,
                            "Claude".into(),
                            "Claude",
                        );
                    });
            });
            ui.add_space(8.0);
            match self.config.selected_provider.as_str() {
                "Gemini" => {
                    ui.label("Gemini API Key:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.config.gemini_key).password(true),
                    );
                    ui.add_space(6.0);
                    ui.label("Model:");
                    ui.text_edit_singleline(&mut self.config.gemini_model);
                }
                "OpenAI" => {
                    ui.label("OpenAI API Key:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.config.openai_key).password(true),
                    );
                    ui.add_space(6.0);
                    ui.label("Model:");
                    ui.text_edit_singleline(&mut self.config.openai_model);
                }
                "DeepSeek" => {
                    ui.label("DeepSeek API Key:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.config.deepseek_key).password(true),
                    );
                    ui.add_space(6.0);
                    ui.label("Model:");
                    ui.text_edit_singleline(&mut self.config.deepseek_model);
                }
                "Claude" => {
                    ui.label("Claude API Key:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.config.claude_key).password(true),
                    );
                    ui.add_space(6.0);
                    ui.label("Model:");
                    ui.text_edit_singleline(&mut self.config.claude_model);
                }
                _ => {}
            }
            ui.add_space(8.0);
            ui.separator();

            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("System Instructions")
                    .size(15.0)
                    .strong()
                    .color(ui.visuals().selection.bg_fill),
            );
            ui.add_space(4.0);
            ui.add(
                egui::TextEdit::multiline(&mut self.config.system_prompt)
                    .desired_rows(4)
                    .desired_width(f32::INFINITY),
            );
            ui.weak("This is sent to the model as its system context on every request.");
            ui.add_space(8.0);
            ui.separator();

            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("Overlay & Stealth Behavior")
                    .size(15.0)
                    .strong()
                    .color(ui.visuals().selection.bg_fill),
            );
            ui.add_space(4.0);
            ui.checkbox(
                &mut self.config.enable_stealth_on_launch,
                "Hide from Screen Recorders on Startup",
            );
            ui.checkbox(&mut self.config.is_click_through, "Start in Click-Through mode");
            ui.add_space(10.0);
            if ui
                .button(egui::RichText::new("💾 Save Settings").strong())
                .clicked()
            {
                let _ = self.config.save();
            }
        });
    }

    pub fn send_prompt(&mut self, ctx: egui::Context) {
        let prompt = self.prompt_text.trim().to_string();
        if prompt.is_empty() || self.is_generating {
            return;
        }

        self.prompt_text.clear();
        self.response_text = "Thinking...".to_string();
        self.is_generating = true;

        let tx = self.tx_stream.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let result = async {
                let http_client = Client::builder()
                    .timeout(Duration::from_secs(60))
                    .connect_timeout(Duration::from_secs(15))
                    .build()
                    .map_err(|e| e.to_string())?;

                let req = PromptRequest {
                    system_prompt: config.system_prompt,
                    user_prompt: prompt,
                };

                match config.selected_provider.as_str() {
                    "Gemini" => {
                        let client = GeminiClient {
                            client: http_client,
                            api_key: config.gemini_key,
                            model: config.gemini_model,
                        };
                        client.stream_response(req, tx.clone()).await
                    }
                    "OpenAI" => {
                        let client = OpenAiClient {
                            client: http_client,
                            api_key: config.openai_key,
                            base_url: "https://api.openai.com/v1".to_string(),
                            model: config.openai_model,
                        };
                        client.stream_response(req, tx.clone()).await
                    }
                    "DeepSeek" => {
                        let client = OpenAiClient {
                            client: http_client,
                            api_key: config.deepseek_key,
                            base_url: "https://api.deepseek.com".to_string(),
                            model: config.deepseek_model,
                        };
                        client.stream_response(req, tx.clone()).await
                    }
                    "Claude" => {
                        let client = ClaudeClient {
                            client: http_client,
                            api_key: config.claude_key,
                            model: config.claude_model,
                        };
                        client.stream_response(req, tx.clone()).await
                    }
                    _ => Err("Invalid provider selected".to_string()),
                }
            }
            .await;

            if let Err(err) = result {
                let _ = tx.send(format!("\n[Error: {}]", err));
            }

            let _ = tx.send("[[DONE]]".to_string());
            ctx.request_repaint();
        });
    }
}

impl eframe::App for StealthApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mut received_new = false;

        while let Ok(chunk) = self.rx_stream.try_recv() {
            received_new = true;
            if chunk == "[[DONE]]" {
                self.is_generating = false;
                if self.response_text == "Thinking..." {
                    self.response_text.clear();
                }
            } else {
                if self.response_text == "Thinking..." {
                    self.response_text.clear();
                }
                self.response_text.push_str(&chunk);
            }
        }

        if received_new || self.is_generating {
            ui.ctx().request_repaint();
        }

        let frame = egui::Frame::default()
            .fill(egui::Color32::from_black_alpha((255.0 * self.config.opacity) as u8));

        egui::CentralPanel::default().frame(frame).show(ui, |ui| {
            self.render_top_bar(ui);

            match self.active_tab {
                ActiveTab::Chat => self.render_chat_tab(ui),
                ActiveTab::Settings => self.render_settings_tab(ui),
            }
        });
    }
}

fn set_modern_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = egui::Color32::from_rgb(15, 17, 22);
    visuals.window_fill = egui::Color32::from_rgb(17, 20, 26);
    visuals.extreme_bg_color = egui::Color32::from_rgb(10, 12, 16);
    visuals.faint_bg_color = egui::Color32::from_rgb(21, 24, 32);
    visuals.override_text_color = Some(egui::Color32::from_rgb(215, 225, 240));
    visuals.selection.bg_fill = egui::Color32::from_rgb(59, 130, 246);
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(59, 130, 246));
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(6);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(45, 50, 60));
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 76, 90));
    visuals.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(28, 32, 42);
    visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(34, 39, 50);
    visuals.hyperlink_color = egui::Color32::from_rgb(96, 165, 250);
    visuals.text_cursor.stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(130, 180, 255));

    ctx.set_visuals(visuals.clone());

    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style.spacing.text_edit_width = 220.0;
    style.spacing.combo_width = 160.0;
    style.visuals = visuals;

    ctx.set_style_of(egui::Theme::Dark, style);
}
