use std::collections::BTreeMap;
use std::os::unix::prelude::FileExt;
use std::io;
use std::fs;
use bytemuck::Pod;
use crate::window::Type;
use std::fs::OpenOptions;

#[derive(Debug, Default)]
pub struct Process {
    pid: u32,
    pub name: Option<String>,
    maps: BTreeMap<usize, usize>,
    handle: Option<fs::File>,
}

pub fn select_process() -> io::Result<BTreeMap<u32, String>> {    
    let mut map: BTreeMap<u32, String> = BTreeMap::new();
    for entry in fs::read_dir("/proc/")? {
        let dir = entry?;
        let stat_path = format!("{}/stat", dir.path().display());
        let content: String = match fs::read_to_string(stat_path) {
            Ok(content) => content,
            Err(_e) => continue,
        };
        let stored: Vec<&str> = content.split(&['(', ')']).collect();        
        map.insert(stored[0].trim().parse::<u32>().unwrap(), stored[1].to_owned());
    }
    
    Ok(map)
}

impl Process {
    fn find_pid(process_name: &str) -> Result<u32, io::Error> {
        for entry in fs::read_dir("/proc/")? {
            let dir = entry?;
            let stat_path = format!("{}/stat", dir.path().display());
            let content: String = match fs::read_to_string(stat_path) {
                Ok(content) => content,
                Err(_e) => continue,
            };
            let stored: Vec<&str> = content.split(&['(', ')']).collect();            
            if stored[1] == process_name {                
                return stored[0].trim().parse::<u32>().map_err(|e|
                       io::Error::new(io::ErrorKind::InvalidData, e));
            }            
        }        
        Err(io::Error::new(io::ErrorKind::NotFound, "Process not found"))
    }
    fn find_maps(pid: u32) -> Result<BTreeMap<usize, usize>, io::Error> {
        let content = match fs::read_to_string(format!("/proc/{}/maps", pid)) {
            Ok(content) => content,
            Err(e) => return Err(e) 
        };
        let mut map: BTreeMap<usize, usize> = BTreeMap::new();
        let stored: Vec<&str> = content.split('\n').collect();        
        for line in stored {
            if line.contains("[vvar]") || line.contains("[vvar_vclock]") {
                continue
            }
            let line: Vec<&str> = line.split(&['-', ' ']).collect();
            if line.len() > 3 {
                if line[2].contains("r") {
                    
                    let start_address = usize::from_str_radix(line[0], 16).unwrap();
                    let end_address = usize::from_str_radix(line[1], 16).unwrap();
                    
                    if start_address > end_address {
                        continue;
                    } else {
                        map.insert(start_address, end_address);
                    }                    
                }            
            }            
        }
        Ok(map)
    }
    // look how concise this is lmao.
    fn find_handle(pid: u32) -> Result<fs::File, io::Error> {
        // opening mem file with correct permissions so we can write to it or else invald handle :P
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(format!("/proc/{}/mem", pid))?;
        
        Ok(file)        
    }
    // we add pod to traits or else we cant use try_from_bytes
    pub fn find_value<T: Copy + PartialEq + Pod>(&self, value: T) -> Result<BTreeMap<usize, Type>, io::Error> {        
        let value_type = match std::any::type_name::<T>() {
            "i32" => Type::Integer,
            "f32" => Type::Float,
            _ => Type::default(),            
        };
        let mut address_map: BTreeMap<usize, Type> = BTreeMap::new();
        for (start_address, end_address) in self.maps.clone() {            
            // we get size of the address space that we want to read from per /proc/pid/maps
            let size: usize = end_address - start_address;
            let mut buf = vec![0u8; size];            
            self.handle.as_ref()
                .unwrap()
                .read_exact_at(&mut buf, start_address as u64)
                .expect(&format!("Failed to read from 0x{start_address:x}-0x{end_address:x}"));            
            for i in 0..=(size - std::mem::size_of::<T>()) {
                // if the value aligns to our type we derefrence it and store in read_value then compare to input value
                let read_value: T = match bytemuck::try_from_bytes(&buf[i..(std::mem::size_of::<T>() + i)]) {
                    Ok(value) => *value,
                    Err(_) => continue,
                };
                
                if read_value == value {                    
                    address_map.insert(i + start_address, value_type.clone());
                }                
            }            
        }                        
        return Ok(address_map)            
    }
    pub fn find_value_repeat<T: Pod + PartialEq + std::fmt::Display>(&self, new_value: T, addresses: &mut BTreeMap<usize, Type>) -> Result<(), io::Error> {        
        for (address, _) in addresses.clone() {
            let value: T = self.read_mem(address).expect("Failed to read_address");            
            if new_value != value {                
                addresses.remove(&address);
            }
        }
        Ok(())
    }
    pub fn read_mem<T: Copy + Pod>(&self, address: usize) -> Result<T, Box<dyn std::error::Error>> {
        let mut buf = vec![0u8; std::mem::size_of::<T>()];                
        self.handle.as_ref()
            .unwrap()
            .read_exact_at(&mut buf, address as u64)?;

        let read_value: T = * bytemuck::try_from_bytes(&buf[0..std::mem::size_of::<T>()])
            .map_err(|err| eprintln!("Failed to read value from address: 0x{:x} ERROR: {}", address, err))
            .unwrap();
        
        Ok(read_value)
    }    
    pub fn write_mem<T: Copy + Pod >(&self, address: usize, value: T) -> Result<(), io::Error> {        
        let buf = bytemuck::bytes_of(&value);        
        let _ = self.handle.as_ref().unwrap().write_at(&buf, address as u64);
        Ok(())
    }
    pub fn new(name: &str) -> Result<Self, io::Error> {
        let pid = Self::find_pid(name)?;
        let maps = Self::find_maps(pid)?;
        let handle = Some(Self::find_handle(pid)?);
        Ok(Process {
            pid,
            name: Some(name.to_string()),
            maps,
            handle
        })
    }
}
