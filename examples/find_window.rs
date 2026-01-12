use crosswin::prelude::*;

#[tokio::main]
async fn main() -> crosswin::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let title = args.get(1).map(|s| s.as_str()).unwrap_or("");

    let matches = find_windows_by_title(title).await?;
    println!("Found {} windows matching '{}':", matches.len(), title);
    for w in matches {
        println!("{:?}", w);
    }
    Ok(())
}
