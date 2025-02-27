use anyhow::{anyhow, Result};
use iris::png::{grammar::Png, PngDecoder};
use std::fs;
use std::time::Instant;

fn read_png(image_path: &str) -> Result<Png> {
    let image_data = fs::read(image_path)?;
    let image = PngDecoder::new(&image_data).decode()?;

    Ok(image)
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let (reference_image, test_image) = match (args.next(), args.next()) {
        (Some(reference_image_path), Some(test_image_path)) => (
            read_png(&reference_image_path)?,
            read_png(&test_image_path)?,
        ),
        _ => {
            return Err(anyhow!(
                "Provide paths to a reference image AND a test image."
            ))
        }
    };

    let now = Instant::now();
    let ssim = reference_image.compute_sim(&test_image)?;
    println!("ssim score: {}\telapsed: {:?}", ssim, now.elapsed());

    Ok(())
}
