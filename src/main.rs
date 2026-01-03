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
    load_error : Option<String>
}

impl BraidApp{
    fn new() -> Self {
        Self {
            time: 0.0,
            braid: Braid::new(),
            show_load_popup : false,
            load_error : None,
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


#[derive(Default)]
struct Braid{
    strands : u32,
    crossings : Vec<i32>,
}

impl Braid{
    fn new() -> Braid {
        Braid::init(10)
    }
    fn init(strands : u32) -> Braid{
        Braid{
            strands : strands,
            crossings : vec![2,-3,4,5,5,-2,-5,-4,9,-9],
        }
    }

    fn draw(&self, drawing_area : Response, painter : Painter ){
        for j in 1..(self.crossings.len() + 2){
            for i in 1..(self.strands+1){    
                let vertical_gap = drawing_area.rect.height()/(self.crossings.len() + 3) as f32;
                let horizontal_gap = drawing_area.rect.width()/ (self.strands+2) as f32;
                let x_pos = i as f32 * horizontal_gap + drawing_area.rect.left_top().x;
                let y_pos = j as f32 * vertical_gap + drawing_area.rect.left_top().y;
                let center = egui::Pos2::new(x_pos, y_pos);
                painter.circle_filled(center, 5.0, egui::Color32::WHITE);

                
                if j-1 >= self.crossings.len() { continue;}
                if self.crossings[j-1].abs() as u32 == i {
                    // connect to the next one
                    if self.crossings[j-1] > 0{
                        painter.line_segment(
                            [
                                egui::pos2(x_pos, y_pos),
                                egui::pos2(x_pos + horizontal_gap, y_pos + vertical_gap),
                                ],
                                egui::Stroke::new(2.0, egui::Color32::WHITE),
                        );
                    }
                    else{
                        painter.line_segment(
                            [
                                egui::pos2(x_pos, y_pos),
                                egui::pos2(x_pos + horizontal_gap/3.0, y_pos + vertical_gap/3.0),
                                ],
                                egui::Stroke::new(2.0, egui::Color32::WHITE),
                        );
                        painter.line_segment(
                            [
                                egui::pos2(x_pos + horizontal_gap*2.0/3.0, y_pos + vertical_gap*2.0/3.0),
                                egui::pos2(x_pos + horizontal_gap, y_pos + vertical_gap),
                                ],
                                egui::Stroke::new(2.0, egui::Color32::WHITE),
                        );
                    }
                }
                else if self.crossings[j-1].abs() as u32 == i-1 {
                    // connect to the previous one
                    if self.crossings[j-1] < 0 {

                        painter.line_segment(
                            [
                                egui::pos2(x_pos, y_pos),
                                egui::pos2(x_pos - horizontal_gap, y_pos + vertical_gap),
                                ],
                                egui::Stroke::new(2.0, egui::Color32::WHITE),
                            );
                    } else{
                        painter.line_segment(
                            [
                                egui::pos2(x_pos, y_pos),
                                egui::pos2(x_pos - horizontal_gap/3.0, y_pos + vertical_gap/3.0),
                                ],
                                egui::Stroke::new(2.0, egui::Color32::WHITE),
                            );

                            painter.line_segment(
                                [
                                    egui::pos2(x_pos - horizontal_gap *2.0/3.0, y_pos + vertical_gap*2.0/3.0),
                                    egui::pos2(x_pos - horizontal_gap, y_pos + vertical_gap),
                                    ],
                                    egui::Stroke::new(2.0, egui::Color32::WHITE),
                                );
                        }
                    }
                else {
                    // straight
                    painter.line_segment(
                    [
                        egui::pos2(x_pos, y_pos),
                        egui::pos2(x_pos, y_pos + vertical_gap),
                        ],
                        egui::Stroke::new(2.0, egui::Color32::WHITE),
                    );
             }
         }
        }



    }

    fn load_braid_from_file() -> Result<Braid, String> {
        let file = rfd::FileDialog::new()
            .add_filter("Braid file", &["braid"])
            .pick_file()
            .ok_or("No file selected")?;
    
        let text = std::fs::read_to_string(file)
            .map_err(|e| e.to_string())?;
    
        Braid::parse_braid(&text)
    }


    fn parse_braid(input: &str) -> Result<Braid, String> {
        let mut lines = input
            .lines()
            .map(|l| l.split('#').next().unwrap().trim()) // remove comments
            .filter(|l| !l.is_empty());
    
        // Parse number of strands
        let strands: u32 = lines
            .next()
            .ok_or("Missing strand count")?
            .parse()
            .map_err(|_| "Invalid strand count")?;
        if strands < 2 {
            return Err("A braid must have at least 2 strands".into());
        }
    
        // Parse remaining crossings (all remaining lines, split by whitespace)
        let mut crossings: Vec<i32> = Vec::new();
        for line in lines {
            for word in line.split_whitespace() {
                let g: i32 = word
                    .parse()
                    .map_err(|_| format!("Invalid crossing value '{}'", word))?;
                crossings.push(g);
            }
        }
    
        // Validate generators
        let max_gen = strands as i32 - 1;
        for &g in &crossings {
            if g == 0 || g.abs() > max_gen {
                return Err(format!(
                    "Invalid generator {}: must be in range ±[1, {}]",
                    g, max_gen
                ));
            }
        }
    
        Ok(Braid { strands, crossings })
    }
    
    
}