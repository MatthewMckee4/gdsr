/// A GDS layer number (0–255 in spec, stored as `u16`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Layer(u16);

impl Layer {
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u16 {
        self.0
    }
}

impl std::fmt::Display for Layer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A GDS data type number (0–255 in spec, stored as `u16`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct DataType(u16);

impl DataType {
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u16 {
        self.0
    }
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An angle measured in radians.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Radians(pub f64);

impl Radians {
    pub const PI: Self = Self(std::f64::consts::PI);
    pub const TAU: Self = Self(std::f64::consts::TAU);
    pub const FRAC_PI_2: Self = Self(std::f64::consts::FRAC_PI_2);
    pub const FRAC_PI_4: Self = Self(std::f64::consts::FRAC_PI_4);

    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> f64 {
        self.0
    }

    /// Returns the sine of the angle.
    pub fn sin(self) -> f64 {
        self.0.sin()
    }

    /// Returns the cosine of the angle.
    pub fn cos(self) -> f64 {
        self.0.cos()
    }

    /// Converts to degrees.
    pub fn to_degrees(self) -> Degrees {
        Degrees(self.0.to_degrees())
    }
}

impl std::fmt::Display for Radians {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::ops::Add for Radians {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::AddAssign for Radians {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl std::ops::Sub for Radians {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl std::ops::Neg for Radians {
    type Output = Self;
    fn neg(self) -> Self {
        Self(-self.0)
    }
}

/// An angle measured in degrees.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Degrees(pub f64);

impl Degrees {
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> f64 {
        self.0
    }

    /// Converts to radians.
    pub fn to_radians(self) -> Radians {
        Radians(self.0.to_radians())
    }
}

impl std::fmt::Display for Degrees {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

/// A mapping from one (`Layer`, `DataType`) pair to another, used for bulk layer remapping.
pub type LayerMapping = std::collections::HashMap<(Layer, DataType), (Layer, DataType)>;
