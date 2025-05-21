use crate::forth::{run_program, EditorStyle, Interpreter};

use egui;
use egui::epaint::{Color32, text::{LayoutJob, TextFormat}, FontFamily, FontId};
use eframe;

pub struct Editor{
    program: String,
    interpreter: Interpreter,
    result: String
}

impl Default for Editor{
    fn default() -> Self {
        Self{
            program: String::default(),
            interpreter: Interpreter::new(),
            result: String::default()
        }
    }
}

fn separate(text: &str) -> Vec<&str>{
    let mut result = Vec::new();
    let mut last = 0;
    for (index, matched) in text.match_indices(|c: char| !(c.is_alphanumeric() || c == '\'')) {
        if last != index {
            result.push(&text[last..index]);
        }
        result.push(matched);
        last = index + matched.len();
    }
    if last < text.len() {
        result.push(&text[last..]);
    }
    result
}

fn highlighting(contents: &str, interpreter: &Interpreter) -> LayoutJob{
    let mut job = LayoutJob::default();
    
    for c in separate(contents){
        
        
        let mut is_whitespace = false;
        
        for ch in c.chars(){
            if ch.is_whitespace(){
                job.append(ch.to_string().as_str(), 0.0, TextFormat::default());
                is_whitespace = true;
            }
        }
        
        if !is_whitespace{
            let color = match interpreter.get_style(&c.to_string()){
                EditorStyle::PRIM => Color32::ORANGE,
                EditorStyle::LITERAL => Color32::GREEN,
                EditorStyle::DEFINED => Color32::CYAN,
                EditorStyle::SYS => Color32::YELLOW,
                EditorStyle::ERROR => Color32::RED
            };
            
            job.append(c, 0.0,
                TextFormat{
                    font_id: FontId::new(14.0, FontFamily::Proportional),
                    color: color,
                    ..Default::default()
                }    
            );
        }
        
        
    }
    
    job
}



impl eframe::App for Editor {
    
    
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());
        
        egui::CentralPanel::default().show(ctx, |ui| {
            //ui.style_mut().visuals = egui::Visuals::dark();
            
            ui.heading("bbforth");
            
            let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                let mut layout_job: egui::text::LayoutJob = highlighting(string, &self.interpreter);
                layout_job.wrap.max_width = wrap_width;
                ui.fonts(|f| f.layout_job(layout_job))
            };
            
            ui.add(egui::TextEdit::multiline(&mut self.program)
                    .layouter(&mut layouter));
            
            if ui.button("Run").clicked(){
                self.result.clear();
                run_program(self.program.as_str(), &mut self.interpreter);
                
                self.result = format!("{:?}", self.interpreter.sys_buffer);
            }
            
            ui.label(format!("Result {}", self.result));
        });
    }
}