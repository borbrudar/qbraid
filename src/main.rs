mod braid;
use braid::*;
use eframe::egui;
use egui::{Painter, Response};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Topological Braid Visualizer",
        options,
        Box::new(|_cc| Ok(Box::new(BraidApp::new()))),
    )
}

#[derive(Default)]
struct BraidApp {
    time: f32,
    braid : Braid,
    show_load_popup : bool,
    load_error : Option<String>,
    new_crossing : i32,
}

impl BraidApp{
    fn new() -> Self {
        Self {
            time: 0.0,
            braid: Braid::new(),
            show_load_popup : false,
            load_error : None,
            new_crossing : 1,
        }
    }
}

impl eframe::App for BraidApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top panel with modern MenuBar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::containers::menu::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Load braid…").clicked() {
                        self.show_load_popup = true;
                        ui.close();
                    }

                    if ui.button("Save braid…").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_file_name("my_braid.braid")
                            .add_filter("Braid files", &["braid"])
                            .save_file()
                        {
                            if let Err(e) = Braid::save_braid_to_file(&self.braid, &path) {
                                self.load_error = Some(e.to_string());
                            }
                        }
                        ui.close();
                    }

                    if ui.button("Quit").clicked() {
                        std::process::exit(0);
                    }
                });
                ui.menu_button("View", |ui| {
                    if ui.button("Reset").clicked() {
                        self.time = 0.0;
                    }
                });
            });
        });

        // braid operations
        egui::SidePanel::right("controls").show(ctx,|ui|{
            ui.heading("Braid Editor");

            if ui.button("Add strand").clicked() {
                self.braid.strands += 1;
            }
        
            if ui.button("Remove strand").clicked() {
                if self.braid.strands > 2 {
                    self.braid.strands -= 1;
                    let mut new_vec = Vec::new();
                    for u in &self.braid.crossings {
                        if (u.abs() as u32) < self.braid.strands {
                            new_vec.push(u.clone());
                        }
                    }
                    self.braid.crossings = new_vec;
                }
            }
        
            ui.separator(); // visual spacing
        
            ui.horizontal(|ui| {
                let max_gen = self.braid.strands as i32 - 1;
                // Ensure new_gen is valid in current strands
                if self.new_crossing.abs() > max_gen || self.new_crossing == 0 {
                    self.new_crossing = 1;
                }

                ui.add(egui::Slider::new(&mut self.new_crossing, -max_gen..=max_gen).text("generator"));
                if ui.button("Add crossing").clicked() {
                    let max_gen = self.braid.strands as i32 - 1;
                    if self.new_crossing != 0 && self.new_crossing.abs() <= max_gen {
                        self.braid.crossings.push(self.new_crossing);
                    }
                }
            });
            
        
            if ui.button("Remove last crossing").clicked() {
                self.braid.crossings.pop();
            }
        
            ui.separator();
            ui.label(format!("Strands: {}", self.braid.strands));
            ui.label(format!("Crossings: {}", self.braid.crossings.len()));
        });

        // Central content
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                let braid_size = egui::vec2(700.0, 1500.0);
            
                let (rect, painter) =
                    ui.allocate_painter(braid_size, egui::Sense::hover());
                
                self.braid.draw(rect, painter);
            });
            
        });

        // braid loading
        if self.show_load_popup {
            egui::Window::new("Load braid")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Load a braid from a text file.");
        
                    if ui.button("Choose file…").clicked() {
                        match Braid::load_braid_from_file() {
                            Ok(braid) => {
                                self.braid = braid;
                                self.show_load_popup = false;
                                self.load_error = None;
                            }
                            Err(e) => {
                                self.load_error = Some(e);
                            }
                        }
                    }
        
                    if let Some(err) = &self.load_error {
                        ui.colored_label(egui::Color32::RED, err);
                    }
        
                    if ui.button("Cancel").clicked() {
                        self.show_load_popup = false;
                        self.load_error = None;
                    }
                });
        }
        
    }
}
