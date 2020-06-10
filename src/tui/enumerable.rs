/// Borrowed from Haskell
/// Should be possible to auto derive
pub trait Enum: Sized {
    fn to_enum(&self) -> u8;
    fn from_enum(i: u8) -> Self;
    fn succ(&self) -> Self {
        Self::from_enum(self.to_enum() + 1)
    }
    fn pred(&self) -> Self {
        Self::from_enum(self.to_enum() - 1)
    }
}
