use std::path::PathBuf;

use droidcore_apk::{inspect_apk, ApkInfo};
use eframe::egui;

#[derive(Default)]
pub struct DroidCoreApp {
    selected_path: Option<PathBuf>,
    apk_info: Option<ApkInfo>,
    status: String,
}

impl DroidCoreApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            status: "Ready. Select an APK to inspect its contents.".to_owned(),
            ..Default::default()
        }
    }

    fn open_apk(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Android application package", &["apk"])
            .pick_file()
        else {
            return;
        };

        self.selected_path = Some(path.clone());
        match inspect_apk(&path) {
            Ok(info) => {
                self.status = format!(
                    "Inspection complete: {} archive entries found.",
                    info.entries
                );
                self.apk_info = Some(info);
            }
            Err(error) => {
                self.status = format!("Inspection failed: {error}");
                self.apk_info = None;
            }
        }
    }
}

impl eframe::App for DroidCoreApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("DroidCore");
                ui.separator();
                ui.label("Android package inspector");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Open APK").clicked() {
                        self.open_apk();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0);
            ui.heading("Package analysis");
            ui.label(&self.status);
            ui.add_space(12.0);

            if let Some(info) = &self.apk_info {
                egui::Grid::new("apk_summary").striped(true).show(ui, |ui| {
                    ui.label("File");
                    ui.monospace(info.path.file_name().unwrap_or_default().to_string_lossy());
                    ui.end_row();

                    ui.label("Package name");
                    ui.monospace(info.package_name.as_deref().unwrap_or("Unavailable (binary manifest)"));
                    ui.end_row();

                    ui.label("File size");
                    ui.label(format_bytes(info.file_size));
                    ui.end_row();

                    ui.label("Manifest");
                    ui.label(info.manifest_kind.to_string());
                    ui.end_row();

                    ui.label("Archive entries");
                    ui.label(info.entries.to_string());
                    ui.end_row();

                    ui.label("Uncompressed size");
                    ui.label(format_bytes(info.total_uncompressed_size));
                    ui.end_row();

                    ui.label("Asset files");
                    ui.label(info.asset_count.to_string());
                    ui.end_row();
                });

                ui.add_space(16.0);
                ui.heading(format!("DEX files ({})", info.dex_files.len()));
                if info.dex_files.is_empty() {
                    ui.label("No DEX files found.");
                } else {
                    for dex in &info.dex_files {
                        ui.monospace(dex);
                    }
                }

                ui.add_space(12.0);
                ui.heading(format!("Native libraries ({})", info.native_libraries.len()));
                if info.native_libraries.is_empty() {
                    ui.label("No native .so libraries found.");
                } else {
                    egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                        egui::Grid::new("native_libraries").striped(true).show(ui, |ui| {
                            ui.strong("ABI");
                            ui.strong("Library");
                            ui.strong("Size");
                            ui.end_row();
                            for library in &info.native_libraries {
                                ui.monospace(&library.abi);
                                ui.monospace(&library.file_name);
                                ui.label(format_bytes(library.size));
                                ui.end_row();
                            }
                        });
                    });
                }

                ui.add_space(12.0);
                ui.label("This tool inspects APK archives. It does not execute Android applications.");
            } else {
                ui.group(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(36.0);
                        ui.heading("No APK loaded");
                        ui.label("Choose an .apk file to inspect its manifest, DEX files and native ABIs.");
                        ui.add_space(16.0);
                        if ui.button("Browse for APK...").clicked() {
                            self.open_apk();
                        }
                        ui.add_space(28.0);
                    });
                });
            }
        });

        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.small("DroidCore 0.1.0");
                ui.separator();
                ui.small("Early development build · No Android runtime yet");
            });
        });
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.2} {}", UNITS[unit])
}
