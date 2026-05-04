mod braid;
mod fibonacci;
mod tree;
mod search;

use braid::*;
use eframe::egui;
use num_complex::Complex64;
use search::*;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
    time::Instant,
};

use crate::tree::evaluate_braid;

const TEXTSIZE: f32 = 40.0;

#[derive(Default, PartialEq)]
enum Tab {
    #[default]
    General,
    Fibonacci3,
}

// =======================
// COMPLEX INPUT UI TYPE
// =======================
#[derive(Clone, Default)]
struct CInput {
    re: f32,
    im: f32,
}

impl CInput {
    fn to_c(&self) -> Complex64 {
        Complex64::new(self.re as f64, self.im as f64)
    }
}

// =======================
// SEARCH STATE
// =======================
struct SearchState {
    target: [[CInput; 2]; 2],
    depth: usize,

    searching: bool,
    stop: Arc<AtomicBool>,

    start: Option<Instant>,
    elapsed: f32,

    rx: Option<mpsc::Receiver<Result<SearchResult, String>>>,
    error: Option<String>,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            target: [
                [
                    CInput { re: 1.0, im: 0.0 },
                    CInput { re: 0.0, im: 0.0 },
                ],
                [
                    CInput { re: 0.0, im: 0.0 },
                    CInput { re: 1.0, im: 0.0 },
                ],
            ],
            depth: 6,
            searching: false,
            stop: Arc::new(AtomicBool::new(false)),
            start: None,
            elapsed: 0.0,
            rx: None,
            error: None,
        }
    }
}

// =======================
// APP
// =======================
#[derive(Default)]
struct BraidApp {
    braid: Braid,
    new_crossing: i32,

    tab: Tab,
    load_error: Option<String>,

    search: SearchState,
}

impl BraidApp {
    fn new() -> Self {
        Self {
            braid: Braid::new(),
            new_crossing: 0,
            tab: Tab::General,
            load_error: None,
            search: SearchState::default(),
        }
    }

    // =======================
    // START SEARCH THREAD
    // =======================
    fn start_search(&mut self) {
        let target = self.search.target.clone().map(|row| {
            row.map(|c| c.to_c())
        });

        let depth = self.search.depth;
        let stop = self.search.stop.clone();

        let (tx, rx) = mpsc::channel();

        self.search.searching = true;
        self.search.stop.store(false, Ordering::Relaxed);
        self.search.start = Some(Instant::now());
        self.search.rx = Some(rx);
        self.search.error = None;

        thread::spawn(move || {
            let res = find_braid(target, depth, stop);
            let _ = tx.send(res);
        });
    }
}

// =======================
// EGUI APP
// =======================
impl eframe::App for BraidApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // update timer
        if self.search.searching {
            if let Some(t) = self.search.start {
                self.search.elapsed = t.elapsed().as_secs_f32();
            }
            ctx.request_repaint();
        }

