#[repr(u8)]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum GDSRecord {
    Header = 0x00,
    BgnLib = 0x01,
    LibName = 0x02,
    Units = 0x03,
    EndLib = 0x04,
    BgnStr = 0x05,
    StrName = 0x06,
    EndStr = 0x07,
    Boundary = 0x08,
    Path = 0x09,
    SRef = 0x0A,
    ARef = 0x0B,
    Text = 0x0C,
    Layer = 0x0D,
    DataType = 0x0E,
    Width = 0x0F,
    XY = 0x10,
    EndEl = 0x11,
    SName = 0x12,
    ColRow = 0x13,
    TextNode = 0x14,
    Node = 0x15,
    TextType = 0x16,
    Presentation = 0x17,
    Spacing = 0x18,
    String = 0x19,
    STrans = 0x1A,
    Mag = 0x1B,
    Angle = 0x1C,
    UInteger = 0x1D,
    UString = 0x1E,
    RefLibs = 0x1F,
    Fonts = 0x20,
    PathType = 0x21,
    Generations = 0x22,
    AttrTable = 0x23,
    StyTable = 0x24,
    StrType = 0x25,
    ElFlags = 0x26,
    ElKey = 0x27,
    LinkType = 0x28,
    LinkKeys = 0x29,
    NodeType = 0x2A,
    PropAttr = 0x2B,
    PropValue = 0x2C,
    Box = 0x2D,
    BoxType = 0x2E,
    Plex = 0x2F,
    BgnExtn = 0x30,
    EndExtn = 0x31,
    TapeNum = 0x32,
    TapeCode = 0x33,
    StrClass = 0x34,
    Reserved = 0x35,
    Format = 0x36,
    Mask = 0x37,
    EndMasks = 0x38,
    LibDirSize = 0x39,
    SrfName = 0x3A,
    LibSecur = 0x3B,
    RaithMbmsPath = 0x5A,
    RaithPxxData = 0x62,
}

impl TryFrom<u8> for GDSRecord {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Header),
            0x01 => Ok(Self::BgnLib),
            0x02 => Ok(Self::LibName),
            0x03 => Ok(Self::Units),
            0x04 => Ok(Self::EndLib),
            0x05 => Ok(Self::BgnStr),
            0x06 => Ok(Self::StrName),
            0x07 => Ok(Self::EndStr),
            0x08 => Ok(Self::Boundary),
            0x09 => Ok(Self::Path),
            0x0A => Ok(Self::SRef),
            0x0B => Ok(Self::ARef),
            0x0C => Ok(Self::Text),
            0x0D => Ok(Self::Layer),
            0x0E => Ok(Self::DataType),
            0x0F => Ok(Self::Width),
            0x10 => Ok(Self::XY),
            0x11 => Ok(Self::EndEl),
            0x12 => Ok(Self::SName),
            0x13 => Ok(Self::ColRow),
            0x14 => Ok(Self::TextNode),
            0x15 => Ok(Self::Node),
            0x16 => Ok(Self::TextType),
            0x17 => Ok(Self::Presentation),
            0x18 => Ok(Self::Spacing),
            0x19 => Ok(Self::String),
            0x1A => Ok(Self::STrans),
            0x1B => Ok(Self::Mag),
            0x1C => Ok(Self::Angle),
            0x1D => Ok(Self::UInteger),
            0x1E => Ok(Self::UString),
            0x1F => Ok(Self::RefLibs),
            0x20 => Ok(Self::Fonts),
            0x21 => Ok(Self::PathType),
            0x22 => Ok(Self::Generations),
            0x23 => Ok(Self::AttrTable),
            0x24 => Ok(Self::StyTable),
            0x25 => Ok(Self::StrType),
            0x26 => Ok(Self::ElFlags),
            0x27 => Ok(Self::ElKey),
            0x28 => Ok(Self::LinkType),
            0x29 => Ok(Self::LinkKeys),
            0x2A => Ok(Self::NodeType),
            0x2B => Ok(Self::PropAttr),
            0x2C => Ok(Self::PropValue),
            0x2D => Ok(Self::Box),
            0x2E => Ok(Self::BoxType),
            0x2F => Ok(Self::Plex),
            0x30 => Ok(Self::BgnExtn),
            0x31 => Ok(Self::EndExtn),
            0x32 => Ok(Self::TapeNum),
            0x33 => Ok(Self::TapeCode),
            0x34 => Ok(Self::StrClass),
            0x35 => Ok(Self::Reserved),
            0x36 => Ok(Self::Format),
            0x37 => Ok(Self::Mask),
            0x38 => Ok(Self::EndMasks),
            0x39 => Ok(Self::LibDirSize),
            0x3A => Ok(Self::SrfName),
            0x3B => Ok(Self::LibSecur),
            0x5A => Ok(Self::RaithMbmsPath),
            0x62 => Ok(Self::RaithPxxData),
            _ => Err(()),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum GDSDataType {
    NoData = 0,
    BitArray = 1,
    TwoByteSignedInteger = 2,
    FourByteSignedInteger = 3,
    FourByteReal = 4,
    EightByteReal = 5,
    AsciiString = 6,
}

impl TryFrom<u8> for GDSDataType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::NoData),
            1 => Ok(Self::BitArray),
            2 => Ok(Self::TwoByteSignedInteger),
            3 => Ok(Self::FourByteSignedInteger),
            4 => Ok(Self::FourByteReal),
            5 => Ok(Self::EightByteReal),
            6 => Ok(Self::AsciiString),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum GDSRecordData {
    I16(Vec<i16>),
    I32(Vec<i32>),
    F64(Vec<f64>),
    Str(String),
    None,
}

impl std::fmt::Display for GDSRecordData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::I16(data) => write!(f, "I16 {data:?}"),
            Self::I32(data) => write!(f, "I32 {data:?}"),
            Self::F64(data) => write!(f, "F64 {data:?}"),
            Self::Str(data) => write!(f, "Str {data:?}"),
            Self::None => write!(f, "None"),
        }
    }
}

pub const fn combine_record_and_data_type(record: GDSRecord, data_type: GDSDataType) -> u16 {
    ((record as u16) << 8) | (data_type as u16)
}
