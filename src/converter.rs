mod csa;
mod kif;

pub use self::csa::ToCsa;
pub use self::kif::ToKif;
use crate::jkf::JsonKifuFormat;
use shogi_core::{PartialPosition, Position, ToUsi};

impl ToUsi for JsonKifuFormat {
    fn to_usi<W: std::fmt::Write>(&self, sink: &mut W) -> std::fmt::Result {
        let pos = Position::try_from(self).expect("failed to convert initial to position");
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