// receive result
if let Some(rx) = &self.search.rx {
    if let Ok(result) = rx.try_recv() {
        self.search.searching = false;

        match result {
            Ok(res) => {
                self.braid = Braid {
                    strands: 3,
                    crossings: res.word,
                };

                self.search.error = Some(format!(
                    "Best error (distance): {:.6}",
                    res.distance
                ));
            }
            Err(e) => self.search.error = Some(e),
        }
    }
}

        // =======================
        // TOP BAR
        // =======================
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab, Tab::General, "General");
                ui.selectable_value(&mut self.tab, Tab::Fibonacci3, "Fibonacci");

                ui.separator();

                if ui.button("Load").clicked() {
                    match Braid::load_braid_from_file() {
                        Ok(b) => self.braid = b,
                        Err(e) => self.load_error = Some(e),
                    }
                }

                if ui.button("Save").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_file_name("braid.braid")
                        .save_file()
                    {
                        let _ = Braid::save_braid_to_file(&self.braid, &path);
                    }
                }
            });
        });

        // =======================
        // SIDE PANEL
        // =======================
        egui::SidePanel::right("side").show(ctx, |ui| {
            ui.heading("Controls");

            if let Some(e) = &self.load_error {
                ui.colored_label(egui::Color32::RED, e);
            }

            if let Some(e) = &self.search.error {
                ui.colored_label(egui::Color32::YELLOW, e);
            }

            if self.tab != Tab::Fibonacci3 {
                if ui.button("Add strand").clicked() {
                    self.braid.strands += 1;
                }

                if ui.button("Remove strand").clicked() {
                    if self.braid.strands > 2 {
                        self.braid.strands -= 1;
                        self.braid.crossings.retain(|g| (g.abs() as u32) < self.braid.strands);
                    }
                }
            } else {
                self.braid.strands = 3;
                ui.label("3 strands fixed");
            }

            ui.separator();

            let max = self.braid.strands as i32 - 1;
            ui.add(egui::Slider::new(&mut self.new_crossing, -max..=max));

            if ui.button("Add crossing").clicked() {
                if self.new_crossing != 0 {
                    self.braid.crossings.push(self.new_crossing);
                }
            }

            if ui.button("Undo").clicked() {
                self.braid.crossings.pop();
            }

            ui.separator();

            // =======================
            // SEARCH UI
            // =======================
            ui.heading("Search target");

            for i in 0..2 {
                for j in 0..2 {
                    ui.horizontal(|ui| {
                        ui.label(format!("[{},{}]", i, j));
                        ui.add(egui::DragValue::new(&mut self.search.target[i][j].re).speed(0.01));
                        ui.add(egui::DragValue::new(&mut self.search.target[i][j].im).speed(0.01));
                    });
                }
            }

            if ui.button("Random unitary").clicked() {
                let unitary = search::random_unitary();
                self.search.target = unitary.map(|row| {
                    row.map(|c| CInput {
                        re: c.re as f32,
                        im: c.im as f32,
                    })
                });
            }

            ui.add(egui::Slider::new(&mut self.search.depth, 1..=11).text("depth"));

            if !self.search.searching {
                if ui.button("Run search").clicked() {
                    self.start_search();
                }
            } else {
                if ui.button("Stop").clicked() {
                    self.search.stop.store(true, Ordering::Relaxed);
                    self.search.searching = false;
                }

                ui.label(format!("Searching... {:.2}s", self.search.elapsed));
            }

            ui.separator();

            ui.label(format!("Strands: {}", self.braid.strands));
            ui.label(format!("Crossings: {}", self.braid.crossings.len()));
        });

        // =======================
        // MAIN VIEW
        // =======================
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |cols| {
                // LEFT: braid always visible
                cols[0].heading("Braid");

                egui::ScrollArea::both().show(&mut cols[0], |ui| {
                    let (r, p) = ui.allocate_painter(egui::vec2(600.0, 1200.0), egui::Sense::hover());
                    self.braid.draw(r, p);
                });

                // RIGHT: result
                cols[1].label(egui::RichText::new("Result").size(TEXTSIZE).strong());

                let res = evaluate_braid(&self.braid.crossings);

                let f = egui::FontId::proportional(TEXTSIZE);

                cols[1].label(egui::RichText::new("Raw").size(TEXTSIZE).strong());
                cols[1].label(
                    egui::RichText::new(format!(
                        "[[{:.4}, {:.4}],\n [{:.4}, {:.4}]]",
                        res.raw[0][0],
                        res.raw[0][1],
                        res.raw[1][0],
                        res.raw[1][1],
                    ))
                    .font(f.clone()),
                );

                cols[1].separator();

                cols[1].label(egui::RichText::new("Normalized").size(TEXTSIZE).strong());
                cols[1].label(
                    egui::RichText::new(format!(
                        "[[{:.4}, {:.4}],\n [{:.4}, {:.4}]]",
                        res.normalized[0][0],
                        res.normalized[0][1],
                        res.normalized[1][0],
                        res.normalized[1][1],
                    ))
                    .font(f),
                );
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1440.0, 900.0]),
        ..Default::default()
    };
    eframe::run_native(
        "qbraid",
        options,
        Box::new(|_cc| Ok(Box::<BraidApp>::default())),
    )
}