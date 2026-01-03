use std::path::Path;

use egui::{Painter, Response};

#[derive(Default)]
pub struct Braid{
    pub strands : u32,
    pub crossings : Vec<i32>,
}

impl Braid{
    pub fn new() -> Braid {
        Braid::init(10)
    }
    pub fn init(strands : u32) -> Braid{
        Braid{
            strands : strands,
            crossings : vec![],
        }
    }

    pub fn draw(&self, drawing_area : Response, painter : Painter ){
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

    pub fn load_braid_from_file() -> Result<Braid, String> {
        let file = rfd::FileDialog::new()
            .add_filter("Braid file", &["braid"])
            .pick_file()
            .ok_or("No file selected")?;
    
        let text = std::fs::read_to_string(file)
            .map_err(|e| e.to_string())?;
    
        Braid::parse_braid(&text)
    }

    pub fn save_braid_to_file(braid: &Braid, path : &Path) -> Result<(), String> {
        let mut content = format!("{}\n", braid.strands);
        for g in &braid.crossings {
            content.push_str(&format!("{} ", g));
        }
        content.push('\n');
    
        std::fs::write(path, content).map_err(|e| e.to_string())
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