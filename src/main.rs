use std::io::Result;

mod process;
mod window;

fn main() -> Result<()> {
    let process = process::Process::new("linux_64_client")?;
    println!("{:?}", process);
    Ok(())
}
