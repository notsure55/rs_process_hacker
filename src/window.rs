use crate::process;
use std::collections::BTreeMap;

#[derive(Debug, Default, Clone, PartialEq)]
pub enum Type {
    #[default]
    Integer,
    Float,
}

#[derive(Debug, Default, PartialEq)]
pub enum SearchOptions {
    #[default]
    Nothing,
    Scan,
    NewScan,
    Restart,
    MemoryViewer,
    AddNewAddress,
}

#[derive(Debug)]
struct StoredValue {
    name: String,
    address: String,
    value: String,
    address_usize: usize,
    type_of_var: Type,    
}

#[derive(Default, Debug)]
struct DisplayAddress {
    display: String,
    real: usize,
}

#[derive(Default, Debug)]
pub struct MyApp {
    // WHERE ADDRESSES ARE STROED
    addresses: BTreeMap<usize, Type>,
    saved_addresses: Vec<StoredValue>,
    
    // PROCESS INPUT AND STUFFS
    process: process::Process,    
    guess: String,
    
    // SCANNING
    value: String,
    type_of_var: Type,
    type_option: SearchOptions,

    // ADDING NEW ADDRESS    
    add_new_address: bool,
    new_address: String,

    // MEMORY VIEWER STUFFS
    top_address: DisplayAddress,
    memory_viewer_toggle: bool,
}

