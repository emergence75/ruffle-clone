use crate::gui::{available_languages, optional_text, text};
use crate::preferences::GlobalPreferences;
use cpal::traits::{DeviceTrait, HostTrait};
use egui::{Align2, Button, ComboBox, Grid, Ui, Widget, Window};
use ruffle_render_wgpu::clap::{GraphicsBackend, PowerPreference};
use ruffle_render_wgpu::descriptors::Descriptors;
use std::borrow::Cow;
use unic_langid::LanguageIdentifier;

pub struct PreferencesDialog {
    available_backends: wgpu::Backends,
    preferences: GlobalPreferences,

    graphics_backend: GraphicsBackend,
    graphics_backend_readonly: bool,
    graphics_backend_changed: bool,

    power_preference: PowerPreference,
    power_preference_readonly: bool,
    power_preference_changed: bool,

    language: LanguageIdentifier,
    language_changed: bool,

    output_device: Option<String>,
    available_output_devices: Vec<String>,
    output_device_changed: bool,
}

impl PreferencesDialog {
    pub fn new(descriptors: &Descriptors, preferences: GlobalPreferences) -> Self {
        let mut available_backends = wgpu::Backends::empty();

        available_backends |= backend_availability(descriptors, wgpu::Backends::VULKAN);
        available_backends |= backend_availability(descriptors, wgpu::Backends::GL);
        available_backends |= backend_availability(descriptors, wgpu::Backends::METAL);
        available_backends |= backend_availability(descriptors, wgpu::Backends::DX12);

        let audio_host = cpal::default_host();
        let mut available_output_devices = Vec::new();
        if let Ok(devices) = audio_host.output_devices() {
            for device in devices {
                if let Ok(name) = device.name() {
                    available_output_devices.push(name);
                }
            }
        }

        Self {
            available_backends,
            graphics_backend: preferences.graphics_backends(),
            graphics_backend_readonly: preferences.cli.graphics.is_some(),
            graphics_backend_changed: false,

            power_preference: preferences.graphics_power_preference(),
            power_preference_readonly: preferences.cli.power.is_some(),
            power_preference_changed: false,

            language: preferences.language(),
            language_changed: false,

            output_device: preferences.output_device_name(),
            available_output_devices,
            output_device_changed: false,

            preferences,
        }
    }

