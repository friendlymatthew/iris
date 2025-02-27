use anyhow::{anyhow, Result};
use iris::png::PngDecoder;

#[cfg(feature = "time")]
use her::util::*;

#[cfg(feature = "time")]
use std::time::Instant;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let image_path = args
        .next()
        .ok_or_else(|| anyhow!("Failed to read image path"))?;

    let content = std::fs::read(image_path)?;

    let mut decoder = PngDecoder::new(&content);

    #[cfg(feature = "time")]
    let a = Instant::now();
    let _ = decoder.decode()?;
    #[cfg(feature = "time")]
    log_event("", Event::TotalElapsed, Some(a.elapsed()));

    Ok(())
}
