use anyhow::{anyhow, Result};
use iris::{png::PngDecoder, renderer};
use pollster::block_on;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let image_path = args
        .next()
        .ok_or_else(|| anyhow!("Failed to read image path"))?;

    let content = std::fs::read(image_path)?;
    let mut decoder = PngDecoder::new(&content);
    let png = decoder.decode()?;

    let _ = block_on(renderer::run(png));

    Ok(())
}
