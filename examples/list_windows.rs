use crosswin::prelude::*;

#[tokio::main]
async fn main() -> crosswin::Result<()> {
    let windows = list_windows().await?;
    println!("Found {} windows", windows.len());
    for w in windows {
        println!("{:?}", w);
    }
    Ok(())
}
