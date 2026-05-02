mod braid;
mod fibonacci;
mod tree;

use braid::*;
use eframe::egui;
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
    braid: Braid,
    new_crossing: i32,

    tab: Tab,
    fib_steps: Vec<FibStep>,
}

impl BraidApp {
    fn new() -> Self {
        Self {
            braid: Braid::new(),
            new_crossing: 1,
            tab: Tab::General,
            fib_steps: vec![],
        }
    }
}

impl eframe::App for BraidApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // TOP BAR
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("General").clicked() {
                    self.tab = Tab::General;
                }
                if ui.button("Fibonacci (3 anyons)").clicked() {
                    self.tab = Tab::Fibonacci3;
                }
            });
        });

        // SIDE PANEL
        egui::SidePanel::right("controls").show(ctx, |ui| {
            ui.heading("Braid Editor");

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

        // MAIN PANEL
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |columns| {
                // LEFT: full braid
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

                        egui::ScrollArea::vertical().show(&mut columns[1], |ui| {
                            for (i, step) in self.fib_steps.iter().enumerate() {
                                ui.push_id(format!("step_{}", i), |ui| {
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

                                    self.draw_braid_in_rect(
                                        &painter,
                                        braid_rect,
                                        &step.braid_remaining,
                                    );

                                    self.draw_tree(&painter, tree_rect, step);

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

impl BraidApp {
    fn draw_braid_in_rect(&self, painter: &egui::Painter, rect: egui::Rect, crossings: &[i32]) {
        let n = 3;

        let steps = crossings.len();
        let steps = steps.max(1);

        let vgap = rect.height() / (steps as f32 + 1.0);
        let hgap = rect.width() / (n as f32 + 1.0);

        // current x positions of strands
        let mut x: Vec<f32> = (0..n)
            .map(|i| rect.left() + (i as f32 + 1.0) * hgap)
            .collect();

        for (j, &g) in crossings.iter().enumerate() {
            let y0 = rect.top() + (j as f32 + 1.0) * vgap;
            let y1 = rect.top() + (j as f32 + 2.0) * vgap;

            let idx = (g.abs() - 1) as usize; // IMPORTANT FIX

            if idx + 1 >= n {
                continue;
            }

            let (a, b) = (idx, idx + 1);

            let dir = if g > 0 { 1.0 } else { -1.0 };

            // swap visual strands
            let xa = x[a];
            let xb = x[b];

            let mid_a = egui::pos2(xa + dir * hgap * 0.5, y0 + vgap * 0.5);
            let mid_b = egui::pos2(xb - dir * hgap * 0.5, y0 + vgap * 0.5);

            painter.line_segment([egui::pos2(xa, y0), mid_a], (2.0, egui::Color32::WHITE));
            painter.line_segment([egui::pos2(xb, y0), mid_b], (2.0, egui::Color32::WHITE));

            painter.line_segment([mid_a, egui::pos2(xb, y1)], (2.0, egui::Color32::WHITE));
            painter.line_segment([mid_b, egui::pos2(xa, y1)], (2.0, egui::Color32::WHITE));

            // swap internal state
            x.swap(a, b);
        }

        // final verticals
        for i in 0..n {
            painter.line_segment(
                [
                    egui::pos2(x[i], rect.top()),
                    egui::pos2(x[i], rect.bottom()),
                ],
                (2.0, egui::Color32::DARK_GRAY),
            );
        }
    }
    fn draw_tree(&self, painter: &egui::Painter, rect: egui::Rect, step: &FibStep) {
        let top_y = rect.top() + 5.0;
        let mid_y = rect.center().y;
        let bot_y = rect.bottom();

        let x1 = rect.left() + rect.width() * 0.25;
        let x2 = rect.center().x;
        let x3 = rect.right() - rect.width() * 0.25;

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

                self.draw_channel(painter, mid, &step.state.vec);
            }
            FusionBasis::Right => {
                let mid = egui::pos2((x2 + x3) / 2.0, mid_y);
                let root = egui::pos2((x1 + mid.x) / 2.0, bot_y);

                painter.line_segment([p2, mid], (2.0, egui::Color32::WHITE));
                painter.line_segment([p3, mid], (2.0, egui::Color32::WHITE));
                painter.line_segment([mid, root], (2.0, egui::Color32::WHITE));
                painter.line_segment([p1, root], (2.0, egui::Color32::WHITE));

                self.draw_channel(painter, mid, &step.state.vec);
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

    fn draw_channel(&self, painter: &egui::Painter, pos: egui::Pos2, v: &[Complex64; 2]) {
        let p0 = v[0].norm_sqr();
        let p1 = v[1].norm_sqr();

        painter.text(
            pos,
            egui::Align2::CENTER_CENTER,
            format!("1:{:.2} τ:{:.2}", p0, p1),
            egui::FontId::proportional(12.0),
            egui::Color32::LIGHT_BLUE,
        );
    }
}
