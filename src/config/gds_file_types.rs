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
            0x00 => Ok(GDSRecord::Header),
            0x01 => Ok(GDSRecord::BgnLib),
            0x02 => Ok(GDSRecord::LibName),
            0x03 => Ok(GDSRecord::Units),
            0x04 => Ok(GDSRecord::EndLib),
            0x05 => Ok(GDSRecord::BgnStr),
            0x06 => Ok(GDSRecord::StrName),
            0x07 => Ok(GDSRecord::EndStr),
            0x08 => Ok(GDSRecord::Boundary),
            0x09 => Ok(GDSRecord::Path),
            0x0A => Ok(GDSRecord::SRef),
            0x0B => Ok(GDSRecord::ARef),
            0x0C => Ok(GDSRecord::Text),
            0x0D => Ok(GDSRecord::Layer),
            0x0E => Ok(GDSRecord::DataType),
            0x0F => Ok(GDSRecord::Width),
            0x10 => Ok(GDSRecord::XY),
            0x11 => Ok(GDSRecord::EndEl),
            0x12 => Ok(GDSRecord::SName),
            0x13 => Ok(GDSRecord::ColRow),
            0x14 => Ok(GDSRecord::TextNode),
            0x15 => Ok(GDSRecord::Node),
            0x16 => Ok(GDSRecord::TextType),
            0x17 => Ok(GDSRecord::Presentation),
            0x18 => Ok(GDSRecord::Spacing),
            0x19 => Ok(GDSRecord::String),
            0x1A => Ok(GDSRecord::STrans),
            0x1B => Ok(GDSRecord::Mag),
            0x1C => Ok(GDSRecord::Angle),
            0x1D => Ok(GDSRecord::UInteger),
            0x1E => Ok(GDSRecord::UString),
            0x1F => Ok(GDSRecord::RefLibs),
            0x20 => Ok(GDSRecord::Fonts),
            0x21 => Ok(GDSRecord::PathType),
            0x22 => Ok(GDSRecord::Generations),
            0x23 => Ok(GDSRecord::AttrTable),
            0x24 => Ok(GDSRecord::StyTable),
            0x25 => Ok(GDSRecord::StrType),
            0x26 => Ok(GDSRecord::ElFlags),
            0x27 => Ok(GDSRecord::ElKey),
            0x28 => Ok(GDSRecord::LinkType),
            0x29 => Ok(GDSRecord::LinkKeys),
            0x2A => Ok(GDSRecord::NodeType),
            0x2B => Ok(GDSRecord::PropAttr),
            0x2C => Ok(GDSRecord::PropValue),
            0x2D => Ok(GDSRecord::Box),
            0x2E => Ok(GDSRecord::BoxType),
            0x2F => Ok(GDSRecord::Plex),
            0x30 => Ok(GDSRecord::BgnExtn),
            0x31 => Ok(GDSRecord::EndExtn),
            0x32 => Ok(GDSRecord::TapeNum),
            0x33 => Ok(GDSRecord::TapeCode),
            0x34 => Ok(GDSRecord::StrClass),
            0x35 => Ok(GDSRecord::Reserved),
            0x36 => Ok(GDSRecord::Format),
            0x37 => Ok(GDSRecord::Mask),
            0x38 => Ok(GDSRecord::EndMasks),
            0x39 => Ok(GDSRecord::LibDirSize),
            0x3A => Ok(GDSRecord::SrfName),
            0x3B => Ok(GDSRecord::LibSecur),
            0x5A => Ok(GDSRecord::RaithMbmsPath),
            0x62 => Ok(GDSRecord::RaithPxxData),
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
            0 => Ok(GDSDataType::NoData),
            1 => Ok(GDSDataType::BitArray),
            2 => Ok(GDSDataType::TwoByteSignedInteger),
            3 => Ok(GDSDataType::FourByteSignedInteger),
            4 => Ok(GDSDataType::FourByteReal),
            5 => Ok(GDSDataType::EightByteReal),
            6 => Ok(GDSDataType::AsciiString),
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
            GDSRecordData::I16(data) => write!(f, "I16 {:?}", data),
            GDSRecordData::I32(data) => write!(f, "I32 {:?}", data),
            GDSRecordData::F64(data) => write!(f, "F64 {:?}", data),
            GDSRecordData::Str(data) => write!(f, "Str {}", data),
            GDSRecordData::None => write!(f, "None"),
        }
    }
}

pub fn combine_record_and_data_type(record: GDSRecord, data_type: GDSDataType) -> u16 {
    ((record as u16) << 8) | (data_type as u16)
}

pub const MAX_POLYGON_POINTS: usize = 8000;
