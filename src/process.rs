use std::collections::BTreeMap;
use std::os::unix::prelude::FileExt;
use std::io;
use std::fs;
use bytemuck::Pod;
use std::fs::OpenOptions;
use mem_cmp::MemEq;

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
    fn parse_pattern_string(s: &str) -> Vec<u8> {
        s.split_whitespace()
            .map(|byte_str| u8::from_str_radix(byte_str, 16).expect("Invalid hex byte"))
            .collect()
    }    
    // pattern scanning also know as sig scanning
    pub fn find_pattern(&self, pattern: &str) -> Result<usize, io::Error> {
        let pattern = Self::parse_pattern_string(pattern);
        let signature = &pattern[..];
        for (start_address, end_address) in self.maps.clone() {            
            // we get size of the address space that we want to read from per /proc/pid/maps
            let size: usize = end_address - start_address;
            let mut buf = vec![0u8; size];            
            self.handle.as_ref()
                .unwrap()
                .read_exact_at(&mut buf, start_address as u64)
                .expect(&format!("Failed to read from 0x{start_address:x}-0x{end_address:x}"));
            
            for i in 0..=(size - signature.len()) {                                                
                if signature.as_ref().mem_eq(&buf[i..(i + signature.len())]) {
                    return Ok(i + start_address)
                }                                
            }            
        }
        
        Err(io::Error::new(io::ErrorKind::NotFound, "Sig not found"))
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
    
    pub fn find_value<T: Copy + PartialEq + Pod + MemEq + num_traits::ops::bytes::ToBytes>(&self, value: T) -> Vec<usize> {
        // getting value bytes and size
        let value_size = std::mem::size_of::<T>();
        let value = value.to_ne_bytes();
        
        let mut addresses = vec![];
        for (start_address, end_address) in self.maps.clone() {            
            // we get size of the address space that we want to read from per /proc/pid/maps
            let size: usize = end_address - start_address;
            let mut buf = vec![0u8; size];            
            self.handle.as_ref()
                .unwrap()
                .read_exact_at(&mut buf, start_address as u64)
                .expect(&format!("Failed to read from 0x{start_address:x}-0x{end_address:x}"));
            
            for i in 0..=(size - std::mem::size_of::<T>()) {                                                
                if value.as_ref().mem_eq(&buf[i..(i + value_size)]) {
                    addresses.push(i + start_address);
                }                                
            }            
        }                        
        addresses           
    }
    pub fn find_value_repeat<T: Pod + PartialEq + std::fmt::Display>(&self, new_value: T, addresses: &mut Vec<usize>) -> Result<(), io::Error> {
        // returns only addresses where it equals new value
        addresses.retain(|&address| {
            let value: T = self.read_mem(&address).expect("Failed to read_address");
            new_value == value            
        });        
        Ok(())
    }
    pub fn read_mem<T: Copy + Pod>(&self, address: &usize) -> Result<T, Box<dyn std::error::Error>> {
        let mut buf = vec![0u8; std::mem::size_of::<T>()];

        self.handle.as_ref()
            .unwrap()
            .read_exact_at(&mut buf, *address as u64)?;

        let read_value: T = * bytemuck::try_from_bytes(&buf[0..std::mem::size_of::<T>()])
            .map_err(|err| eprintln!("Failed to read value from address: 0x{:x} ERROR: {}", address, err))
            .unwrap();                
        
        Ok(read_value)
    }
    pub fn read_mem_and_bytes<T: Copy + Pod>(&self, address: usize) -> (T, Vec<u8>){
        let mut buf = vec![0u8; std::mem::size_of::<T>()];
                
        self.handle.as_ref()
            .unwrap()
            .read_exact_at(&mut buf, address as u64)
            .expect("Failed to read bytes");

        let read_value: T = * bytemuck::try_from_bytes(&buf[0..std::mem::size_of::<T>()])
            .map_err(|err| eprintln!("Failed to read value from address: 0x{:x} ERROR: {}", address, err))
            .unwrap();

        return (read_value, buf)        
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
