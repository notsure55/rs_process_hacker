use crate::process;
use std::collections::BTreeMap;

#[derive(Debug, Default, Clone)]
pub enum Type {
    #[default]
    Integer,
    Float,
    Address,
}

#[derive(Default)]
pub struct MyApp {
    name: Option<String>,        
    addresses: BTreeMap<usize, Type>,    
    process: process::Process,    
    guess: String,
    value: String,
    type_of_var: Type,
    show_types: bool,
}

// TODO: implement grid, to store addresses for further inspection,
// display all addresses with the value of current, and initial value when searching.

impl MyApp {
    fn scan_value(&mut self) {
        if !self.addresses.is_empty() {
            match &self.type_of_var {
                Type::Integer => {                                
                    let value: i32 = self.value.trim().parse().expect("Failed to parse value string");
                    self.process.find_value_repeat(value, &mut self.addresses).expect("Failed to find value");
                },
                Type::Float => {
                    let value: f32 = self.value.trim().parse().expect("Failed to parse value string");
                    self.process.find_value_repeat(value, &mut self.addresses).expect("Failed to find value");                    
                },
                _ => eprintln!("Black"),
            }            
        } else {
            match &self.type_of_var {
                Type::Integer => {                                
                    let value: i32 = self.value.trim().parse().expect("Failed to parse value string");
                    self.addresses = self.process.find_value(value).expect("Failed to find value");
                },
                Type::Float => {
                    let value: f32 = self.value.trim().parse().expect("Failed to parse value string");
                    self.addresses = self.process.find_value(value).expect("Failed to find value");
                }
                _ => eprintln!("WRONG TYPE"),
            }
        }
    }
    fn build_search_option(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) -> Result<(), Box<dyn std::error::Error>> {
        ui.horizontal(|ui| {                        
            let name_label = ui.label("Search value: ");
            ui.text_edit_singleline(&mut self.value)
                .labelled_by(name_label.id);
            ui.label(format!("{:?}", self.type_of_var));
            if ui.button(format!("Choose type")).clicked() {
                self.show_types = !self.show_types;
            }
            if self.show_types {
                if ui.button(format!("I32")).clicked() {
                    self.type_of_var = Type::Integer;
                    self.show_types = !self.show_types;
                }
                if ui.button(format!("F32")).clicked() {
                    self.type_of_var = Type::Float;
                    self.show_types = !self.show_types;
                }
            }
        });
        if ui.button("New Scan").clicked() {
            self.addresses = BTreeMap::new();
        }
        if ui.button("Scan").clicked() {
            self.scan_value();
        }
        
        Ok(())
    }
    fn show_address_grid(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) -> Result<(), Box<dyn std::error::Error>> {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("Addresses").show(ui, |ui| {                            
                for (address, value_type) in &mut self.addresses {
                    match value_type {
                        Type::Integer => {
                            let value: i32 = self.process.read_mem(*address).unwrap();
                            if ui.button(format!("{:?}", value_type)).clicked() {
                                *value_type =  Type::Float;
                            }
                            ui.label(format!("Address: 0x{:x} | Value:{}", address,value));
                            if ui.button("Save").clicked() {
                                
                            }
                            ui.end_row();                                            
                        },
                        Type::Float => {
                            let value: f32 = self.process.read_mem(*address).unwrap();
                            if ui.button(format!("{:?}", value_type)).clicked() {
                                *value_type =  Type::Integer;
                            }
                            ui.label(format!("Address: 0x{:x} | Value:{}", address,value));
                            ui.end_row();                                            
                        }
                        _ => eprintln!("WRONG TYPE"),
                    }                                    
                }
            });                            
        });
        
        Ok(())
    }
    fn find_process(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) -> Result<(), Box<dyn std::error::Error>> {
        let map = process::select_process().unwrap();                    
        let process_name_label = ui.label("Enter a process_name");
        ui.text_edit_singleline(&mut self.guess).labelled_by(process_name_label.id);
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (pid, process_name) in map {
                if process_name.contains(&self.guess) {
                    if ui.button(format!("Process_name: {}, pid: {}", process_name, pid)).clicked() {
                        self.name = Some(process_name.clone());                                    
                        self.process = process::Process::new(&process_name).unwrap();
                        break
                    }
                }                    
            }
        });
        
        Ok(())
    }
}

impl eframe::App for MyApp {    
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");            
            match self.name {
                Some(ref _name) =>
                {
                    let _ = self.build_search_option(ctx, ui);
                    
                    if !self.addresses.is_empty() {
                        let _ = self.show_address_grid(ctx, ui);
                    }
                    
                },
                None =>
                {
                    let _ = self.find_process(ctx, ui);
                },
            }                                                            
        });
    }
}


