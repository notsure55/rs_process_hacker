use crate::process;
use std::collections::BTreeMap;

#[derive(Default, Clone)]
enum Type {
    #[default]
    Integer,
    Float,
    Address,
}

#[derive(Default)]
pub struct MyApp {
    name: Option<String>,
    pid: u32,
    address: String,    
    guess: String,
    value: String,
    addresses: BTreeMap<String, Type>,
    vec: Vec<usize>,
}

// TODO: implement grid, to store addresses for further inspection,
// display all addresses with the value of current, and initial value when searching.

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");            
            match self.name {
                Some(ref name) =>
                {                                                            
                    ui.horizontal(|ui| {                        
                        let name_label = ui.label("Your value: ");
                        ui.text_edit_singleline(&mut self.value)
                            .labelled_by(name_label.id);
                    });
                    
                    if ui.button("Search").clicked() {
                        let process = process::Process::new(name).unwrap();
                        self.vec = process.find_value(100).unwrap();
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for address in self.vec.clone() {
                                ui.label(format!("{}", address));
                            }
                        });
                    }                                        
                    if ui.button("Enter").clicked() {
                        let process = process::Process::new(name).unwrap();
                        /*self.value = process.read_mem(
                            usize::from_str_radix(&self.address
                                                  .strip_prefix("0x")
                                                  .unwrap(), 16).unwrap()).unwrap();*/
                    };
                    ui.label(format!("Process_name: {}, pid: {}, address: {}, value: {}",
                                     name, self.pid, self.address, self.value));
                },
                None =>
                {
                    let map = process::select_process().unwrap();                    
                    let process_name_label = ui.label("Enter a process_name");
                    ui.text_edit_singleline(&mut self.guess).labelled_by(process_name_label.id);
                    
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (pid, process_name) in map {
                            if process_name.contains(&self.guess) {
                                if ui.button(format!("Process_name: {}, pid: {}", process_name, pid)).clicked() {
                                    self.name = Some(process_name);
                                    self.pid = pid;
                                    break
                                }
                            }                    
                        }
                    });
                },
            }                                                            
        });
    }
}