    pub fn show(&mut self, locale: &LanguageIdentifier, egui_ctx: &egui::Context) -> bool {
        let mut keep_open = true;
        let mut should_close = false;
        let locked_text = text(locale, "preference-locked-by-cli");

        Window::new(text(locale, "preferences-dialog"))
            .open(&mut keep_open)
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .show(egui_ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    Grid::new("preferences-dialog-graphics")
                        .num_columns(2)
                        .striped(true)
                        .show(ui, |ui| {
                            self.show_graphics_preferences(locale, &locked_text, ui);

                            self.show_language_preferences(locale, ui);

                            self.show_audio_preferences(locale, ui);
                        });

                    if self.restart_required() {
                        ui.colored_label(
                            ui.style().visuals.error_fg_color,
                            "A restart is required to apply the selected changes",
                        );
                    }

                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if Button::new(text(locale, "save")).ui(ui).clicked() {
                                self.save();
                                should_close = true;
                            }
                        })
                    });
                });
            });

        keep_open && !should_close
    }

    fn restart_required(&self) -> bool {
        self.graphics_backend != self.preferences.graphics_backends()
            || self.power_preference != self.preferences.graphics_power_preference()
            || self.output_device != self.preferences.output_device_name()
    }

    fn show_graphics_preferences(
        &mut self,
        locale: &LanguageIdentifier,
        locked_text: &str,
        ui: &mut Ui,
    ) {
        ui.label(text(locale, "graphics-backend"));
        if self.graphics_backend_readonly {
            ui.label(graphics_backend_name(locale, self.graphics_backend))
                .on_hover_text(locked_text);
        } else {
            let previous = self.graphics_backend;
            ComboBox::from_id_source("graphics-backend")
                .selected_text(graphics_backend_name(locale, self.graphics_backend))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.graphics_backend,
                        GraphicsBackend::Default,
                        text(locale, "graphics-backend-default"),
                    );
                    if self.available_backends.contains(wgpu::Backends::VULKAN) {
                        ui.selectable_value(
                            &mut self.graphics_backend,
                            GraphicsBackend::Vulkan,
                            "Vulkan",
                        );
                    }
                    if self.available_backends.contains(wgpu::Backends::METAL) {
                        ui.selectable_value(
                            &mut self.graphics_backend,
                            GraphicsBackend::Metal,
                            "Metal",
                        );
                    }
                    if self.available_backends.contains(wgpu::Backends::DX12) {
                        ui.selectable_value(
                            &mut self.graphics_backend,
                            GraphicsBackend::Dx12,
                            "DirectX 12",
                        );
                    }
                    if self.available_backends.contains(wgpu::Backends::GL) {
                        ui.selectable_value(
                            &mut self.graphics_backend,
                            GraphicsBackend::Gl,
                            "OpenGL",
                        );
                    }
                });
            if self.graphics_backend != previous {
                self.graphics_backend_changed = true;
            }
        }
        ui.end_row();

        ui.label(text(locale, "graphics-power"));
        if self.power_preference_readonly {
            ui.label(graphics_power_name(locale, self.power_preference))
                .on_hover_text(locked_text);
        } else {
            let previous = self.power_preference;
            ComboBox::from_id_source("graphics-power")
                .selected_text(graphics_power_name(locale, self.power_preference))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.power_preference,
                        PowerPreference::Low,
                        text(locale, "graphics-power-low"),
                    );
                    ui.selectable_value(
                        &mut self.power_preference,
                        PowerPreference::High,
                        text(locale, "graphics-power-high"),
                    );
                });
            if self.power_preference != previous {
                self.power_preference_changed = true;
            }
        }
        ui.end_row();
    }

    fn show_language_preferences(&mut self, locale: &LanguageIdentifier, ui: &mut Ui) {
        ui.label(text(locale, "language"));
        let previous = self.language.clone();
        ComboBox::from_id_source("language")
            .selected_text(self.language.to_string())
            .show_ui(ui, |ui| {
                for language in available_languages() {
                    let name = optional_text(language, "language-name")
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| language.to_string());
                    ui.selectable_value(&mut self.language, language.clone(), name);
                }
            });
        if self.language != previous {
            self.language_changed = true;
        }
        ui.end_row();
    }

    fn show_audio_preferences(&mut self, locale: &LanguageIdentifier, ui: &mut Ui) {
        ui.label(text(locale, "audio-output-device"));

        let previous = self.output_device.clone();
        let default = text(locale, "audio-output-device-default");
        ComboBox::from_id_source("audio-output-device")
            .selected_text(self.output_device.as_deref().unwrap_or(default.as_ref()))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.output_device, None, default);
                for device in &self.available_output_devices {
                    ui.selectable_value(&mut self.output_device, Some(device.to_string()), device);
                }
            });
        if self.output_device != previous {
            self.output_device_changed = true;
        }
        ui.end_row();
    }

    fn save(&mut self) {
        if let Err(e) = self.preferences.write_preferences(|preferences| {
            if self.graphics_backend_changed {
                preferences.set_graphics_backend(self.graphics_backend);
            }
            if self.power_preference_changed {
                preferences.set_graphics_power_preference(self.power_preference);
            }
            if self.language_changed {
                preferences.set_language(self.language.clone());
            }
            if self.output_device_changed {
                preferences.set_output_device(self.output_device.clone());
                // [NA] TODO: Inform the running player that the device changed
            }
        }) {
            // [NA] TODO: Better error handling... everywhere in desktop, really
            tracing::error!("Could not save preferences: {e}");
        }
    }
}

fn graphics_backend_name(locale: &LanguageIdentifier, backend: GraphicsBackend) -> Cow<str> {
    match backend {
        GraphicsBackend::Default => text(locale, "graphics-backend-default"),
        GraphicsBackend::Vulkan => Cow::Borrowed("Vulkan"),
        GraphicsBackend::Metal => Cow::Borrowed("Metal"),
        GraphicsBackend::Dx12 => Cow::Borrowed("DirectX 12"),
        GraphicsBackend::Gl => Cow::Borrowed("OpenGL"),
    }
}

fn graphics_power_name(locale: &LanguageIdentifier, power_preference: PowerPreference) -> Cow<str> {
    match power_preference {
        PowerPreference::Low => text(locale, "graphics-power-low"),
        PowerPreference::High => text(locale, "graphics-power-high"),
    }
}

fn backend_availability(descriptors: &Descriptors, backend: wgpu::Backends) -> wgpu::Backends {
    if descriptors
        .wgpu_instance
        .enumerate_adapters(backend)
        .is_empty()
    {
        wgpu::Backends::empty()
    } else {
        backend
    }
}
