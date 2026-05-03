mod braid;
mod fibonacci;
mod tree;

use braid::*;
use eframe::egui;
use num_complex::Complex64;
use std::path::Path;

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
    braid: Braid,
    new_crossing: i32,

    tab: Tab,
    fib_steps: Vec<FibStep>,

    // NEW: UI state
    load_error: Option<String>,
}

impl BraidApp {
    fn new() -> Self {
        Self {
            braid: Braid::new(),
            new_crossing: 1,
            tab: Tab::General,
            fib_steps: vec![],
            load_error: None,
        }
    }

    fn draw_tree(&mut self, painter: &egui::Painter, rect: egui::Rect, step: &FibStep) {
        // Draw the fusion tree visualization
        painter.rect_filled(rect, 0.0, egui::Color32::WHITE);
        painter.rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::BLACK),
            egui::StrokeKind::Middle,
        );
    }

    fn draw_braid_in_rect(&mut self, painter: &egui::Painter, rect: egui::Rect, braid: &[i32]) {
        // Draw the braid in the specified rectangle
        painter.rect_filled(rect, 0.0, egui::Color32::WHITE);
        painter.rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::BLACK),
            egui::StrokeKind::Middle,
        );
    }
}

impl eframe::App for BraidApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // =======================
        // TOP BAR
        // =======================
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("General").clicked() {
                    self.tab = Tab::General;
                }
                if ui.button("Fibonacci (3 anyons)").clicked() {
                    self.tab = Tab::Fibonacci3;
                }

                ui.separator();

                // =======================
                // LOAD / SAVE RESTORED
                // =======================
                if ui.button("Load braid").clicked() {
                    match Braid::load_braid_from_file() {
                        Ok(b) => {
                            self.braid = b;
                            self.load_error = None;
                        }
                        Err(e) => {
                            self.load_error = Some(e);
                        }
                    }
                }

                if ui.button("Save braid").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_file_name("braid.braid")
                        .add_filter("braid", &["braid"])
                        .save_file()
                    {
                        if let Err(e) = Braid::save_braid_to_file(&self.braid, &path) {
                            self.load_error = Some(e);
                        }
                    }
                }
            });
        });

        // =======================
        // SIDE PANEL
        // =======================
        egui::SidePanel::right("controls").show(ctx, |ui| {
            ui.heading("Braid Editor");

            if let Some(err) = &self.load_error {
                ui.colored_label(egui::Color32::RED, err);
            }

            if let Tab::Fibonacci3 = self.tab {
                self.braid.strands = 3;
                ui.label("Strands fixed to 3");
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

            ui.separator();

            ui.horizontal(|ui| {
                let max_gen = self.braid.strands as i32 - 1;

                if self.new_crossing.abs() > max_gen || self.new_crossing == 0 {
                    self.new_crossing = 1;
                }

                ui.add(
                    egui::Slider::new(&mut self.new_crossing, -max_gen..=max_gen).text("generator"),
                );

                if ui.button("Add").clicked() {
                    if self.new_crossing != 0 && self.new_crossing.abs() <= max_gen {
                        self.braid.crossings.push(self.new_crossing);
                    }
                }
            });

            if ui.button("Undo").clicked() {
                self.braid.crossings.pop();
            }

            ui.separator();
            ui.label(format!("Strands: {}", self.braid.strands));
            ui.label(format!("Crossings: {}", self.braid.crossings.len()));
        });

        // =======================
        // MAIN PANEL
        // =======================
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |columns| {
                // LEFT
                columns[0].heading("Full braid");

                egui::ScrollArea::both().show(&mut columns[0], |ui| {
                    let size = egui::vec2(600.0, 1200.0);
                    let (resp, painter) = ui.allocate_painter(size, egui::Sense::hover());
                    self.braid.draw(resp, painter);
                });

                // RIGHT
                columns[1].heading("Fusion evolution");

                match self.tab {
                    Tab::General => {
                        columns[1].label("Switch to Fibonacci mode.");
                    }

                    Tab::Fibonacci3 => {
                        if self.braid.strands != 3 {
                            columns[1].colored_label(egui::Color32::RED, "Need exactly 3 strands");
                            return;
                        }

                        self.fib_steps = braid_to_fib_steps(&self.braid.crossings);

                        let steps = self.fib_steps.clone();
                        egui::ScrollArea::vertical().show(&mut columns[1], |ui| {
                            for (i, step) in steps.iter().enumerate() {
                                ui.push_id(format!("step_{i}"), |ui| {
                                    ui.heading(format!("Step {}: {}", i, step.label));

                                    let (resp, painter) = ui.allocate_painter(
                                        egui::vec2(260.0, 340.0),
                                        egui::Sense::hover(),
                                    );

                                    let rect = resp.rect;
                                    let split = rect.top() + rect.height() * 0.55;

                                    let braid_rect = egui::Rect::from_min_max(
                                        rect.min,
                                        egui::pos2(rect.max.x, split),
                                    );

                                    let tree_rect = egui::Rect::from_min_max(
                                        egui::pos2(rect.min.x, split),
                                        rect.max,
                                    );

                                    //self.draw_braid_in_rect(
                                    //    &painter,
                                    //    braid_rect,
                                    //    &step.braid_remaining,
                                    //);
//
                                    //self.draw_tree(&painter, tree_rect, step);

                                    ui.monospace(format!(
                                        "[[{:.2}, {:.2}],\n [{:.2}, {:.2}]]",
                                        step.matrix[0][0],
                                        step.matrix[0][1],
                                        step.matrix[1][0],
                                        step.matrix[1][1],
                                    ));

                                    ui.separator();
                                });
                            }

                            let total = compute_total(&self.fib_steps);
                            println!(
                                "[[{:.3}, {:.3}],\n [{:.3}, {:.3}]]",
                                total[0][0], total[0][1], total[1][0], total[1][1],
                            );
                            ui.heading("Final matrix:");
                            ui.monospace(format!(
                                "[[{:.3}, {:.3}],\n [{:.3}, {:.3}]]",
                                total[0][0], total[0][1], total[1][0], total[1][1],
                            ));
                        });
                    }
                }
            });
        });
    }
}
