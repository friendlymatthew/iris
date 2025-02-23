#![allow(clippy::needless_lifetimes)]

use crate::png::grammar::{Filter, ImageHeader};
use crate::png::interlace::compute_pass_counts;
use anyhow::Result;

#[derive(Debug)]
pub struct ScanlineReader<'a> {
    input_buffer: &'a [u8],
    image_header: &'a ImageHeader,
}

impl<'a> ScanlineReader<'a> {
    pub(crate) fn new(input_buffer: &'a [u8], image_header: &'a ImageHeader) -> Self {
        let row_bytes = image_header.num_bytes_per_pixel() * image_header.width as usize;

        assert_eq!(
            input_buffer.len() - image_header.height as usize,
            row_bytes * image_header.height as usize
        );

        Self {
            input_buffer,
            image_header,
        }
    }

    pub(crate) fn read_lines(&self) -> Result<Vec<u8>> {
        if self.image_header.interlace_method {
            self.adam7_deinterlace()
        } else {
            self.non_interlaced()
        }
    }
}

impl<'a> ScanlineReader<'a> {
    fn non_interlaced(&self) -> Result<Vec<u8>> {
        let mut pixel_buffer =
            vec![0_u8; self.input_buffer.len() - self.image_header.height as usize];

        let bytes_per_pixel = self.image_header.num_bytes_per_pixel();
        let bytes_per_row = bytes_per_pixel * self.image_header.width as usize;

        for i in 0..self.image_header.height as usize {
            let mut row_start_idx = i * (1 + bytes_per_row);
            let filter_type = Filter::try_from(self.input_buffer[row_start_idx])?;
            row_start_idx += 1;
            let row = &self.input_buffer[row_start_idx..row_start_idx + bytes_per_row];

            let pixel_row_start = i * bytes_per_row;

            match filter_type {
                Filter::None => {
                    // the best filter.
                    pixel_buffer[pixel_row_start..pixel_row_start + bytes_per_row]
                        .copy_from_slice(row);
                }
                Filter::Sub => {
                    pixel_buffer[pixel_row_start..pixel_row_start + bytes_per_pixel]
                        .copy_from_slice(&row[0..bytes_per_pixel]);

                    for j in bytes_per_pixel..bytes_per_row {
                        let filtered = row[j]
                            .wrapping_add(pixel_buffer[pixel_row_start + j - bytes_per_pixel]);

                        pixel_buffer[pixel_row_start + j] = filtered;
                    }
                }
                Filter::Up => {
                    if pixel_row_start < bytes_per_row {
                        pixel_buffer[pixel_row_start..pixel_row_start + bytes_per_row]
                            .copy_from_slice(row);

                        continue;
                    }

                    for j in 0..bytes_per_row {
                        let filtered =
                            row[j].wrapping_add(pixel_buffer[pixel_row_start - bytes_per_row + j]);

                        pixel_buffer[pixel_row_start + j] = filtered;
                    }
                }
                Filter::Average => {
                    let has_prev_row = pixel_row_start >= bytes_per_row;

                    if has_prev_row {
                        for j in 0..bytes_per_pixel {
                            pixel_buffer[pixel_row_start + j] = row[j].wrapping_add(
                                (pixel_buffer[pixel_row_start - bytes_per_row + j] as u16 / 2)
                                    as u8,
                            )
                        }
                    } else {
                        pixel_buffer[pixel_row_start..pixel_row_start + bytes_per_pixel]
                            .copy_from_slice(&row[0..bytes_per_pixel]);
                    }

                    for j in bytes_per_pixel..bytes_per_row {
                        let mut a = pixel_buffer[pixel_row_start + j - bytes_per_pixel] as u16;

                        if has_prev_row {
                            a += pixel_buffer[pixel_row_start - bytes_per_row + j] as u16;
                        }

                        a /= 2;

                        let filtered = row[j].wrapping_add(a as u8);
                        pixel_buffer[pixel_row_start + j] = filtered;
                    }
                }
                Filter::Paeth => {
                    for j in 0..row.len() {
                        let left = if j < bytes_per_pixel {
                            0
                        } else {
                            pixel_buffer[pixel_row_start + j - bytes_per_pixel]
                        };

                        let up_left = if j < bytes_per_pixel || pixel_row_start < bytes_per_row {
                            0
                        } else {
                            pixel_buffer[pixel_row_start - bytes_per_row + j - bytes_per_pixel]
                        };

                        let filtered = row[j].wrapping_add(self.paeth(
                            left,
                            if pixel_row_start < bytes_per_row {
                                0
                            } else {
                                pixel_buffer[pixel_row_start - bytes_per_row + j]
                            },
                            up_left,
                        ));

                        pixel_buffer[pixel_row_start + j] = filtered;
                    }
                }
            }
        }

        Ok(pixel_buffer)
    }

    #[inline]
    const fn paeth(&self, left: u8, up: u8, up_left: u8) -> u8 {
        let a = left as i16;
        let b = up as i16;
        let c = up_left as i16;

        let p = a + b - c;

        let pa = (p - a).abs();
        let pb = (p - b).abs();
        let pc = (p - c).abs();

        if pa <= pb && pa <= pc {
            left
        } else if pb <= pc {
            up
        } else {
            up_left
        }
    }
}

impl<'a> ScanlineReader<'a> {
    fn adam7_deinterlace(&self) -> Result<Vec<u8>> {
        let bytes_per_pixel = self.image_header.num_bytes_per_pixel();

        let mut pixel_buffer =
            vec![
                0u8;
                bytes_per_pixel * (self.image_header.height * self.image_header.width) as usize
            ];

        let pass_counts = compute_pass_counts(self.image_header.width, self.image_header.height);
        let mut cursor = 0;

        for pass in pass_counts.into_iter() {
            let bytes_per_row = bytes_per_pixel * pass.width;

            for i in 0..pass.height {
                let mut row_start_idx = cursor + i * (1 + bytes_per_row);
                let filter_type = Filter::try_from(self.input_buffer[row_start_idx])?;
                row_start_idx += 1;

                let pixel_y = (pass.compute_y)(i);
                let row = &self.input_buffer[row_start_idx..row_start_idx + bytes_per_row];

                let _pixel_row_start = i * bytes_per_row;

                match filter_type {
                    Filter::None => {
                        for (j, pixel) in row.chunks_exact(bytes_per_pixel).enumerate() {
                            let pixel_x = (pass.compute_x)(j);

                            let index = (pixel_y * self.image_header.width as usize) + pixel_x;
                            pixel_buffer[index..index + bytes_per_pixel].copy_from_slice(pixel);
                        }
                    }
                    Filter::Sub => {
                        let mut new_row = row.to_vec();

                        for i in bytes_per_pixel..bytes_per_row {
                            let filtered = new_row[i].wrapping_add(new_row[i - bytes_per_pixel]);
                            new_row[i] = filtered;
                        }

                        for (j, pixel) in new_row.chunks_exact(bytes_per_pixel).enumerate() {
                            let pixel_x = (pass.compute_x)(j);

                            let index = (pixel_y * self.image_header.width as usize) + pixel_x;
                            pixel_buffer[index..index + bytes_per_pixel].copy_from_slice(pixel);
                        }
                    }
                    _ => todo!("What do other filters look like?"),
                }

                dbg!(filter_type);
            }

            cursor += (1 + bytes_per_row) * pass.height;
        }

        Ok(pixel_buffer)
    }
}
