// A parser to parse the file names in PngSuite.
// http://www.schaik.com/pngsuite/

use std::{os::unix::ffi::OsStrExt, path::PathBuf};

use anyhow::{anyhow, bail, Result};

pub struct PNGSuiteTestCase<'a> {
    pub test_desc: &'a str,
    pub should_fail: bool,
}

pub fn parse_test_file<'a>(file_path: &PathBuf) -> Result<PNGSuiteTestCase<'a>> {
    let test_file = file_path
        .file_stem()
        .ok_or_else(|| anyhow!("Failed to parse file stem from {:?}", file_path))?
        .as_bytes();

    let should_fail = test_file[0] == b'z';

    let test_desc = match test_file {
        b"basn0g01" => "black & white",
        b"basn0g02" => "2 bit (4 level) grayscale",
        b"basn0g04" => "4 bit (16 level) grayscale",
        b"basn0g08" => "8 bit (256 level) grayscale",
        b"basn0g16" => "16 bit (64k level) grayscale",
        b"basn2c08" => "3x8 bits rgb color",
        b"basn2c16" => "3x16 bits rgb color",
        b"basn3p01" => "1 bit (2 color) paletted",
        b"basn3p02" => "2 bit (4 color) paletted",
        b"basn3p04" => "4 bit (16 color) paletted",
        b"basn3p08" => "8 bit (256 color) paletted",
        b"basn4a08" => "8 bit grayscale + 8 bit alpha-channel",
        b"basn4a16" => "16 bit grayscale + 16 bit alpha-channel",
        b"basn6a08" => "3x8 bits rgb color + 8 bit alpha-channel",
        b"basn6a16" => "3x16 bits rgb color + 16 bit alpha-channel",
        b"basi0g01" => "black & white (Adam-7 interlaced)",
        b"basi0g02" => "2 bit (4 level) grayscale (Adam-7 interlaced)",
        b"basi0g04" => "4 bit (16 level) grayscale (Adam-7 interlaced)",
        b"basi0g08" => "8 bit (256 level) grayscale (Adam-7 interlaced)",
        b"basi0g16" => "16 bit (64k level) grayscale (Adam-7 interlaced)",
        b"basi2c08" => "3x8 bits rgb color (Adam-7 interlaced)",
        b"basi2c16" => "3x16 bits rgb color (Adam-7 interlaced)",
        b"basi3p01" => "1 bit (2 color) paletted (Adam-7 interlaced)",
        b"basi3p02" => "2 bit (4 color) paletted (Adam-7 interlaced)",
        b"basi3p04" => "4 bit (16 color) paletted (Adam-7 interlaced)",
        b"basi3p08" => "8 bit (256 color) paletted (Adam-7 interlaced)",
        b"basi4a08" => "8 bit grayscale + 8 bit alpha-channel (Adam-7 interlaced)",
        b"basi4a16" => "16 bit grayscale + 16 bit alpha-channel (Adam-7 interlaced)",
        b"basi6a08" => "3x8 bits rgb color + 8 bit alpha-channel (Adam-7 interlaced)",
        b"basi6a16" => "3x16 bits rgb color + 16 bit alpha-channel (Adam-7 interlaced)",
        b"bgai4a08" => "8 bit grayscale, alpha, no background chunk, interlaced",
        b"bgai4a16" => "16 bit grayscale, alpha, no background chunk, interlaced",
        b"bgan6a08" => "3x8 bits rgb color, alpha, no background chunk",
        b"bgan6a16" => "3x16 bits rgb color, alpha, no background chunk",
        b"bgbn4a08" => "8 bit grayscale, alpha, black background chunk",
        b"bggn4a16" => "16 bit grayscale, alpha, gray background chunk",
        b"bgwn6a08" => "3x8 bits rgb color, alpha, white background chunk",
        b"bgyn6a16" => "3x16 bits rgb color, alpha, yellow background chunk",
        b"ccwn2c08" => "chroma chunk w:0.3127,0.3290 r:0.64,0.33 g:0.30,0.60 b:0.15,0.06",
        b"ccwn3p08" => "chroma chunk w:0.3127,0.3290 r:0.64,0.33 g:0.30,0.60 b:0.15,0.06",
        b"cdfn2c08" => "physical pixel dimensions, 8x32 flat pixels",
        b"cdhn2c08" => "physical pixel dimensions, 32x8 high pixels",
        b"cdsn2c08" => "physical pixel dimensions, 8x8 square pixels",
        b"cdun2c08" => "physical pixel dimensions, 1000 pixels per 1 meter",
        b"ch1n3p04" => "histogram 15 colors",
        b"ch2n3p08" => "histogram 256 colors",
        b"cm0n0g04" => "modification time, 01-jan-2000 12:34:56",
        b"cm7n0g04" => "modification time, 01-jan-1970 00:00:00",
        b"cm9n0g04" => "modification time, 31-dec-1999 23:59:59",
        b"cs3n2c16" => "color, 13 significant bits",
        b"cs3n3p08" => "paletted, 3 significant bits",
        b"cs5n2c08" => "color, 5 significant bits",
        b"cs5n3p08" => "paletted, 5 significant bits",
        b"cs8n2c08" => "color, 8 significant bits (reference)",
        b"cs8n3p08" => "paletted, 8 significant bits (reference)",
        b"ct0n0g04" => "no textual data",
        b"ct1n0g04" => "with textual data",
        b"ctzn0g04" => "with compressed textual data",
        b"cten0g04" => "international UTF-8, english",
        b"ctfn0g04" => "international UTF-8, finnish",
        b"ctgn0g04" => "international UTF-8, greek",
        b"cthn0g04" => "international UTF-8, hindi",
        b"ctjn0g04" => "international UTF-8, japanese",
        b"exif2c08" => "chunk with jpeg exif data",
        b"f00n0g08" => "grayscale, no interlacing, filter-type 0",
        b"f00n2c08" => "color, no interlacing, filter-type 0",
        b"f01n0g08" => "grayscale, no interlacing, filter-type 1",
        b"f01n2c08" => "color, no interlacing, filter-type 1",
        b"f02n0g08" => "grayscale, no interlacing, filter-type 2",
        b"f02n2c08" => "color, no interlacing, filter-type 2",
        b"f03n0g08" => "grayscale, no interlacing, filter-type 3",
        b"f03n2c08" => "color, no interlacing, filter-type 3",
        b"f04n0g08" => "grayscale, no interlacing, filter-type 4",
        b"f04n2c08" => "color, no interlacing, filter-type 4",
        b"f99n0g04" => "bit-depth 4, filter changing per scanline",
        b"g03n0g16" => "grayscale, file-gamma = 0.35",
        b"g03n2c08" => "color, file-gamma = 0.35",
        b"g03n3p04" => "paletted, file-gamma = 0.35",
        b"g04n0g16" => "grayscale, file-gamma = 0.45",
        b"g04n2c08" => "color, file-gamma = 0.45",
        b"g04n3p04" => "paletted, file-gamma = 0.45",
        b"g05n0g16" => "grayscale, file-gamma = 0.55",
        b"g05n2c08" => "color, file-gamma = 0.55",
        b"g05n3p04" => "paletted, file-gamma = 0.55",
        b"g07n0g16" => "grayscale, file-gamma = 0.70",
        b"g07n2c08" => "color, file-gamma = 0.70",
        b"g07n3p04" => "paletted, file-gamma = 0.70",
        b"g10n0g16" => "grayscale, file-gamma = 1.00",
        b"g10n2c08" => "color, file-gamma = 1.00",
        b"g10n3p04" => "paletted, file-gamma = 1.00",
        b"g25n0g16" => "grayscale, file-gamma = 2.50",
        b"g25n2c08" => "color, file-gamma = 2.50",
        b"g25n3p04" => "paletted, file-gamma = 2.50",
        b"oi1n0g16" => "grayscale mother image with 1 idat-chunk",
        b"oi1n2c16" => "color mother image with 1 idat-chunk",
        b"oi2n0g16" => "grayscale image with 2 idat-chunks",
        b"oi2n2c16" => "color image with 2 idat-chunks",
        b"oi4n0g16" => "grayscale image with 4 unequal sized idat-chunks",
        b"oi4n2c16" => "color image with 4 unequal sized idat-chunks",
        b"oi9n0g16" => "grayscale image with all idat-chunks length one",
        b"oi9n2c16" => "color image with all idat-chunks length one",
        b"pp0n2c16" => "six-cube palette-chunk in true-color image",
        b"pp0n6a08" => "six-cube palette-chunk in true-color+alpha image",
        b"ps1n0g08" => "six-cube suggested palette (1 byte) in grayscale image",
        b"ps1n2c16" => "six-cube suggested palette (1 byte) in true-color image",
        b"ps2n0g08" => "six-cube suggested palette (2 bytes) in grayscale image",
        b"ps2n2c16" => "six-cube suggested palette (2 bytes) in true-color image",
        b"s01i3p01" => "1x1 paletted file, interlaced",
        b"s01n3p01" => "1x1 paletted file, no interlacing",
        b"s02i3p01" => "2x2 paletted file, interlaced",
        b"s02n3p01" => "2x2 paletted file, no interlacing",
        b"s03i3p01" => "3x3 paletted file, interlaced",
        b"s03n3p01" => "3x3 paletted file, no interlacing",
        b"s04i3p01" => "4x4 paletted file, interlaced",
        b"s04n3p01" => "4x4 paletted file, no interlacing",
        b"s05i3p02" => "5x5 paletted file, interlaced",
        b"s05n3p02" => "5x5 paletted file, no interlacing",
        b"s06i3p02" => "6x6 paletted file, interlaced",
        b"s06n3p02" => "6x6 paletted file, no interlacing",
        b"s07i3p02" => "7x7 paletted file, interlaced",
        b"s07n3p02" => "7x7 paletted file, no interlacing",
        b"s08i3p02" => "8x8 paletted file, interlaced",
        b"s08n3p02" => "8x8 paletted file, no interlacing",
        b"s09i3p02" => "9x9 paletted file, interlaced",
        b"s09n3p02" => "9x9 paletted file, no interlacing",
        b"s32i3p04" => "32x32 paletted file, interlaced",
        b"s32n3p04" => "32x32 paletted file, no interlacing",
        b"s33i3p04" => "33x33 paletted file, interlaced",
        b"s33n3p04" => "33x33 paletted file, no interlacing",
        b"s34i3p04" => "34x34 paletted file, interlaced",
        b"s34n3p04" => "34x34 paletted file, no interlacing",
        b"s35i3p04" => "35x35 paletted file, interlaced",
        b"s35n3p04" => "35x35 paletted file, no interlacing",
        b"s36i3p04" => "36x36 paletted file, interlaced",
        b"s36n3p04" => "36x36 paletted file, no interlacing",
        b"s37i3p04" => "37x37 paletted file, interlaced",
        b"s37n3p04" => "37x37 paletted file, no interlacing",
        b"s38i3p04" => "38x38 paletted file, interlaced",
        b"s38n3p04" => "38x38 paletted file, no interlacing",
        b"s39i3p04" => "39x39 paletted file, interlaced",
        b"s39n3p04" => "39x39 paletted file, no interlacing",
        b"s40i3p04" => "40x40 paletted file, interlaced",
        b"s40n3p04" => "40x40 paletted file, no interlacing",
        b"tbbn0g04" => "transparent, black background chunk",
        b"tbbn2c16" => "transparent, blue background chunk",
        b"tbbn3p08" => "transparent, black background chunk",
        b"tbgn2c16" => "transparent, green background chunk",
        b"tbgn3p08" => "transparent, light-gray background chunk",
        b"tbrn2c08" => "transparent, red background chunk",
        b"tbwn0g16" => "transparent, white background chunk",
        b"tbwn3p08" => "transparent, white background chunk",
        b"tbyn3p08" => "transparent, yellow background chunk",
        b"tp0n0g08" => "not transparent for reference (logo on gray)",
        b"tp0n2c08" => "not transparent for reference (logo on gray)",
        b"tp0n3p08" => "not transparent for reference (logo on gray)",
        b"tp1n3p08" => "transparent, but no background chunk",
        b"tm3n3p02" => "multiple levels of transparency, 3 entries",
        b"xs1n0g01" => "signature byte 1 MSBit reset to zero",
        b"xs2n0g01" => "signature byte 2 is a 'Q'",
        b"xs4n0g01" => "signature byte 4 lowercase",
        b"xs7n0g01" => "7th byte a space instead of control-Z",
        b"xcrn0g04" => "added cr bytes",
        b"xlfn0g04" => "added lf bytes",
        b"xhdn0g08" => "incorrect IHDR checksum",
        b"xc1n0g08" => "color type 1",
        b"xc9n2c08" => "color type 9",
        b"xd0n2c08" => "bit-depth 0",
        b"xd3n2c08" => "bit-depth 3",
        b"xd9n2c08" => "bit-depth 99",
        b"xdtn0g01" => "missing IDAT chunk",
        b"xcsn0g01" => "incorrect IDAT checksum",
        b"z00n2c08" => "color, no interlacing, compression level 0 (none)",
        b"z03n2c08" => "color, no interlacing, compression level 3",
        b"z06n2c08" => "color, no interlacing, compression level 6 (default)",
        b"z09n2c08" => "color, no interlacing, compression level 9 (maximum)",
        _ => bail!("unknown test file: {:?}", file_path),
    };

    Ok(PNGSuiteTestCase {
        test_desc,
        should_fail,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;
    use std::fs;

    #[test]
    fn parse_every_file() -> Result<()> {
        for entry in fs::read_dir("./test_suite")? {
            let path = entry?.path();

            if let Some(true) = path
                .extension()
                .and_then(OsStr::to_str)
                .map(|ext| ext.eq_ignore_ascii_case("png"))
            {
                assert!(parse_test_file(&path).is_ok(), "Failed: {:?}", path);
            }
        }

        Ok(())
    }
}
