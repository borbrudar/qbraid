mod braid;
mod fibonacci;
mod tree;

use braid::*;
use eframe::egui;
use tree::*;

const TEXTSIZE: f32 = 40.0;

#[derive(Default, PartialEq)]
enum Tab {
    #[default]
    General,
    Fibonacci3,
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "qbraid",
        options,
        Box::new(|_cc| Ok(Box::new(BraidApp::new()))),
    )
}

struct BraidApp {
    braid: Braid,
    new_crossing: i32,

    tab: Tab,
    load_error: Option<String>,
}

impl Default for BraidApp {
    fn default() -> Self {
        Self::new()
    }
}

impl BraidApp {
    fn new() -> Self {
        Self {
            braid: Braid::new(),
            new_crossing: 1,
            tab: Tab::General,
            load_error: None,
        }
    }
}

impl eframe::App for BraidApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // =======================
        // TOP BAR
        // =======================
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab, Tab::General, "General");
                ui.selectable_value(&mut self.tab, Tab::Fibonacci3, "Fibonacci (3 anyons)");

                ui.separator();

                if ui.button("Load").clicked() {
                    match Braid::load_braid_from_file() {
                        Ok(b) => {
                            self.braid = b;
                            self.load_error = None;
                        }
                        Err(e) => self.load_error = Some(e),
                    }
                }

                if ui.button("Save").clicked() {
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

            if self.tab != Tab::Fibonacci3 {
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
            } else {
                self.braid.strands = 3;
                ui.label("Strands fixed to 3");
            }

            ui.separator();

            let max_gen = self.braid.strands as i32 - 1;

            ui.add(egui::Slider::new(&mut self.new_crossing, -max_gen..=max_gen).text("generator"));

            if ui.button("Add crossing").clicked() {
                if self.new_crossing != 0 {
                    self.braid.crossings.push(self.new_crossing);
                }
            }

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
            ui.columns(2, |cols| {
                // levo: braid
                cols[0].heading("Braid");

                egui::ScrollArea::both().show(&mut cols[0], |ui| {
                    let size = egui::vec2(600.0, 1200.0);
                    let (resp, painter) = ui.allocate_painter(size, egui::Sense::hover());
                    self.braid.draw(resp, painter);
                });

                // desno: rezultat
                cols[1].label(
                    egui::RichText::new("Result")
                        .size(TEXTSIZE)
                        .strong(),
                );

                match self.tab {
                    Tab::General => {
                        cols[1].label("Switch to Fibonacci mode for matrix evaluation.");
                    }

                    Tab::Fibonacci3 => {
                        if self.braid.strands != 3 {
                            cols[1].colored_label(egui::Color32::RED, "Need exactly 3 strands");
                            return;
                        }

                        let result = evaluate_braid(&self.braid.crossings);

                        let big_font = egui::FontId::proportional(TEXTSIZE);

                        cols[1].label(egui::RichText::new("Raw matrix:").size(TEXTSIZE).strong());
                        cols[1].label(
                            egui::RichText::new(format!(
                                "[[{:.4}, {:.4}],\n [{:.4}, {:.4}]]",
                                result.raw[0][0],
                                result.raw[0][1],
                                result.raw[1][0],
                                result.raw[1][1],
                            ))
                            .font(big_font.clone()),
                        );

                        cols[1].separator();

                        cols[1].label(
                            egui::RichText::new("Normalized:")
                                .size(TEXTSIZE)
                                .strong(),
                        );
                        cols[1].label(
                            egui::RichText::new(format!(
                                "[[{:.4}, {:.4}],\n [{:.4}, {:.4}]]",
                                result.normalized[0][0],
                                result.normalized[0][1],
                                result.normalized[1][0],
                                result.normalized[1][1],
                            ))
                            .font(big_font),
                        );
                    }
                }
            });
        });
    }
}
