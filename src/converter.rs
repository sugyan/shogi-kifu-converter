//! Converters for [`jkf::JsonKifuFormat`](crate::jkf::JsonKifuFormat)
//!
//! Also provides implementation [`shogi_core::ToUsi`] for [`jkf::JsonKifuFormat`](crate::jkf::JsonKifuFormat)

mod csa;
mod kakinoki;
mod ki2;
mod kif;

pub use self::csa::ToCsa;
pub use self::ki2::ToKi2;
pub use self::kif::ToKif;
use crate::jkf::JsonKifuFormat;
use shogi_core::{PartialPosition, Position, ToUsi};

/// Writes `startpos` or `sfen …`, then ` moves …` up to the first special: the arguments
/// of a USI `position` command.
///
/// A record [`normalize`](JsonKifuFormat::normalize) accepted always converts to a
/// [`Position`]. One built by hand might not, and `fmt::Result` cannot say why, so
/// nothing is written for it.
impl ToUsi for JsonKifuFormat {
    fn to_usi<W: std::fmt::Write>(&self, sink: &mut W) -> std::fmt::Result {
        let pos = match Position::try_from(self) {
            Ok(pos) => pos,
            Err(_) => return Ok(()),
        };
        if pos.initial_position() == &PartialPosition::startpos() {
            sink.write_str("startpos")?;
        } else {
            sink.write_str("sfen ")?;
            pos.initial_position().to_sfen(sink)?;
        }
        if !pos.moves().is_empty() {
            sink.write_str(" moves")?;
            for mv in pos.moves() {
                sink.write_str(" ")?;
                mv.to_usi(sink)?;
            }
        }
        Ok(())
    }
}
