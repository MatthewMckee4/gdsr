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
    PropAttr = 0x2B,
    PropValue = 0x2C,
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

pub fn combine_record_and_data_type(record: GDSRecord, data_type: GDSDataType) -> u16 {
    ((record as u16) << 8) | (data_type as u16)
}
