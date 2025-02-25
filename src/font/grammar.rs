use std::collections::BTreeMap;

use anyhow::{anyhow, bail, Result};

pub type ShortFrac = i16;
pub type Fixed = i32;
pub type FWord = i16;
pub type UnsignedFWord = u16;
pub type F2Dot14 = i16;
pub type LongDateTime = i64;

#[derive(Debug)]
pub struct TrueTypeFontFile<'a> {
    pub font_directory: FontDirectory<'a>,
}

#[derive(Debug)]
pub struct FontDirectory<'a> {
    pub offset_sub_table: OffsetSubTable,
    pub table_directory: BTreeMap<TableTag<'a>, TableRecord>,
}

impl<'a> FontDirectory<'a> {
    pub fn get_table_record(&self, table_tag: &'a TableTag) -> Result<&TableRecord> {
        self.table_directory
            .get(table_tag)
            .ok_or_else(|| anyhow!("Failed to find TableTag: {:?}", table_tag))
    }
}

#[derive(Debug)]
pub enum ScalarType {
    TrueType,
    PostScript,
    OpenType,
}

impl TryFrom<&[u8; 4]> for ScalarType {
    type Error = anyhow::Error;

    fn try_from(value: &[u8; 4]) -> Result<Self, Self::Error> {
        let scalar_type = match value {
            b"true" | b"\x00\x01\x00\x00" => Self::TrueType,
            b"typ1" => Self::PostScript,
            b"OTTO" => Self::OpenType,
            foreign => bail!("Foreign scalar type: {:?}", foreign),
        };

        Ok(scalar_type)
    }
}

#[derive(Debug)]
pub struct OffsetSubTable {
    pub scalar_type: ScalarType,
    pub num_tables: u16,
    pub search_range: u16,
    pub entry_selector: u16,
    pub range_shift: u16,
}

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum TableTag<'a> {
    CMap,
    Glyf,
    Head,
    HHea,
    HMtx,
    Loca,
    MaxP,
    Name,
    Post,

    // Optional tags below
    CVT,
    FPgm,
    HDMx,
    Kern,
    OS2,
    Prep,

    Foreign(&'a [u8; 4]),
}

impl<'a> TryFrom<&'a [u8; 4]> for TableTag<'a> {
    type Error = anyhow::Error;

    fn try_from(value: &'a [u8; 4]) -> Result<Self, Self::Error> {
        let tag = match value {
            b"cmap" => Self::CMap,
            b"glyf" => Self::Glyf,
            b"head" => Self::Head,
            b"hhea" => Self::HHea,
            b"hmtx" => Self::HMtx,
            b"loca" => Self::Loca,
            b"maxp" => Self::MaxP,
            b"name" => Self::Name,
            b"post" => Self::Post,
            // optional tags below
            b"cvt " => Self::CVT,
            b"fpgm" => Self::FPgm,
            b"hdmx" => Self::HDMx,
            b"kern" => Self::Kern,
            b"OS/2" => Self::OS2,
            b"prep" => Self::Prep,
            foreign => Self::Foreign(foreign),
        };

        Ok(tag)
    }
}

