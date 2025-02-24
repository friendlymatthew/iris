use std::collections::BTreeMap;

use crate::font::grammar::{
    FWord, Fixed, FontDirectory, Glyph, GlyphFlag, HHea, Head, LongDateTime, OffsetSubTable,
    ScalarType, TableRecord, TableTag, TrueTypeFontFile, UnsignedFWord,
};

use crate::util::read_bytes::{U16_BYTES, U32_BYTES, U64_BYTES, U8_BYTES};
use crate::{eof, read};
use anyhow::{anyhow, ensure, Result};

use super::grammar::MaxP;

#[derive(Debug)]
pub struct TrueTypeFontParser<'a> {
    cursor: usize,
    data: &'a [u8],
}

impl<'a> TrueTypeFontParser<'a> {
    pub const fn new(data: &'a [u8]) -> Self {
        Self { cursor: 0, data }
    }

    pub fn parse(&mut self) -> Result<TrueTypeFontFile<'a>> {
        let font_directory = self.parse_font_directory()?;

        let head = {
            let head_table_record = font_directory.get_table_record(&TableTag::Head)?;
            self.jump_to_table_record(head_table_record)?;
            self.parse_head()?
        };

        let hhea = {
            let hhea_table_record = font_directory.get_table_record(&TableTag::HHea)?;
            self.jump_to_table_record(hhea_table_record)?;
            self.parse_hhea()?
        };

        let maxp = {
            let maxp_table_record = font_directory.get_table_record(&TableTag::MaxP)?;
            self.jump_to_table_record(maxp_table_record)?;
            self.parse_maxp()?;
        };

        dbg!(&head, &hhea, &maxp);

