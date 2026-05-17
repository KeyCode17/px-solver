use anyhow::Result;

pub fn run() -> Result<()> {
    println!(
        "px-cli serve is a thin pointer; run the px-server binary directly: \
         `cargo run -p px-server` (or `px-server` if installed)."
    );
    Ok(())
}
