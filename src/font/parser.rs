use std::collections::BTreeMap;

use super::grammar::{
    CMapFormat0, CMapFormat4, CMapFormat6, CMapSubtable, ComponentGlyph, ComponentGlyphArgument,
    ComponentGlyphFlag, ComponentGlyphTransformation, F2Dot14, FWord, Fixed, FontDirectory, Glyph,
    GlyphDescription, GlyphTable, HHeaTable, HMtxTable, HeadTable, LongDateTime,
    LongHorizontalMetric, MaxPTable, OffsetSubTable, ScalarType, SimpleGlyphFlag, TableRecord,
    TableTag, TrueTypeFontFile, UnsignedFWord,
};

use crate::font::grammar::{Platform, PlatformDouble};
use crate::util::read_bytes::{U16_BYTES, U32_BYTES, U64_BYTES, U8_BYTES};
use crate::{eof, read};
use anyhow::{bail, ensure, Result};

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

            let offset = self.cursor;
            let head = self.parse_head_table()?;
            debug_assert_eq!(self.cursor - offset, head_table_record.length as usize);

            head
        };

        let hhea = {
            let hhea_table_record = font_directory.get_table_record(&TableTag::HHea)?;
            self.jump_to_table_record(hhea_table_record)?;

            let offset = self.cursor;
            let hhea = self.parse_hhea_table()?;
            debug_assert_eq!(self.cursor - offset, hhea_table_record.length as usize);

            hhea
        };

        let maxp = {
            let maxp_table_record = font_directory.get_table_record(&TableTag::MaxP)?;
            self.jump_to_table_record(maxp_table_record)?;

            let offset = self.cursor;
            let maxp = self.parse_maxp_table()?;
            debug_assert_eq!(self.cursor - offset, maxp_table_record.length as usize);

            maxp
        };

        let hmtx = {
            let hmtx_table_record = font_directory.get_table_record(&TableTag::HMtx)?;
            self.jump_to_table_record(&hmtx_table_record)?;

            let offset = self.cursor;
            let htmx = self.parse_hmtx_table(hhea.num_of_long_hor_metrics, maxp.num_glyphs)?;
            debug_assert_eq!(self.cursor - offset, hmtx_table_record.length as usize);

            htmx
        };

        let cmap = {
            let cmap_table_record = font_directory.get_table_record(&TableTag::CMap)?;
            self.jump_to_table_record(&cmap_table_record)?;

            let cmap = self.parse_cmap_table()?;

            cmap
        };

        let glyph = {
            let glyph_table_record = font_directory.get_table_record(&TableTag::Glyf)?;
            self.jump_to_table_record(&glyph_table_record)?;

            let offset = self.cursor;
            let glyph = self.parse_glyph_table(maxp.num_glyphs)?;
            debug_assert_eq!(self.cursor - offset, glyph_table_record.length as usize);

            glyph
        };

        dbg!(&head, &hhea, &maxp, &hmtx, &cmap, &glyph);

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

    fn parse_head_table(&mut self) -> Result<HeadTable> {
        ensure!(
            self.read_fixed()? == 0x00010000,
            "Expected fixed version (1.0)."
        );

        Ok(HeadTable {
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

    fn parse_hhea_table(&mut self) -> Result<HHeaTable> {
        ensure!(
            self.read_fixed()? == 0x00010000,
            "Expected fixed version (1.0)."
        );

        Ok(HHeaTable {
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

    fn parse_maxp_table(&mut self) -> Result<MaxPTable> {
        // note: fonts with postscript outlines use a different table struct.
        ensure!(self.read_fixed()? == 0x00010000, "Expected version 1.0");

        Ok(MaxPTable {
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

    fn parse_hmtx_table(
        &mut self,
        num_of_long_hor_metrics: u16,
        num_glyphs: u16,
    ) -> Result<HMtxTable> {
        let h_metrics = self.read_list(
            num_of_long_hor_metrics as usize,
            Self::parse_long_horizontal_metric,
        )?;

        let num_left_side_bearing = num_glyphs - num_of_long_hor_metrics;

        let left_side_bearing = self.read_list(num_left_side_bearing as usize, Self::read_fword)?;

        Ok(HMtxTable {
            h_metrics,
            left_side_bearing,
        })
    }

    fn parse_long_horizontal_metric(&mut self) -> Result<LongHorizontalMetric> {
        Ok(LongHorizontalMetric {
            advance_width: self.read_u16()?,
            left_side_bearing: self.read_i16()?,
        })
    }

    fn parse_cmap_table(&mut self) -> Result<BTreeMap<CMapSubtable, Vec<PlatformDouble>>> {
        let cmap_offset = self.cursor;
        let version = self.read_u16()?;
        ensure!(
            version == 0,
            "Expected cmap table version to be 0. Got: {:?}.",
            version
        );

        let number_of_subtables = self.read_u16()?;
        let mut mapping_subtables = BTreeMap::new();

        for _ in 0..number_of_subtables {
            let platform_double = PlatformDouble {
                platform: Platform::try_from(self.read_u16()?)?,
                platform_specific_id: self.read_u16()?,
            };

            let offset = self.read_u32()? as usize;

            mapping_subtables
                .entry(offset)
                .or_insert_with(Vec::new)
                .push(platform_double);
        }

        let mut subtables = BTreeMap::new();

        for (offset, platform_doubles) in mapping_subtables {
            self.jump(cmap_offset + offset, 0)?;

            let cmap_subtable = self.parse_cmap_subtable()?;

            if subtables.contains_key(&cmap_subtable) {
                panic!("Can cmap subtables be duplicated?");
            }

            subtables.insert(cmap_subtable, platform_doubles);
        }

        Ok(subtables)
    }

    fn parse_cmap_subtable(&mut self) -> Result<CMapSubtable> {
        let offset = self.cursor;
        let format = self.read_u16()?;
        let length = self.validate_cmap_subtable_length(&format)?;

        self.eof(length)?;

        let subtable = match format {
            0 => CMapSubtable::Zero(self.parse_cmap_subtable_format_0()?),
            4 => CMapSubtable::Four(self.parse_cmap_subtable_format_4()?),
            6 => CMapSubtable::Six(self.parse_cmap_subtable_format_6()?),
            _ => todo!(),
        };

        ensure!(self.cursor - offset == length);

        Ok(subtable)
    }

    fn validate_cmap_subtable_length(&mut self, format: &u16) -> Result<usize> {
        let subtable_length = match format {
            0 => {
                let length = self.read_u16()? as usize;
                ensure!(
                    length == 262,
                    "Expected length 262 for cmap format 0 subtable. Got: {}.",
                    length
                );

                length
            }
            4 | 6 => self.read_u16()? as usize,
            2 | 8 | 10 | 12 | 13 | 14 => todo!(),
            foreign => bail!("Unexpected cmap format: {}.", foreign),
        };

        Ok(subtable_length)
    }

    fn parse_cmap_subtable_format_0(&mut self) -> Result<CMapFormat0> {
        Ok(CMapFormat0 {
            language: self.read_u16()?,
            glyph_index_array: self.read_list(256, Self::read_u8)?,
        })
    }

    fn parse_cmap_subtable_format_4(&mut self) -> Result<CMapFormat4> {
        let language = self.read_u16()?;
        let seg_count_x2 = self.read_u16()?;
        let search_range = self.read_u16()?;
        let entry_selector = self.read_u16()?;
        let range_shift = self.read_u16()?;

        let seg_count = seg_count_x2 as usize / 2;
        let end_codes = {
            let codes = self.read_list(seg_count, Self::read_u16)?;
            ensure!(codes.len() > 0 && *codes.last().unwrap() == 0xFFFF);

            codes
        };

        let _reserved = self.read_u16()?;
        let start_codes = self.read_list(seg_count, Self::read_u16)?;

        let id_deltas = self.read_list(seg_count, Self::read_u16)?;

        let id_range_offset = self.read_list(seg_count, Self::read_u16)?;

        if !id_range_offset.iter().all(|&id| id == 0) {
            todo!("How do glyph index arrays look like when id range offsets are not zero?");
        }

        let glyph_index_array = vec![];

        Ok(CMapFormat4 {
            language,
            seg_count_x2,
            search_range,
            entry_selector,
            range_shift,
            end_codes,
            start_codes,
            id_deltas,
            id_range_offset,
            glyph_index_array,
        })
    }

    fn parse_cmap_subtable_format_6(&mut self) -> Result<CMapFormat6> {
        let language = self.read_u16()?;
        let first_code = self.read_u16()?;
        let entry_count = self.read_u16()?;
        let glyph_index_array = self.read_list(entry_count as usize, Self::read_u16)?;

        Ok(CMapFormat6 {
            language,
            first_code,
            glyph_index_array,
        })
    }

    fn parse_glyph_table(&mut self, num_glyphs: u16) -> Result<GlyphTable> {
        let mut glyphs = Vec::with_capacity(num_glyphs as usize);

        for i in 0..num_glyphs {
            let glyph_description = self.parse_glyph_description()?;

            // https://github.com/khaledhosny/ots/issues/120
            if glyph_description.number_of_contours == 0 {
                continue;
            }

            let glyph = if glyph_description.is_simple() {
                self.parse_simple_glyph(glyph_description.number_of_contours as usize)?
            } else {
                self.parse_compound_glyph()?
            };

            glyphs.push((glyph_description, glyph));
        }

        Ok(GlyphTable(glyphs))
    }

    fn parse_glyph_description(&mut self) -> Result<GlyphDescription> {
        Ok(GlyphDescription {
            number_of_contours: self.read_i16()?,
            x_min: self.read_fword()?,
            y_min: self.read_fword()?,
            x_max: self.read_fword()?,
            y_max: self.read_fword()?,
        })
    }

    fn parse_simple_glyph(&mut self, number_of_contours: usize) -> Result<Glyph> {
        let end_points_of_contours = self.read_list(number_of_contours, Self::read_u16)?;

        let instruction_length = self.read_u16()?;
        let instructions = self.read_list(instruction_length as usize, Self::read_u8)?;

        let number_of_points = *end_points_of_contours.last().unwrap() as usize + 1;

        let mut flags = Vec::new();

        while flags.len() < number_of_points {
            let flag = SimpleGlyphFlag(self.read_u8()?);
            flags.push(flag);

            if flag.should_repeat() {
                for _ in 0..self.read_u8()? {
                    flags.push(flag);
                }
            }
        }

        let mut x_coordinates = vec![];

        let mut prev_x = 0;
        for flag in &flags {
            if let Some(delta_coord) =
                self.parse_glyph_coordinate(flag.x_short_vector(), flag.x_is_same_or_sign())?
            {
                prev_x += delta_coord;
            }

            x_coordinates.push(prev_x);
        }

        let mut y_coordinates = vec![];
        let mut prev_y = 0;
        for flag in &flags {
            if let Some(delta_coord) =
                self.parse_glyph_coordinate(flag.y_short_vector(), flag.y_is_same_or_sign())?
            {
                prev_y += delta_coord;
            }

            y_coordinates.push(prev_y);
        }

        Ok(Glyph::Simple {
            end_points_of_contours,
            instruction_length,
            instructions,
            flags,
            coordinates: x_coordinates.into_iter().zip(y_coordinates).collect(),
        })
    }

    fn parse_glyph_coordinate(
        &mut self,
        is_short_vector: bool,
        same_or_sign: bool,
    ) -> Result<Option<i16>> {
        let delta = match (is_short_vector, same_or_sign) {
            (true, true) => self.read_u8()? as i16,
            (true, false) => -1 * self.read_u8()? as i16,
            (false, true) => return Ok(None),
            (false, false) => self.read_i16()?,
        };

        Ok(Some(delta))
    }

    fn parse_compound_glyph(&mut self) -> Result<Glyph> {
        let mut components = Vec::new();
        let mut flag = ComponentGlyphFlag(self.read_u16()?);

        loop {
            let glyph_index = self.read_u16()?;

            let (arg_1, arg_2) = match (flag.arg1_2_are_words(), flag.args_are_xy_values()) {
                (true, true) => (
                    ComponentGlyphArgument::Coord(self.read_i16()?),
                    ComponentGlyphArgument::Coord(self.read_i16()?),
                ),
                (false, true) => (
                    ComponentGlyphArgument::Coord(self.read_i8()? as i16),
                    ComponentGlyphArgument::Coord(self.read_i8()? as i16),
                ),
                (true, false) => (
                    ComponentGlyphArgument::Point(self.read_u16()?),
                    ComponentGlyphArgument::Point(self.read_u16()?),
                ),
                (false, false) => (
                    ComponentGlyphArgument::Point(self.read_u8()? as u16),
                    ComponentGlyphArgument::Point(self.read_u8()? as u16),
                ),
            };

            let transformation = {
                if flag.we_have_a_scale() {
                    ComponentGlyphTransformation::Uniform(self.read_f2dot14()?)
                } else if flag.we_have_an_xy_scale() {
                    ComponentGlyphTransformation::NonUniform {
                        x_scale: self.read_f2dot14()?,
                        y_scale: self.read_f2dot14()?,
                    }
                } else if flag.we_have_two_by_two() {
                    ComponentGlyphTransformation::Affine {
                        x_scale: self.read_f2dot14()?,
                        scale_01: self.read_f2dot14()?,
                        scale_10: self.read_f2dot14()?,
                        y_scale: self.read_f2dot14()?,
                    }
                } else {
                    ComponentGlyphTransformation::Uniform(1 << 14)
                }
            };

            let component_glyph = ComponentGlyph {
                flag,
                glyph_index,
                arg_1,
                arg_2,
                transformation,
            };

            components.push(component_glyph);

            if !flag.more_components() {
                break;
            }

            flag = ComponentGlyphFlag(self.read_u16()?);
        }

        Ok(Glyph::Compound { components })
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

    fn read_list<T>(
        &mut self,
        capacity: usize,
        read_fn: impl Fn(&mut Self) -> Result<T>,
    ) -> Result<Vec<T>> {
        let mut list = Vec::with_capacity(capacity);

        for _ in 0..capacity {
            list.push(read_fn(self)?)
        }

        Ok(list)
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
    read!(read_f2dot14, F2Dot14, U16_BYTES);
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
    fn test_parse_lato() -> Result<()> {
        let ttf_file = fs::read("./src/font/Lato-Regular.ttf")?;
        let _parser = TrueTypeFontParser::new(&ttf_file).parse()?;

        Ok(())
    }
}
