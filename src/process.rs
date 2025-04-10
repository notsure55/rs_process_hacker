use std::collections::BTreeMap;
use std::os::unix::prelude::FileExt;
use std::io;
use std::fs;
use bytemuck::Pod;

#[derive(Debug)]
pub struct Process {
    pid: u32,
    name: String,
    maps: BTreeMap<usize, usize>,
    handle: fs::File,
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
            let line: Vec<&str> = line.split(&['-', ' ']).collect();
            if line.len() > 3 {
                if line[2].contains("r") {
                    
                    map.insert(usize::from_str_radix(line[0], 16).unwrap(), usize::from_str_radix(line[1], 16).unwrap());
                }            
            }            
        }
        Ok(map)
    }
    // look how concise this is lmao.
    fn find_handle(pid: u32) -> Result<fs::File, io::Error> {
        Ok(fs::File::open(format!("/proc/{}/mem", pid))?)
    }
    // we add pod to traits or else we cant use try_from_bytes
    pub fn find_value<T: Copy + PartialEq + Pod>(&self, value: T) -> Result<Vec<usize>, io::Error> {
        let mut address_vec = Vec::new();
        for (start_address, end_address) in self.maps.clone() {            
            // make sure address doesnt access invalid mem
            let size = end_address - start_address;
            for i in 0..=(size - std::mem::size_of::<T>()) {
                let mut buf = vec![0u8; size];
                self.handle.read_at(&mut buf, start_address as u64)?;
                let read_value: T = *bytemuck::try_from_bytes(&buf[i..=(std::mem::size_of::<T>() + i)]).unwrap(); 
                if read_value == value {
                    address_vec.push(i + start_address);
                }                
            }            
        }
        if address_vec.is_empty() {
            Err(io::Error::new(io::ErrorKind::NotFound, "No Values Found"))
        } else {
            return Ok(address_vec)
        }                
    }
    pub fn read_mem<T: Copy>(&self, address: usize) -> Result<T, io::Error> {
        let mut buf = vec![0u8; std::mem::size_of::<T>()];                
        self.handle.read_at(&mut buf, address as u64)?;

        let ptr = buf.as_ptr() as *const T;
        unsafe {
            Ok(ptr.read_unaligned())
        }        
    }
    pub fn new(name: &str) -> Result<Self, io::Error> {
        let pid = Self::find_pid(name)?;
        let maps = Self::find_maps(pid)?;
        let handle = Self::find_handle(pid)?;
        Ok(Process {
            pid,
            name: name.to_string(),
            maps,
            handle
        })
    }
}