impl<'a> TableTag<'a> {
    pub const fn is_required(&self) -> bool {
        match self {
            Self::CMap
            | Self::Glyf
            | Self::Head
            | Self::HHea
            | Self::HMtx
            | Self::Loca
            | Self::MaxP
            | Self::Name
            | Self::Post => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct TableRecord {
    pub _checksum: u32,
    pub offset: u32,
    pub length: u32,
}

#[derive(Debug)]
pub struct HeadTable {
    pub font_revision: Fixed,
    pub checksum_adjustment: u32,
    pub magic_number: u32,
    pub flags: u16,
    pub units_per_em: u16,
    pub created: LongDateTime,
    pub modified: LongDateTime,
    pub x_min: FWord,
    pub y_min: FWord,
    pub x_max: FWord,
    pub y_max: FWord,
    pub mac_style: u16,
    pub lowest_rec_ppem: u16,
    pub font_direction_hint: i16,
    pub index_to_loc_format: bool,
    pub glyph_data_format: i16,
}

#[derive(Debug)]
pub struct HHeaTable {
    pub ascent: FWord,
    pub descent: FWord,
    pub line_gap: FWord,
    pub advance_width_max: UnsignedFWord,
    pub min_left_side_bearing: FWord,
    pub min_right_side_bearing: FWord,
    pub x_max_extent: FWord,
    pub caret_slope_rise: i16,
    pub caret_slope_run: i16,
    pub caret_offset: FWord,
    pub _reserved: i64,
    pub metric_data_format: i16,
    pub num_of_long_hor_metrics: u16,
}

#[derive(Debug)]
pub struct MaxPTable {
    pub num_glyphs: u16,
    pub max_points: u16,
    pub max_contours: u16,
    pub max_component_points: u16,
    pub max_component_contours: u16,
    pub max_zones: u16,
    pub max_twilight_points: u16,
    pub max_storage: u16,
    pub max_function_defs: u16,
    pub max_instruction_defs: u16,
    pub max_stack_elements: u16,
    pub max_size_of_instructions: u16,
    pub max_component_elements: u16,
    pub max_component_depth: u16,
}

#[derive(Debug)]
pub struct LongHorizontalMetric {
    pub advance_width: u16,
    pub left_side_bearing: i16,
}

#[derive(Debug)]
pub struct HMtxTable {
    pub h_metrics: Vec<LongHorizontalMetric>,
    pub left_side_bearing: Vec<FWord>,
}

#[derive(Debug)]
pub struct GlyphTable(pub Vec<(GlyphDescription, Glyph)>);

#[derive(Debug)]
pub struct GlyphDescription {
    pub number_of_contours: i16,
    pub x_min: FWord,
    pub y_min: FWord,
    pub x_max: FWord,
    pub y_max: FWord,
}

impl GlyphDescription {
    pub(crate) fn is_simple(&self) -> bool {
        self.number_of_contours >= 0
    }
}

#[derive(Debug)]
pub enum Glyph {
    Simple {
        end_points_of_contours: Vec<u16>,
        instruction_length: u16,
        instructions: Vec<u8>,
        flags: Vec<SimpleGlyphFlag>,
        coordinates: Vec<(i16, i16)>,
    },
    Compound {
        components: Vec<ComponentGlyph>,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct SimpleGlyphFlag(pub u8);

impl SimpleGlyphFlag {
    pub const fn on_curve(&self) -> bool {
        self.0 & 0b1 == 1
    }

    pub const fn x_short_vector(&self) -> bool {
        (self.0 & 0b10) >> 1 == 1
    }

    pub const fn y_short_vector(&self) -> bool {
        (self.0 & 0b100) >> 2 == 1
    }

    pub const fn should_repeat(&self) -> bool {
        (self.0 & 0b1000) >> 3 == 1
    }

    pub const fn repeat_or_sign_x_short_vector(&self) -> bool {
        (self.0 & 0b10000) >> 4 == 1
    }

    pub const fn repeat_or_sign_y_short_vector(&self) -> bool {
        (self.0 & 0b100000) >> 5 == 1
    }
}

#[derive(Debug)]
pub enum ComponentGlyphArgument {
    Point(u16),
    Coord(i16),
}

#[derive(Debug)]
pub struct ComponentGlyph {
    pub flag: ComponentGlyphFlag,
    pub glyph_index: u16,
    pub arg_1: ComponentGlyphArgument,
    pub arg_2: ComponentGlyphArgument,
    pub transformation: ComponentGlyphTransformation,
}

#[derive(Debug, Clone, Copy)]
pub struct ComponentGlyphFlag(pub u16);

impl ComponentGlyphFlag {
    pub const fn arg1_2_are_words(&self) -> bool {
        self.0 & 0b1 == 1
    }

    pub const fn args_are_xy_values(&self) -> bool {
        self.0 & 0b10 << 1 == 1
    }

    pub const fn round_xy_to_grid(&self) -> bool {
        self.0 & 0b100 << 2 == 1
    }

    pub const fn we_have_a_scale(&self) -> bool {
        self.0 & 0b1000 << 3 == 1
    }

    pub const fn more_components(&self) -> bool {
        self.0 & 0b100000 << 5 == 1
    }

    pub const fn we_have_an_xy_scale(&self) -> bool {
        self.0 & 0b1000000 << 6 == 1
    }

    pub const fn we_have_two_by_two(&self) -> bool {
        self.0 & 0b10000000 << 7 == 1
    }

    pub const fn we_have_instructions(&self) -> bool {
        self.0 & 0b100000000 << 8 == 1
    }

    pub const fn use_my_metrics(&self) -> bool {
        self.0 & 0b1000000000 << 9 == 1
    }

    pub const fn overlap_compound(&self) -> bool {
        self.0 & 0b10000000000 << 10 == 1
    }
}

#[derive(Debug)]
pub enum ComponentGlyphTransformation {
    Uniform(F2Dot14),
    NonUniform {
        x_scale: F2Dot14,
        y_scale: F2Dot14,
    },
    Affine {
        x_scale: F2Dot14,
        scale_01: F2Dot14,
        scale_10: F2Dot14,
        y_scale: F2Dot14,
    },
}
