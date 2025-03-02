use std::collections::BTreeMap;

use anyhow::{anyhow, bail, ensure, Result};

use crate::util::read_bytes::{U16_BYTES, U8_BYTES};

pub type ShortFrac = i16;
pub type Fixed = i32;
pub type FWord = i16;
pub type UnsignedFWord = u16;
pub type F2Dot14 = i16;
pub type LongDateTime = i64;

#[derive(Debug)]
pub struct TrueTypeFontFile<'a> {
    pub font_directory: FontDirectory<'a>,
    pub head_table: HeadTable,
    pub hhea_table: HHeaTable,
    pub maxp_table: MaxPTable,
    pub loca_table: Vec<u32>,
    pub hmtx_table: HMtxTable,
    pub cmap_table: BTreeMap<CMapSubtable, Vec<PlatformDouble>>,
    pub glyph_table: GlyphTable,
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CMapSubtable {
    Zero(CMapFormat0),
    // Two(CMapFormat2),
    Four(CMapFormat4),
    // Six(CMapFormat6),
    // Eight(CMapFormat8),
    // Ten(CMapFormat10),
    Twelve(CMapFormat12),
    // Fourteen(CMapFormat14),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CMapFormat0 {
    pub language: u16,
    pub glyph_index_array: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CMapFormat2 {
    pub language: u16,
    pub subheader_keys: [u16; 256],
    pub subheaders: Vec<CMapSubHeader>,
    pub glyph_index_array: Vec<u16>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CMapSubHeader {
    pub first_code: u16,
    pub entry_count: u16,
    pub id_delta: i16,
    pub id_range_offset: u16,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CMapFormat4 {
    pub language: u16,
    pub seg_count_x2: u16,
    pub search_range: u16,
    pub entry_selector: u16,
    pub range_shift: u16,
    pub end_codes: Vec<u16>,
    pub start_codes: Vec<u16>,
    pub id_deltas: Vec<u16>,
    pub id_range_offset: Vec<u16>,
    pub glyph_index_array: Vec<u16>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CMapFormat6 {
    pub language: u16,
    pub first_code: u16,
    pub glyph_index_array: Vec<u16>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CMapFormat8 {
    pub language: u32,
    pub is_32: [u8; 65536],
    pub groups: Vec<CMapIndividualGroup>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CMapIndividualGroup {
    pub start_char_code: u32,
    pub end_char_code: u32,
    pub start_glyph_code: u32,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CMapFormat10 {
    pub language: u32,
    pub start_char_code: u32,
    pub num_chars: u32,
    pub glyphs: Vec<u16>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CMapFormat12 {
    pub language: u32,
    pub groups: Vec<CMapIndividualGroup>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CMapFormat14 {
    pub records: Vec<VariationSelectorRecord>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VariationSelectorRecord {
    pub variation_selector: u32, // actually a u24
    pub default_uvs_offset: u32,
    pub non_default_uvs_offset: u32,
}

#[derive(Debug)]
pub struct DefaultUVSTable {
    pub unicode_value_ranges: Vec<UnicodeValueRange>,
}

#[derive(Debug)]
pub struct UnicodeValueRange {
    pub start_unicode_value: u32, // note this is a u24
    pub additional_count: u8,
}

#[derive(Debug)]
pub struct NonDefaultUVSTable {
    pub uvs_mappings: Vec<UnicodeValueMap>,
}

#[derive(Debug)]
pub struct UnicodeValueMap {
    pub unicode_value: u32, // note this is a u24
    pub glyph_id: u16,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PlatformDouble {
    pub platform: Platform,
    pub platform_specific_id: u16,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Platform {
    Unicode,
    Macintosh,
    Microsoft,
}

impl TryFrom<u16> for Platform {
    type Error = anyhow::Error;

    fn try_from(value: u16) -> std::result::Result<Self, Self::Error> {
        let platform_id = match value {
            0 => Self::Unicode,
            1 => Self::Macintosh,
            2 => bail!("Reserved platform id. Do not use."),
            3 => Self::Microsoft,
            foreign => bail!("Foreign platform id: {}.", foreign),
        };

        Ok(platform_id)
    }
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
    pub index_to_loc_format: IndexToLocFormat,
    pub glyph_data_format: i16,
}

#[derive(Debug)]
pub enum IndexToLocFormat {
    Short,
    Long,
}

impl TryFrom<i16> for IndexToLocFormat {
    type Error = anyhow::Error;

    fn try_from(value: i16) -> std::result::Result<Self, Self::Error> {
        ensure!(
            value == 0 || value == 1,
            "Expected boolean flag when parsing index to loc format."
        );

        if value == 0 {
            return Ok(Self::Short);
        }

        return Ok(Self::Long);
    }
}

impl IndexToLocFormat {
    pub const fn size(&self) -> usize {
        match self {
            Self::Short => U16_BYTES,
            Self::Long => U8_BYTES,
        }
    }
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
pub struct GlyphTable {
    pub glyphs: Vec<Glyph>,
}

#[derive(Debug)]
pub struct Glyph {
    pub description: GlyphDescription,
    pub data: GlyphData,
}

impl Glyph {
    pub const fn is_simple(&self) -> bool {
        matches!(self.data, GlyphData::Simple(_))
    }

    pub fn draw_to_canvas(&self, key: usize) -> String {
        let GlyphData::Simple(simple_glyph) = &self.data else {
            todo!("how does compound glyphs look on canvas?");
        };

        let mut out = String::new();
        out += &format!("ctx{key}.translate(0, newCanvas{key}.height - 300);\n");
        out += &format!("ctx{key}.scale(0.5, -0.5);\n");

        out += &format!("ctx{key}.beginPath()\n");

        let mut start_index = 0;

        for end_index in &simple_glyph.end_points_of_contours {
            let end_index = *end_index as usize;

            let (x_start, y_start) = simple_glyph.coordinates[start_index];
            out += &format!("ctx{key}.moveTo({}, {});\n", x_start, y_start);
            for i in start_index + 1..=end_index {
                let (x, y) = simple_glyph.coordinates[i];
                out += &format!("ctx{key}.lineTo({}, {});\n", x, y);
            }

            out += &format!("ctx{key}.closePath();\n");

            start_index = end_index + 1;
        }

        out += &format!("ctx{key}.lineWidth = 9;\n");
        out += &format!("ctx{key}.stroke();\n");

        out
    }
}

#[derive(Debug)]
pub struct GlyphDescription {
    pub number_of_contours: i16,
    pub x_min: FWord,
    pub y_min: FWord,
    pub x_max: FWord,
    pub y_max: FWord,
}

impl GlyphDescription {
    pub const fn width(&self) -> usize {
        (self.x_max - self.x_min) as usize
    }

    pub const fn height(&self) -> usize {
        (self.y_max - self.y_min) as usize
    }

    pub(crate) const fn is_simple(&self) -> bool {
        self.number_of_contours >= 0
    }
}

#[derive(Debug)]
pub enum GlyphData {
    Simple(SimpleGlyph),
    Compound(CompoundGlyph),
}

#[derive(Debug)]
pub struct SimpleGlyph {
    pub end_points_of_contours: Vec<u16>,
    pub instruction_length: u16,
    pub instructions: Vec<u8>,
    pub flags: Vec<SimpleGlyphFlag>,
    pub coordinates: Vec<(i16, i16)>,
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

    pub const fn x_is_same_or_sign(&self) -> bool {
        (self.0 & 0b10000) >> 4 == 1
    }

    pub const fn y_is_same_or_sign(&self) -> bool {
        (self.0 & 0b100000) >> 5 == 1
    }
}

#[derive(Debug)]
pub struct CompoundGlyph {
    pub components: Vec<ComponentGlyph>,
}

#[derive(Debug)]
pub struct ComponentGlyph {
    pub flag: ComponentGlyphFlag,
    pub glyph_index: u16,
    pub arg_1: ComponentGlyphArgument,
    pub arg_2: ComponentGlyphArgument,
    pub transformation: ComponentGlyphTransformation,
}

#[derive(Debug)]
pub enum ComponentGlyphArgument {
    Point(u16),
    Coord(i16),
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