impl StoredValue {
    pub fn new(address: String) -> Self {
        let address_usize: usize = usize::from_str_radix(address.clone()
            .trim(), 16)
            .expect(&format!("Failed to parse address: {}", address.clone()));
        
        Self {
            name: String::new(),
            address: address,
            value: String::new(),
            address_usize: address_usize,
            type_of_var: Type::default(),
        }
    }
}

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
                },                
            }
        }
    }
    fn build_search_option(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) -> Result<(), Box<dyn std::error::Error>> {
        egui::Grid::new("Top Options").max_col_width(150.0).spacing(egui::vec2(1.0, 2.0)).show(ui, |ui| {                      
            let name_label = ui.label("Search value: ");
            let response = ui.text_edit_singleline(&mut self.value)
                .labelled_by(name_label.id);

            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !self.value.is_empty() {            
                self.scan_value();                   
            }
            
            egui::ComboBox::from_id_salt("Search_type_combo_box")
                .selected_text(format!("{:?}", self.type_of_var))
                .show_ui(ui, |ui|{
                    
                    ui.selectable_value(&mut self.type_of_var, Type::Integer, "I32");
                    ui.selectable_value(&mut self.type_of_var, Type::Float, "F32");                
                });

            egui::ComboBox::from_id_salt("Search_options_combo_box")
                .selected_text("Options")
                .show_ui(ui, |ui|{

                    ui.selectable_value(&mut self.type_option, SearchOptions::Nothing, "Nothing");
                    ui.selectable_value(&mut self.type_option, SearchOptions::Scan, "Scan");
                    ui.selectable_value(&mut self.type_option, SearchOptions::NewScan, "NewScan");
                    ui.selectable_value(&mut self.type_option, SearchOptions::Restart, "Restart");
                    ui.selectable_value(&mut self.type_option, SearchOptions::MemoryViewer, "MemoryViewer");
                    ui.selectable_value(&mut self.type_option, SearchOptions::AddNewAddress, "AddNewAddress");                
                });
            
            match &self.type_option {
                SearchOptions::Nothing =>
                {
                    // do nothing
                },
                SearchOptions::Scan =>
                {
                    if !self.value.is_empty() {
                        self.scan_value();
                    }
                    self.type_option = SearchOptions::Nothing;
                }, 
                SearchOptions::NewScan =>
                {
                    self.addresses = BTreeMap::new();
                    self.type_option = SearchOptions::Nothing;
                },
                SearchOptions::Restart =>
                {
                    self.process.name = None;
                    self.type_option = SearchOptions::Nothing;
                },
                SearchOptions::MemoryViewer =>
                {
                    // TODO: implement memviewer
                },
                SearchOptions::AddNewAddress =>
                {
                    let response = ui.add(egui::TextEdit::singleline(&mut self.new_address).desired_width(75.0));
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {                                     
                        self.saved_addresses.push(StoredValue::new(self.new_address.clone()));
                    }
                },
            };
                                                                        
            ui.end_row();
            
        });
                
        Ok(())
    }
    fn show_address_grid(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) -> Result<(), Box<dyn std::error::Error>> {
        egui::ScrollArea::vertical().id_salt("scroll_area_2").auto_shrink([false; 2]).show(ui, |ui| {
            egui::Grid::new("New Addresses").spacing(egui::vec2(5.0, 6.0)).show(ui, |ui| {                        
                let mut count = 0;
                for (address, value_type) in self.addresses.clone().iter() {                    
                    match value_type {
                        Type::Integer => {
                            let value: i32 = self.process.read_mem(*address).unwrap();
                            ui.label(format!("Address: 0x{:x} | {:?}:Value:{}", address, value_type, value));
                            
                            if ui.button("Save").clicked() {                                 
                                // store addresses, and remove from stack
                                self.saved_addresses.push(StoredValue::new(format!("{:x}", address.clone())));
                                self.addresses.remove(&address);
                            }
                        },
                        Type::Float => {
                            let value: f32 = self.process.read_mem(*address).unwrap();
                            ui.label(format!("Address: 0x{:x} | {:?}:Value:{}", address, value_type, value));
                            
                            if ui.button("Save").clicked() {
                                // store addresses, and remove from stack
                                self.saved_addresses.push(StoredValue::new(format!("{:x}", address.clone())));
                                self.addresses.remove(&address);                                
                            }
                        },                        
                    }                    
                    ui.end_row();                                            
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
                        self.process = process::Process::new(&process_name).unwrap();                        
                        break
                    }
                }                    
            }
        });
        
        Ok(())
    }
    fn show_stored_value(process : &process::Process, _ctx: &egui::Context, ui: &mut egui::Ui, stored: &mut StoredValue) {
        // NAME
        ui.add(egui::TextEdit::singleline(&mut stored.name)
               .frame(true).desired_width(ui.available_width()));
        // ADDRESS                    
        let response = ui.add(egui::TextEdit::singleline(&mut stored.address)
                              .frame(true).desired_width(ui.available_width()));
        // if let statements go hard
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Ok(addr) = usize::from_str_radix(&stored.address.clone().trim(), 16) {
                stored.address_usize = addr;                                                                     
            }                        
        }
        // VALUE
        let response = ui.add(egui::TextEdit::singleline(&mut stored.value)
                              .frame(true).desired_width(ui.available_width()));
        
        match stored.type_of_var {
            Type::Integer =>
            {                                                               
                // editing the value                                        
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Ok(parsed) = stored.value.trim().parse::<i32>() {
                        let value: i32 = parsed;
                        println!("0x{:x}", stored.address_usize);
                        process.write_mem(stored.address_usize, value)
                            .expect("failed to write to memory");
                    }
                }
                // when not editing value, constantly read from mem for real time update
                else if !response.has_focus() {                    
                    let value: i32 = process.read_mem(stored.address_usize)
                        .expect("Failed to unwrap read_mem");
                    stored.value = value.to_string();
                }                                                
            }
            Type::Float =>
            {
                // editing the value                                        
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Ok(parsed) = stored.value.trim().parse::<f32>() {
                        let value: f32 = parsed;
                        println!("0x{:x}", stored.address_usize);
                        process.write_mem(stored.address_usize, value)
                            .expect("failed to write to memory");
                    }
                }
                // when not editing value, constantly read from mem for real time update
                else if !response.has_focus() {                    
                    let value: f32 = process.read_mem(stored.address_usize)
                        .expect("Failed to unwrap read_mem");
                    stored.value = value.to_string();
                }
            },            
        }                                
        egui::ComboBox::from_id_salt(format!("{}", stored.address))
            .selected_text(format!("{:?}", stored.type_of_var))
            .show_ui(ui, |ui| {
             ui.selectable_value(&mut stored.type_of_var, Type::Integer, "I32");
             ui.selectable_value(&mut stored.type_of_var, Type::Float, "F32");                
        });                
    }
    fn show_saved_addresses(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) -> Result<(), Box<dyn std::error::Error>> {
        egui::ScrollArea::vertical().id_salt("scroll_area_1").auto_shrink([false; 2]).show(ui, |ui| {
            egui::Grid::new("Saved Addresses").min_col_width(75.0).spacing(egui::vec2(2.0, 6.0)).show(ui, |ui| {
                let mut saved_indexes = Vec::new();
                for (i, stored) in self.saved_addresses.iter_mut().enumerate() {
                    Self::show_stored_value(&self.process, _ctx, ui, stored);
                    if ui.button("Delete").clicked() {
                        saved_indexes.push(i);
                    }
                    ui.end_row();
                }
                for i in saved_indexes {
                    self.saved_addresses.remove(i);
                }
            });
        });
        
        Ok(())
    }
    fn memory_viewer(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui)
    {
        let response = ui.text_edit_singleline(&mut self.top_address.display);
        
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Ok(parsed) = usize::from_str_radix(self.top_address.display.clone().trim(), 16)
            {
                self.top_address.real = parsed;
                self.memory_viewer_toggle = true;
            }                                        
        }

        egui::ComboBox::from_id_salt(format!("{}", stored.address))
            .selected_text(format!("{:?}", stored.type_of_var))
            .show_ui(ui, |ui| {
             ui.selectable_value(&mut stored.type_of_var, Type::Integer, "I32");
             ui.selectable_value(&mut stored.type_of_var, Type::Float, "F32");                
            });
        
        egui::ScrollArea::vertical().id_salt("Memory_viewer").auto_shrink([false; 2]).show(ui, |ui| {
            
        });
    }
}

// TODO: Add a memory viewer for ses, and the ability to change values and types of values within the memoryspace
// Basically adding reclass.net

impl eframe::App for MyApp {    
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Rem Engine");            
            match self.process.name {
                Some(ref _name) =>
                {
                    if self.type_option == SearchOptions::MemoryViewer {
                        self.memory_viewer(ctx, ui);
                    }
                    else {
                        let _ = self.build_search_option(ctx, ui);

                        egui::Grid::new("Black").show(ui, |ui| {
                            let mut size = ui.spacing().interact_size;
                            size.x = 350.0;
                            size.y = 200.0;
                            ui.allocate_ui_with_layout(size, egui::Layout::left_to_right(egui::Align::Min), |ui| {
                                let _ = self.show_address_grid(ctx, ui);
                                let _ = self.show_saved_addresses(ctx, ui);
                            });                        
                        });
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
