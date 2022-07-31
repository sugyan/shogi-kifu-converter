use crate::jkf::JsonKifFormat;
use std::fmt::{Result, Write};

/// A type that is convertible to KIF format.
pub trait ToKif {
    /// Write `self` in KIF format.
    ///
    /// This function returns Err(core::fmt::Error)
    /// if and only if it fails to write to `sink`.
    fn to_kif<W: Write>(&self, sink: &mut W) -> Result;

    /// Returns `self`'s string representation.
    fn to_kif_owned(&self) -> String {
        let mut s = String::new();
        // guaranteed to be Ok(())
        let result = self.to_kif(&mut s);
        debug_assert_eq!(result, Ok(()));
        s
    }
}

impl ToKif for JsonKifFormat {
    fn to_kif<W: Write>(&self, sink: &mut W) -> Result {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_kif() {
        assert_eq!(r#""#, JsonKifFormat::default().to_kif_owned());
    }
}
