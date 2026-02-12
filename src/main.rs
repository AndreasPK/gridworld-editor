mod app_state;
mod dna_widget;
mod dnaparser;
mod pdf_infos;

use std::path::Path;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use eframe::egui;

use crate::{app_state::AppState, dna_widget::DnaWidget, dnaparser::CreatureDNA};

const DATA_DIR: &str = "gridworld-editor";

fn on_start() -> AppState {
    std::fs::read(format!("{DATA_DIR}/state.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<AppState>(&bytes).ok())
        .unwrap_or_default()
}

fn on_exit(app_state: &AppState) {
    let data_dir = Path::new(DATA_DIR);
    if std::fs::create_dir_all(data_dir).is_err() {
        return;
    }

    let state_path = data_dir.join("state.json");
    if let Ok(bytes) = serde_json::to_vec(app_state) {
        let _ = std::fs::write(state_path, bytes);
    }
}

fn load_creature<P: AsRef<Path>>(
    app_state: &mut AppState,
    filepath: P,
    dna: &mut Option<CreatureDNA>,
) -> Result<(), String> {
    let path = filepath.as_ref();
    let content = std::fs::read_to_string(path)
        .map_err(|err| format!("failed to read '{}': {err}", path.display()))?;
    let decoded = dnaparser::parse_creature_dna(&content)?;

    app_state.open_file = Some(path.to_path_buf());
    app_state.last_folder = path.parent().map(Path::to_path_buf);
    *dna = Some(decoded);

    Ok(())
}

fn save_creature<P: AsRef<Path>>(filepath: P, dna: &CreatureDNA) -> Result<(), String> {
    let path = filepath.as_ref();
    let content = dna.to_text();
    std::fs::write(path, content)
        .map_err(|err| format!("failed to write '{}': {err}", path.display()))
}

#[test]
fn test_load_creature() {
    let mut dna = None;
    let mut app_state = AppState::default();
    assert!(load_creature(&mut app_state, "data/e5.txt", &mut dna).is_ok())
}

struct GridworldApp {
    app_state: AppState,
    creature_dna: Option<CreatureDNA>,
    dna_widget: DnaWidget,
    status_message: Option<String>,
    shutdown_requested: Arc<AtomicBool>,
}

impl GridworldApp {
    fn new(shutdown_requested: Arc<AtomicBool>) -> Self {
        let mut app_state = on_start();
        let mut creature_dna = None;
        let mut status_message = None;

        if let Some(last_open_file) = app_state.open_file.clone() {
            if let Err(err) = load_creature(&mut app_state, &last_open_file, &mut creature_dna) {
                status_message = Some(format!("Failed to open last file: {err}"));
            }
        }

        Self {
            app_state,
            creature_dna,
            dna_widget: DnaWidget::new(),
            status_message,
            shutdown_requested,
        }
    }

    fn open_file_dialog(&mut self) {
        let mut dialog = rfd::FileDialog::new();
        if let Some(last_folder) = self.app_state.last_folder.as_ref() {
            dialog = dialog.set_directory(last_folder);
        }

        if let Some(path) = dialog.pick_file() {
            match load_creature(&mut self.app_state, &path, &mut self.creature_dna) {
                Ok(()) => {
                    self.dna_widget.refresh_from_dna();
                    self.status_message = Some(format!("Loaded {}", path.display()));
                }
                Err(err) => {
                    self.status_message = Some(format!("Failed to load file: {err}"));
                }
            }
        }
    }

    fn save_to_path(&mut self, path: &Path) {
        let Some(dna) = self.creature_dna.as_ref() else {
            self.status_message = Some("No DNA loaded to save.".to_string());
            return;
        };

        match save_creature(path, dna) {
            Ok(()) => {
                self.status_message = Some(format!("Saved {}", path.display()));
                self.app_state.open_file = Some(path.to_path_buf());
                self.app_state.last_folder = path.parent().map(Path::to_path_buf);
            }
            Err(err) => {
                self.status_message = Some(format!("Failed to save file: {err}"));
            }
        }
    }

    fn save_current_file(&mut self) {
        let Some(path) = self.app_state.open_file.clone() else {
            self.status_message = Some("No open file to save.".to_string());
            return;
        };
        self.save_to_path(&path);
    }

    fn save_as_file_dialog(&mut self) {
        if self.creature_dna.is_none() {
            self.status_message = Some("No DNA loaded to save.".to_string());
            return;
        }

        let mut dialog = rfd::FileDialog::new();
        if let Some(last_folder) = self.app_state.last_folder.as_ref() {
            dialog = dialog.set_directory(last_folder);
        }
        if let Some(open_file) = self.app_state.open_file.as_ref() {
            if let Some(parent) = open_file.parent() {
                dialog = dialog.set_directory(parent);
            }
            if let Some(file_name) = open_file.file_name().and_then(|n| n.to_str()) {
                dialog = dialog.set_file_name(file_name);
            }
        }

        if let Some(path) = dialog.save_file() {
            self.save_to_path(&path);
        }
    }
}

impl Drop for GridworldApp {
    fn drop(&mut self) {
        on_exit(&self.app_state);
    }
}

impl eframe::App for GridworldApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.shutdown_requested.load(Ordering::Relaxed) {
            self.status_message = Some("Received Ctrl+C, shutting down...".to_string());
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        ui.close();
                        self.open_file_dialog();
                    }
                    if ui.button("Save").clicked() {
                        self.save_current_file();
                        ui.close();
                    }
                    if ui.button("Save As...").clicked() {
                        self.save_as_file_dialog();
                        ui.close();
                    }
                    if ui.button("Quit").clicked() {
                        ui.close();
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Help", |ui| {
                    ui.label("Gridworld Editor");
                });

                ui.separator();
                if let Some(msg) = self.status_message.as_deref() {
                    ui.label(msg);
                }
            });
        });

        egui::SidePanel::left("sidebar")
            .default_width(280.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Sidebar");
                ui.separator();
                self.dna_widget.sidebar_ui(ui, self.creature_dna.as_ref());
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Sub Frame");
            ui.separator();
            self.dna_widget.detail_ui(ui, self.creature_dna.as_mut());
        });
    }
}

fn main() -> eframe::Result<()> {
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let shutdown_requested_for_handler = Arc::clone(&shutdown_requested);
    let _ = ctrlc::set_handler(move || {
        shutdown_requested_for_handler.store(true, Ordering::Relaxed);
        std::process::abort();
    });

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 800.0])
            .with_min_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Gridworld Editor",
        native_options,
        Box::new(move |_cc| Ok(Box::new(GridworldApp::new(Arc::clone(&shutdown_requested))))),
    )
}
