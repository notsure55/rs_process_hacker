use std::collections::BTreeMap;
use std::io;
use std::fs;

#[derive(Debug)]
pub struct Process {
    pid: u32,
    name: String,
    maps: BTreeMap<usize, usize>,
    handle: fs::File,
}

// BTreeMap<String, u32>

pub fn select_process() -> io::Result<()> {
    for entry in fs::read_dir("/proc/")? {
        let dir = entry?;
        let stat_path = format!("{}/stat", dir.path().display());
        let content: String = match fs::read_to_string(stat_path) {
            Ok(content) => content,
            Err(_e) => continue,
        };
        let stored: Vec<&str> = content.split(&['(', ')']).collect();
        println!("{}, {}", stored[1], stored[0]);
    }
    Ok(())
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
            if line[2].contains("r") {
                if line.len() > 2 {
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
