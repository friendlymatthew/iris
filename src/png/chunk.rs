use crate::png::{
    grammar::ImageHeader,
    scanline_writer::ScanlineWriter,
};
use anyhow::Result;
use flate2::{
    write::ZlibEncoder,
    Compression,
};
use std::io::Write;

pub trait PngChunk {
    const NAME: [u8; 4];

    fn name(&self) -> &[u8; 4] {
        &Self::NAME
    }

    fn data(&self) -> Result<Vec<u8>> {
        Ok(vec![])
    }

    fn write<W: Write>(&self, mut w: W) -> Result<()> {
        let data = self.data()?;

        w.write_all(&(data.len() as u32).to_be_bytes())?;
        w.write_all(self.name())?;

        let mut hash_data = Vec::new();
        hash_data.extend_from_slice(self.name());
        hash_data.extend_from_slice(&data);

        let crc = crc32fast::hash(&hash_data).to_be_bytes();

        w.write_all(&data)?;
        w.write_all(&crc)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct IHDRChunk<'a> {
    pub image_header: &'a ImageHeader,
}

impl PngChunk for IHDRChunk<'_> {
    const NAME: [u8; 4] = *b"IHDR";

    fn data(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        let ImageHeader {
            width,
            height,
            bit_depth,
            color_type,
            compression_method,
            filter_method,
            interlace_method,
        } = self.image_header;

        buffer.extend_from_slice(&width.to_be_bytes());
        buffer.extend_from_slice(&height.to_be_bytes());
        buffer.extend_from_slice(&bit_depth.to_be_bytes());
        buffer.extend_from_slice(&(*color_type as u8).to_be_bytes());
        buffer.extend_from_slice(&compression_method.to_be_bytes());
        buffer.extend_from_slice(&filter_method.to_be_bytes());
        buffer.extend_from_slice(&(*interlace_method as u8).to_be_bytes());

        Ok(buffer)
    }
}

// #[derive(Debug)]
// pub struct PLTEChunk; // todo!, how does the palette chunk serialize?

// impl PngChunk for PLTEChunk {
//     const NAME: [u8; 4] = *b"PLTE";
// }

#[derive(Debug)]
pub struct IDATChunk<'a> {
    pub image_header: &'a ImageHeader,
    pub data: &'a [u8],
}

impl PngChunk for IDATChunk<'_> {
    const NAME: [u8; 4] = *b"IDAT";

    fn data(&self) -> Result<Vec<u8>> {
        let scanned_pixels = Vec::new();
        let mut scanline_writer = ScanlineWriter::new(scanned_pixels, self.image_header);
        scanline_writer.write(self.data)?;

        let compressed_data = Vec::new();
        let mut encoder = ZlibEncoder::new(compressed_data, Compression::fast());
        encoder.write_all(&scanline_writer.finish())?;

        Ok(encoder.finish()?)
    }
}

#[derive(Debug)]
pub struct IENDChunk;

impl PngChunk for IENDChunk {
    const NAME: [u8; 4] = *b"IEND";
}
