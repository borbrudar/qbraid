mod braid;
mod fibonacci;
mod tree;
use braid::*;
use eframe::egui;
use egui::{Painter, Response};
use num_complex::Complex64;

use crate::{
    fibonacci::FusionBasis,
    tree::{FibStep, braid_to_fib_steps, compute_total},
};

#[derive(Default)]
enum Tab {
    #[default]
    General,
    Fibonacci3,
}

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
    braid: Braid,
    show_load_popup: bool,
    load_error: Option<String>,
    new_crossing: i32,

    tab: Tab,
    fib_steps: Vec<FibStep>,
}

impl BraidApp {
    fn new() -> Self {
        Self {
            time: 0.0,
            braid: Braid::new(),
            show_load_popup: false,
            load_error: None,
            new_crossing: 1,
            tab: Tab::General,
            fib_steps: vec![],
        }
    }
}

impl eframe::App for BraidApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top panel with modern MenuBar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.menu_button("Mode", |ui| {
                if ui.button("General braid").clicked() {
                    self.tab = Tab::General;
                    ui.close();
                }
                if ui.button("3-anyon Fibonacci").clicked() {
                    self.tab = Tab::Fibonacci3;
                    ui.close();
                }
            });

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
        egui::SidePanel::right("controls").show(ctx, |ui| {
            ui.heading("Braid Editor");

            if let Tab::Fibonacci3 = self.tab {
                self.braid.strands = 3;
                ui.label("Strands fixed to 3 (Fibonacci mode)");
            } else {
                if ui.button("Add strand").clicked() {
                    self.braid.strands += 1;
                }

                if ui.button("Remove strand").clicked() {
                    if self.braid.strands > 2 {
                        self.braid.strands -= 1;
                        self.braid
                            .crossings
                            .retain(|u| (u.abs() as u32) < self.braid.strands);
                    }
                }
            }

            ui.separator(); // visual spacing

            ui.horizontal(|ui| {
                let max_gen = self.braid.strands as i32 - 1;
                // Ensure new_gen is valid in current strands
                if self.new_crossing.abs() > max_gen || self.new_crossing == 0 {
                    self.new_crossing = 1;
                }

                ui.add(
                    egui::Slider::new(&mut self.new_crossing, -max_gen..=max_gen).text("generator"),
                );
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
            ui.columns(2, |columns| {
                // LEFT: braid
                columns[0].heading("Braid");

                egui::ScrollArea::both().show(&mut columns[0], |ui| {
                    let braid_size = egui::vec2(600.0, 1200.0);
                    let (rect, painter) = ui.allocate_painter(braid_size, egui::Sense::hover());
                    self.braid.draw(rect, painter);
                });

                // RIGHT: Fibonacci stuff
                columns[1].heading("Fibonacci Anyons");

                match self.tab {
                    Tab::General => {
                        columns[1].label("Switch to Fibonacci mode.");
                    }

                    Tab::Fibonacci3 => {
                        if self.braid.strands != 3 {
                            columns[1].label("This mode requires exactly 3 strands.");
                            return;
                        }

                        self.fib_steps = braid_to_fib_steps(&self.braid.crossings);

                        egui::ScrollArea::vertical().show(&mut columns[1], |ui| {
                            for (i, step) in self.fib_steps.iter().enumerate() {
                                ui.heading(format!("Step {}: {}", i, step.label));

                                // draw braid prefix
                                let temp_braid = Braid {
                                    strands: 3,
                                    crossings: step.braid_prefix.clone(),
                                };

                                let (rect, painter) = ui.allocate_painter(
                                    egui::vec2(200.0, 200.0),
                                    egui::Sense::hover(),
                                );

                                temp_braid.draw(rect, painter);

                                // draw fusion tree
                                self.draw_fusion_tree(ui, step);

                                ui.separator();
                            }

                            let total = compute_total(&self.fib_steps);

                            ui.heading("Final state vector:");
                            ui.monospace(format!("[{:.3}, {:.3}]", total[0], total[1]));
                        });
                    }
                }
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
impl BraidApp {
    fn _draw_braid_tab(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {}
    fn draw_fusion_tree(&self, ui: &mut egui::Ui, step: &FibStep) {
        let (resp, painter) = ui.allocate_painter(egui::vec2(220.0, 160.0), egui::Sense::hover());

        let rect = resp.rect;

        let top_y = rect.top() + 20.0;
        let mid_y = rect.center().y;
        let bot_y = rect.bottom() - 20.0;

        let x1 = rect.left() + 40.0;
        let x2 = rect.center().x;
        let x3 = rect.right() - 40.0;

        let p1 = egui::pos2(x1, top_y);
        let p2 = egui::pos2(x2, top_y);
        let p3 = egui::pos2(x3, top_y);

        match step.state.basis {
            FusionBasis::Left => {
                let mid = egui::pos2((x1 + x2) / 2.0, mid_y);
                let root = egui::pos2((mid.x + x3) / 2.0, bot_y);

                painter.line_segment([p1, mid], (2.0, egui::Color32::WHITE));
                painter.line_segment([p2, mid], (2.0, egui::Color32::WHITE));
                painter.line_segment([mid, root], (2.0, egui::Color32::WHITE));
                painter.line_segment([p3, root], (2.0, egui::Color32::WHITE));

                Self::draw_channel(&painter, mid, &step.state.vec);
            }
            FusionBasis::Right => {
                let mid = egui::pos2((x2 + x3) / 2.0, mid_y);
                let root = egui::pos2((x1 + mid.x) / 2.0, bot_y);

                painter.line_segment([p2, mid], (2.0, egui::Color32::WHITE));
                painter.line_segment([p3, mid], (2.0, egui::Color32::WHITE));
                painter.line_segment([mid, root], (2.0, egui::Color32::WHITE));
                painter.line_segment([p1, root], (2.0, egui::Color32::WHITE));

                Self::draw_channel(&painter, mid, &step.state.vec);
            }
        }

        for p in [p1, p2, p3] {
            painter.text(
                p,
                egui::Align2::CENTER_CENTER,
                "τ",
                egui::FontId::proportional(14.0),
                egui::Color32::WHITE,
            );
        }
    }

    fn draw_channel(painter: &egui::Painter, pos: egui::Pos2, v: &[Complex64; 2]) {
        let prob0 = v[0].norm_sqr();
        let prob1 = v[1].norm_sqr();

        let text = format!("1:{:.2} τ:{:.2}", prob0, prob1);

        painter.text(
            pos,
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(13.0),
            egui::Color32::LIGHT_BLUE,
        );
    }
}