        Ok(TrueTypeFontFile { font_directory })
    }

    fn parse_font_directory(&mut self) -> Result<FontDirectory<'a>> {
        let offset_sub_table = OffsetSubTable {
            scalar_type: ScalarType::try_from(self.read_slice::<U32_BYTES>()?)?,
            num_tables: self.read_u16()?,
            search_range: self.read_u16()?,
            entry_selector: self.read_u16()?,
            range_shift: self.read_u16()?,
        };

        let mut table_directory = BTreeMap::new();

        for _ in 0..offset_sub_table.num_tables as usize {
            // todo: what does a checksum validation look like?
            let table_tag = TableTag::try_from(self.read_slice::<U32_BYTES>()?)?;

            ensure!(
                !table_directory.contains_key(&table_tag),
                "Todo: can certain table tags appear twice?"
            );

            table_directory.insert(
                table_tag,
                TableRecord {
                    _checksum: self.read_u32()?,
                    offset: self.read_u32()?,
                    length: self.read_u32()?,
                },
            );
        }

        Ok(FontDirectory {
            offset_sub_table,
            table_directory,
        })
    }

    fn parse_head(&mut self) -> Result<Head> {
        ensure!(
            self.read_fixed()? == 0x00010000,
            "Expected fixed version (1.0)."
        );

        Ok(Head {
            font_revision: self.read_fixed()?,
            checksum_adjustment: self.read_u32()?,
            magic_number: {
                let magic = self.read_u32()?;
                ensure!(magic == 0x5F0F3CF5, "Incorrect magic number.");
                magic
            },
            flags: self.read_u16()?,
            units_per_em: self.read_u16()?,
            created: self.read_long_date_time()?,
            modified: self.read_long_date_time()?,
            x_min: self.read_fword()?,
            y_min: self.read_fword()?,
            x_max: self.read_fword()?,
            y_max: self.read_fword()?,
            mac_style: self.read_u16()?,
            lowest_rec_ppem: self.read_u16()?,
            font_direction_hint: self.read_i16()?,
            index_to_loc_format: {
                let flag = self.read_i16()?;
                ensure!(
                    flag == 0 || flag == 1,
                    "Expected boolean flag. Got: {}",
                    flag
                );

                flag == 1
            },
            glyph_data_format: {
                let b = self.read_i16()?;
                ensure!(b == 0, "Expected data format to be 0. Got: {}.", b);
                b
            },
        })
    }

    fn parse_hhea(&mut self) -> Result<HHea> {
        ensure!(
            self.read_fixed()? == 0x00010000,
            "Expected fixed version (1.0)."
        );

        Ok(HHea {
            ascent: self.read_fword()?,
            descent: self.read_fword()?,
            line_gap: self.read_fword()?,
            advance_width_max: self.read_unsigned_fword()?,
            min_left_side_bearing: self.read_fword()?,
            min_right_side_bearing: self.read_fword()?,
            x_max_extent: self.read_fword()?,
            caret_slope_rise: self.read_i16()?,
            caret_slope_run: self.read_i16()?,
            caret_offset: self.read_fword()?,
            _reserved: self.read_i64()?,
            metric_data_format: self.read_i16()?,
            num_of_long_hor_metrics: self.read_u16()?,
        })
    }

    fn parse_maxp(&mut self) -> Result<MaxP> {
        // note: fonts with postscript outlines use a different table struct.
        ensure!(self.read_fixed()? == 0x00010000, "Expected version 1.0");

        Ok(MaxP {
            num_glyphs: self.read_u16()?,
            max_points: self.read_u16()?,
            max_contours: self.read_u16()?,
            max_component_points: self.read_u16()?,
            max_component_contours: self.read_u16()?,
            max_zones: self.read_u16()?,
            max_twilight_points: self.read_u16()?,
            max_storage: self.read_u16()?,
            max_function_defs: self.read_u16()?,
            max_instruction_defs: self.read_u16()?,
            max_stack_elements: self.read_u16()?,
            max_size_of_instructions: self.read_u16()?,
            max_component_elements: self.read_u16()?,
            max_component_depth: self.read_u16()?,
        })
    }

    fn _parse_glyph(&mut self) -> Result<Glyph> {
        let mut glyph_table = Glyph {
            number_of_contours: self.read_i16()?,
            x_min: self.read_fword()?,
            y_min: self.read_fword()?,
            x_max: self.read_fword()?,
            y_max: self.read_fword()?,
            flags: vec![],
            coordinates: vec![],
        };

        if glyph_table.number_of_contours < 0 {
            todo!("How does a compound glyph parse?")
        } else {
            let number_of_contours = glyph_table.number_of_contours as usize;
            let mut end_points_of_contours = Vec::with_capacity(number_of_contours);
            for _ in 0..number_of_contours {
                end_points_of_contours.push(self.read_u16()?);
            }

            let instruction_length = self.read_u16()? as usize;
            let mut instructions = Vec::with_capacity(instruction_length);
            for _ in 0..instruction_length {
                instructions.push(self.read_u8()?);
            }

            let number_of_points = *end_points_of_contours
                .last()
                .ok_or_else(|| anyhow!("Expect at least one point of contour."))?
                as usize
                + 1;

            let mut flags = Vec::new();

            while flags.len() < number_of_points {
                let flag = GlyphFlag(self.read_u8()?);
                flags.push(flag);

                if flag.should_repeat() {
                    for _ in 0..self.read_u8()? {
                        flags.push(flag);
                    }
                }
            }

            let mut x_coordinates = vec![0]; // since the first element is relative to (0, 0)

            for flag in &flags {
                if let Some(glyph_coord) = self._parse_glyph_coordinate(
                    flag.x_short_vector(),
                    flag.repeat_or_sign_x_short_vector(),
                )? {
                    x_coordinates.push(glyph_coord);
                    continue;
                }

                x_coordinates.push(*x_coordinates.last().unwrap());
            }

            let mut y_coordinates = vec![0]; // since first element is relative to (0, 0)
            for flag in &flags {
                if let Some(glyph_coord) = self._parse_glyph_coordinate(
                    flag.y_short_vector(),
                    flag.repeat_or_sign_y_short_vector(),
                )? {
                    y_coordinates.push(glyph_coord);
                    continue;
                }

                y_coordinates.push(*y_coordinates.last().unwrap());
            }

            glyph_table.flags = flags;
            glyph_table.coordinates = x_coordinates.into_iter().zip(y_coordinates).collect();
        }

        Ok(glyph_table)
    }

    fn _parse_glyph_coordinate(
        &mut self,
        is_short_vector: bool,
        repeat_or_sign_short_flag: bool,
    ) -> Result<Option<i16>> {
        let coord_or_repeat = match (is_short_vector, repeat_or_sign_short_flag) {
            (true, signed) => {
                let coord = if signed {
                    let a = self.read_i8()? as i16;
                    dbg!(a);

                    -a
                } else {
                    self.read_u8()? as i16
                };

                Some(coord)
            }
            (false, is_repeat) => {
                if is_repeat {
                    None
                } else {
                    Some(self.read_i16()?)
                }
            }
        };

        Ok(coord_or_repeat)
    }

    eof!();

    fn jump_to_table_record(&mut self, table_record: &TableRecord) -> Result<()> {
        let (offset, length) = (table_record.offset as usize, table_record.length as usize);
        self.jump(offset, length)
    }

    fn jump(&mut self, offset: usize, length: usize) -> Result<()> {
        ensure!(offset < self.data.len(), "Offset is out of bounds.");
        self.cursor = offset;
        self.eof(length)?;

        Ok(())
    }

    fn read_slice<const N: usize>(&mut self) -> Result<&'a [u8; N]> {
        self.eof(N)?;
        let bs = &self.data[self.cursor..self.cursor + N];
        self.cursor += N;

        Ok(bs.try_into()?)
    }

    // read!(read_short_frac, ShortFrac, U16_BYTES);
    read!(read_fixed, Fixed, U32_BYTES);
    read!(read_fword, FWord, U16_BYTES);
    read!(read_unsigned_fword, UnsignedFWord, U16_BYTES);
    // read!(read_f2dot14, F2Dot14, U16_BYTES);
    read!(read_long_date_time, LongDateTime, U64_BYTES);
    read!(read_i8, i8, U8_BYTES);
    read!(read_u8, u8, U8_BYTES);
    read!(read_u16, u16, U16_BYTES);
    read!(read_u32, u32, U32_BYTES);
    read!(read_i16, i16, U16_BYTES);
    read!(read_i64, i64, U64_BYTES);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_read_papyrus() -> Result<()> {
        let ttf_file = fs::read("./src/font/Papyrus.ttf")?;
        let _parser = TrueTypeFontParser::new(&ttf_file).parse()?;

        Ok(())
    }
}
