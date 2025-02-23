use anyhow::{bail, Result};

pub type ShortFrac = i16;
pub type Fixed = i32;
pub type FWord = i16;
pub type UnsignedFWord = u16;
pub type F2Dot14 = i16;
pub type LongDateTime = i64;

#[derive(Debug)]
pub struct TrueTypeFontFile<'a> {
    pub(crate) font_directory: FontDirectory<'a>,
}

#[derive(Debug)]
pub struct FontDirectory<'a> {
    pub(crate) offset_sub_table: OffsetSubTable,
    pub(crate) table_directory: Vec<TableRecord<'a>>,
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
    pub(crate) scalar_type: ScalarType,
    pub(crate) num_tables: u16,
    pub(crate) search_range: u16,
    pub(crate) entry_selector: u16,
    pub(crate) range_shift: u16,
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
pub struct TableRecord<'a> {
    pub(crate) table_tag: TableTag<'a>,
    pub(crate) checksum: u32,
    pub(crate) offset: u32,
    pub(crate) length: u32,
}

#[derive(Debug)]
pub enum Table {
    CMap,
    Glyf(Glyph),
    Head(Head),
    HHea(HHea),
    HMtx,
    Loca,
    MaxP,
    Name,
    Post,
}

#[derive(Debug)]
pub struct Head {
    pub(crate) font_revision: Fixed,
    pub(crate) checksum_adjustment: u32,
    pub(crate) magic_number: u32,
    pub(crate) flags: u16,
    pub(crate) units_per_em: u16,
    pub(crate) created: LongDateTime,
    pub(crate) modified: LongDateTime,
    pub(crate) x_min: FWord,
    pub(crate) y_min: FWord,
    pub(crate) x_max: FWord,
    pub(crate) y_max: FWord,
    pub(crate) mac_style: u16,
    pub(crate) lowest_rec_ppem: u16,
    pub(crate) font_direction_hint: i16,
    pub(crate) index_to_loc_format: bool,
    pub(crate) glyph_data_format: i16,
}

#[derive(Debug)]
pub struct HHea {
    pub(crate) ascent: FWord,
    pub(crate) descent: FWord,
    pub(crate) line_gap: FWord,
    pub(crate) advance_width_max: UnsignedFWord,
    pub(crate) min_left_side_bearing: FWord,
    pub(crate) min_right_side_bearing: FWord,
    pub(crate) x_max_extent: FWord,
    pub(crate) caret_slope_rise: i16,
    pub(crate) caret_slope_run: i16,
    pub(crate) caret_offset: FWord,
    pub(crate) _reserved: i64,
    pub(crate) metric_data_format: i16,
    pub(crate) num_of_long_hor_metrics: u16,
}

#[derive(Debug)]
pub struct Glyph {
    pub(crate) number_of_contours: i16,
    pub(crate) x_min: FWord,
    pub(crate) y_min: FWord,
    pub(crate) x_max: FWord,
    pub(crate) y_max: FWord,
    pub(crate) flags: Vec<GlyphFlag>,
}

#[derive(Debug, Clone, Copy)]
pub struct GlyphFlag(pub u8);

impl GlyphFlag {
    pub(crate) const fn on_curve(&self) -> bool {
        self.0 & 0b1 == 1
    }

    pub(crate) const fn x_short_vector(&self) -> bool {
        (self.0 & 0b10) >> 1 == 1
    }

    pub(crate) const fn y_short_vector(&self) -> bool {
        (self.0 & 0b100) >> 2 == 1
    }

    pub(crate) const fn should_repeat(&self) -> bool {
        (self.0 & 0b1000) >> 3 == 1
    }

    pub(crate) const fn repeat_or_sign_x_short_vector(&self) -> bool {
        (self.0 & 0b10000) >> 4 == 1
    }

    pub(crate) const fn repeat_or_sign_y_short_vector(&self) -> bool {
        (self.0 & 0b100000) >> 5 == 1
    }
}
