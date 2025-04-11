use crate::process;
use std::collections::BTreeMap;

#[derive(Debug, Default, Clone)]
pub enum Type {
    #[default]
    Integer,
    Float,
    Address,
}

#[derive(Debug)]
struct StoredValue {
    address: String,
    value: i32,
}

#[derive(Default, Debug)]
pub struct MyApp {
    name: Option<String>,        
    addresses: BTreeMap<usize, Type>,
    saved_addresses: Vec<StoredValue>,
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
        egui::ScrollArea::vertical().id_salt("scroll_area_2").auto_shrink([false; 2]).show(ui, |ui| {
            egui::Grid::new("New Addresses").spacing(egui::vec2(10.0, 6.0)).show(ui, |ui| {                        
                let mut count = 0;
                for (address, value_type) in self.addresses.clone().iter() {                    
                    match value_type {
                        Type::Integer => {
                            let value: i32 = self.process.read_mem(*address).unwrap();                            
                            ui.label(format!("Address: 0x{:x} | {:?}:Value:{}", address, value_type, value ));
                            if ui.button("Save").clicked() {
                                let stored_value = StoredValue {
                                    address: address.clone().to_string(),
                                    value: value,                                    
                                };
                                self.saved_addresses.push(stored_value);
                                self.addresses.remove(&address);
                            }
                            ui.end_row();                                            
                        },
                        Type::Float => {
                            let value: f32 = self.process.read_mem(*address).unwrap();
                            ui.label(format!("Address: 0x{:x} | {:?}:Value:{}", address, value_type, value));
                            ui.end_row();                                            
                        }
                        _ => eprintln!("WRONG TYPE"),
                    }
                    if count > 100 { break; }
                    count += 1;
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
    fn show_saved_addresses(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) -> Result<(), Box<dyn std::error::Error>> {
        egui::ScrollArea::vertical().id_salt("scroll_area_1").auto_shrink([false; 2]).show(ui, |ui| {
            egui::Grid::new("Saved Addresses").spacing(egui::vec2(10.0, 6.0)).show(ui, |ui| {                
                for stored in self.saved_addresses.iter_mut() {                                                            
                    /*if ui.button(format!("{:?}", value_type)).clicked() {
                     *value_type =  Type::Float;
                    }*/                    
                    let value_label = ui.label(format!("Address:{} | Value:{}", stored.address, stored.value));
                    let response = ui.text_edit_singleline(&mut stored.address)
                        .labelled_by(value_label.id);
                    if response.lost_focus() {
                        stored.value = self.process.read_mem(usize::from_str_radix(&stored.address.clone(), 16).unwrap()).unwrap();
                    }
                    ui.end_row();                                                            
                }
                
            });                            
        });
        
        Ok(())
    }
}

impl eframe::App for MyApp {    
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Rem Engine");            
            match self.name {
                Some(ref _name) =>
                {
                    let _ = self.build_search_option(ctx, ui);

                    egui::Grid::new("Black").show(ui, |ui| {
                        let mut size = ui.spacing().interact_size;
                        size.x = 200.0;
                        size.y = 200.0;
                        ui.allocate_ui_with_layout(size, egui::Layout::left_to_right(egui::Align::Min), |ui| {
                            if !self.addresses.is_empty() {                            
                                let _ = self.show_address_grid(ctx, ui);                                
                            }                            
                            if !self.saved_addresses.is_empty() {
                                let _ = self.show_saved_addresses(ctx, ui);
                            }
                        });                        
                    });
                },
                None =>
                {
                    let _ = self.find_process(ctx, ui);
                },
            }                                                            
        });
    }
}


