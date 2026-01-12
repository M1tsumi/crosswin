use crosswin::prelude::*;

fn main() -> crosswin::Result<()> {
    let info = get_system_info()?;
    println!("System info: {:?}", info);

    match get_system_uptime() {
        Ok(u) => println!("Uptime: {:?}", u),
        Err(e) => println!("Uptime not supported: {}", e),
    }

    match get_boot_time() {
        Ok(t) => println!("Boot time (approx): {:?}", t),
        Err(e) => println!("Boot time not supported: {}", e),
    }

    Ok(())
}
