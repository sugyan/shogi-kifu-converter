use crate::error::NormalizeError;
use crate::jkf::*;
use shogi_core::{LegalityChecker, PartialPosition, Square};
use shogi_legality_lite::LiteLegalityChecker;
use shogi_official_kifu::display_single_move_kansuji;

/// The largest hand count a kifu can carry: KIF spells it in kanji and stops at 九十九,
/// and SFEN takes at most two digits in one token. `normalize` rejects anything larger.
pub(crate) const MAX_HAND_COUNT: u8 = 99;

pub(crate) const HIRATE_BOARD: [[Piece; 9]; 9] = {
    #[rustfmt::skip]
    const EMP: Piece = Piece { color: None, kind: None };
    #[rustfmt::skip]
    const BFU: Piece = Piece { color: Some(Color::Black), kind: Some(Kind::FU) };
    #[rustfmt::skip]
    const BKY: Piece = Piece { color: Some(Color::Black), kind: Some(Kind::KY) };
    #[rustfmt::skip]
    const BKE: Piece = Piece { color: Some(Color::Black), kind: Some(Kind::KE) };
    #[rustfmt::skip]
    const BGI: Piece = Piece { color: Some(Color::Black), kind: Some(Kind::GI) };
    #[rustfmt::skip]
    const BKI: Piece = Piece { color: Some(Color::Black), kind: Some(Kind::KI) };
    #[rustfmt::skip]
    const BKA: Piece = Piece { color: Some(Color::Black), kind: Some(Kind::KA) };
    #[rustfmt::skip]
    const BHI: Piece = Piece { color: Some(Color::Black), kind: Some(Kind::HI) };
    #[rustfmt::skip]
    const BOU: Piece = Piece { color: Some(Color::Black), kind: Some(Kind::OU) };
    #[rustfmt::skip]
    const WFU: Piece = Piece { color: Some(Color::White), kind: Some(Kind::FU) };
    #[rustfmt::skip]
    const WKY: Piece = Piece { color: Some(Color::White), kind: Some(Kind::KY) };
    #[rustfmt::skip]
    const WKE: Piece = Piece { color: Some(Color::White), kind: Some(Kind::KE) };
    #[rustfmt::skip]
    const WGI: Piece = Piece { color: Some(Color::White), kind: Some(Kind::GI) };
    #[rustfmt::skip]
    const WKI: Piece = Piece { color: Some(Color::White), kind: Some(Kind::KI) };
    #[rustfmt::skip]
    const WKA: Piece = Piece { color: Some(Color::White), kind: Some(Kind::KA) };
    #[rustfmt::skip]
    const WHI: Piece = Piece { color: Some(Color::White), kind: Some(Kind::HI) };
    #[rustfmt::skip]
    const WOU: Piece = Piece { color: Some(Color::White), kind: Some(Kind::OU) };
    [
        [WKY, EMP, WFU, EMP, EMP, EMP, BFU, EMP, BKY],
        [WKE, WKA, WFU, EMP, EMP, EMP, BFU, BHI, BKE],
        [WGI, EMP, WFU, EMP, EMP, EMP, BFU, EMP, BGI],
        [WKI, EMP, WFU, EMP, EMP, EMP, BFU, EMP, BKI],
        [WOU, EMP, WFU, EMP, EMP, EMP, BFU, EMP, BOU],
        [WKI, EMP, WFU, EMP, EMP, EMP, BFU, EMP, BKI],
        [WGI, EMP, WFU, EMP, EMP, EMP, BFU, EMP, BGI],
        [WKE, WHI, WFU, EMP, EMP, EMP, BFU, BKA, BKE],
        [WKY, EMP, WFU, EMP, EMP, EMP, BFU, EMP, BKY],
    ]
};

/// Every preset that names a layout: all but `PresetOther`.
const PRESETS: [Preset; 16] = [
    Preset::PresetHirate,
    Preset::PresetKY,
    Preset::PresetKYR,
    Preset::PresetKA,
    Preset::PresetHI,
    Preset::PresetHIKY,
    Preset::Preset2,
    Preset::Preset3,
    Preset::Preset4,
    Preset::Preset5,
    Preset::Preset5L,
    Preset::Preset6,
    Preset::Preset7L,
    Preset::Preset7R,
    Preset::Preset8,
    Preset::Preset10,
];

/// The squares a handicap clears from [`HIRATE_BOARD`]. This is the one table of the
/// layouts: the position a preset stands for, CSA's `PI` line and the board
/// `normalize` folds back into a preset are all derived from it. `None` for
/// `PresetOther`, which names no layout. The layouts follow json-kifu-format's reference
/// implementation.
#[rustfmt::skip]
pub(crate) fn handicap_squares(preset: Preset) -> Option<&'static [Square]> {
    Some(match preset {
        Preset::PresetHirate => &[],
        Preset::PresetKY     => &[Square::SQ_1A],
        Preset::PresetKYR    => &[Square::SQ_9A],
        Preset::PresetKA     => &[Square::SQ_2B],
        Preset::PresetHI     => &[Square::SQ_8B],
        Preset::PresetHIKY   => &[Square::SQ_8B, Square::SQ_1A],
        Preset::Preset2      => &[Square::SQ_8B, Square::SQ_2B],
        Preset::Preset3      => &[Square::SQ_8B, Square::SQ_2B, Square::SQ_1A],
        Preset::Preset4      => &[Square::SQ_8B, Square::SQ_2B, Square::SQ_9A, Square::SQ_1A],
        Preset::Preset5      => &[Square::SQ_8B, Square::SQ_2B, Square::SQ_9A, Square::SQ_1A, Square::SQ_8A],
        Preset::Preset5L     => &[Square::SQ_8B, Square::SQ_2B, Square::SQ_9A, Square::SQ_1A, Square::SQ_2A],
        Preset::Preset6      => &[Square::SQ_8B, Square::SQ_2B, Square::SQ_9A, Square::SQ_1A, Square::SQ_8A, Square::SQ_2A],
        Preset::Preset7L     => &[Square::SQ_8B, Square::SQ_2B, Square::SQ_9A, Square::SQ_1A, Square::SQ_8A, Square::SQ_2A, Square::SQ_3A],
        Preset::Preset7R     => &[Square::SQ_8B, Square::SQ_2B, Square::SQ_9A, Square::SQ_1A, Square::SQ_8A, Square::SQ_2A, Square::SQ_7A],
        Preset::Preset8      => &[Square::SQ_8B, Square::SQ_2B, Square::SQ_9A, Square::SQ_1A, Square::SQ_8A, Square::SQ_2A, Square::SQ_7A, Square::SQ_3A],
        Preset::Preset10     => &[Square::SQ_8B, Square::SQ_2B, Square::SQ_9A, Square::SQ_1A, Square::SQ_8A, Square::SQ_2A, Square::SQ_7A, Square::SQ_3A, Square::SQ_6A, Square::SQ_4A],
        Preset::PresetOther  => return None,
    })
}

/// The board data `preset` stands for, by way of its position: the same conversion any
/// other board goes through, so the two cannot disagree on what a layout looks like.
fn layout(preset: Preset) -> Option<StateFormat> {
    let pos = PartialPosition::try_from(&Initial { preset, data: None }).ok()?;
    Initial::from(&pos).data
}

impl Piece {
    pub(crate) const fn empty() -> Self {
        Self {
            color: None,
            kind: None,
        }
    }
}

impl Kind {
    pub(crate) fn promoted(self) -> Self {
        match self {
            Kind::FU => Kind::TO,
            Kind::KY => Kind::NY,
            Kind::KE => Kind::NK,
            Kind::GI => Kind::NG,
            Kind::KA => Kind::UM,
            Kind::HI => Kind::RY,
            _ => self,
        }
    }
    pub(crate) fn unpromoted(self) -> Self {
        match self {
            Kind::TO => Kind::FU,
            Kind::NY => Kind::KY,
            Kind::NK => Kind::KE,
            Kind::NG => Kind::GI,
            Kind::UM => Kind::KA,
            Kind::RY => Kind::HI,
            _ => self,
        }
    }
}

impl Hand {
    // Driven by a record's own piece list, which is not bounded by the rules, so this
    // saturates for the same reason `decrement` does.
    pub(crate) fn increment(&mut self, kind: Kind) {
        match kind {
            Kind::FU => self.FU = self.FU.saturating_add(1),
            Kind::KY => self.KY = self.KY.saturating_add(1),
            Kind::KE => self.KE = self.KE.saturating_add(1),
            Kind::GI => self.GI = self.GI.saturating_add(1),
            Kind::KI => self.KI = self.KI.saturating_add(1),
            Kind::KA => self.KA = self.KA.saturating_add(1),
            Kind::HI => self.HI = self.HI.saturating_add(1),
            _ => unreachable!(),
        }
    }
    // Piece by piece, saturating for the same reason `increment` does.
    pub(crate) fn add(&mut self, other: &Hand) {
        self.FU = self.FU.saturating_add(other.FU);
        self.KY = self.KY.saturating_add(other.KY);
        self.KE = self.KE.saturating_add(other.KE);
        self.GI = self.GI.saturating_add(other.GI);
        self.KI = self.KI.saturating_add(other.KI);
        self.KA = self.KA.saturating_add(other.KA);
        self.HI = self.HI.saturating_add(other.HI);
    }
    // Used to work out what `AL` stands for by subtracting the board and the hands from a
    // full set, so a record with more of a kind than the standard set must not overflow.
    pub(crate) fn decrement(&mut self, kind: Kind) {
        match kind {
            Kind::FU => self.FU = self.FU.saturating_sub(1),
            Kind::KY => self.KY = self.KY.saturating_sub(1),
            Kind::KE => self.KE = self.KE.saturating_sub(1),
            Kind::GI => self.GI = self.GI.saturating_sub(1),
            Kind::KI => self.KI = self.KI.saturating_sub(1),
            Kind::KA => self.KA = self.KA.saturating_sub(1),
            Kind::HI => self.HI = self.HI.saturating_sub(1),
            _ => unreachable!(),
        }
    }
}

/// Splits `secs` into hours, minutes and seconds under 60. A `TimeFormat` in that form
/// holds at most 255:59:59, so a longer time comes to that.
pub(crate) fn hms(secs: u64) -> (u8, u8, u8) {
    let s = secs.min(255 * 3600 + 59 * 60 + 59);
    ((s / 3600) as u8, (s / 60 % 60) as u8, (s % 60) as u8)
}

fn add_timeformat(lhs: &TimeFormat, rhs: &TimeFormat) -> TimeFormat {
    // In `u64`: each field of either is a `u8`, and their sums can pass 255.
    let secs = |t: &TimeFormat| {
        u64::from(t.h.unwrap_or_default()) * 3600 + u64::from(t.m) * 60 + u64::from(t.s)
    };
    let (h, m, s) = hms(secs(lhs) + secs(rhs));
    TimeFormat { h: Some(h), m, s }
}

fn pk2k(pk: shogi_core::PieceKind) -> Kind {
    match pk {
        shogi_core::PieceKind::Pawn => Kind::FU,
        shogi_core::PieceKind::Lance => Kind::KY,
        shogi_core::PieceKind::Knight => Kind::KE,
        shogi_core::PieceKind::Silver => Kind::GI,
        shogi_core::PieceKind::Gold => Kind::KI,
        shogi_core::PieceKind::Bishop => Kind::KA,
        shogi_core::PieceKind::Rook => Kind::HI,
        shogi_core::PieceKind::King => Kind::OU,
        shogi_core::PieceKind::ProPawn => Kind::TO,
        shogi_core::PieceKind::ProLance => Kind::NY,
        shogi_core::PieceKind::ProKnight => Kind::NK,
        shogi_core::PieceKind::ProSilver => Kind::NG,
        shogi_core::PieceKind::ProBishop => Kind::UM,
        shogi_core::PieceKind::ProRook => Kind::RY,
    }
}

impl JsonKifuFormat {
    /// Replays the record from its initial position, each line up to its first special,
    /// and fills in what the source format left implicit (the from-square, `same`,
    /// `promote`, `capture`, `relative` and the total times), so that every parser hands
    /// back the same shape for the same game. What follows a special is not replayed.
    ///
    /// The `parse_*` functions call this. A value built by hand, or deserialized without
    /// [`parse_jkf_str`](crate::parser::parse_jkf_str), should go through it before it is
    /// written out: the writers rely on what this fills in, and write anything else only
    /// as far as their format can express it.
    ///
    /// A move with no `from` is a drop when it is marked 打 (`relative: H`). Otherwise
    /// its from-square is worked out from the position, as KI2 needs, and it is a drop
    /// only if no piece on the board can make it. `parse_jkf_str` and `parse_jkf_file`
    /// instead read every move with no `from` as a drop, as JKF specifies.
    ///
    /// # Errors
    ///
    /// Returns [`NormalizeError`] if a line cannot be replayed from its initial position
    /// up to its first special, if a hand in it holds more than 99 of a piece (the most a
    /// kifu can spell), or if a move entry past `moves[0]` has neither `move` nor
    /// `special`.
    pub fn normalize(&mut self) -> Result<(), NormalizeError> {
        self.normalize_with(false)
    }

    // `normalize` for a record read as JKF, which gives every move but a drop its
    // from-square.
    pub(crate) fn normalize_jkf(&mut self) -> Result<(), NormalizeError> {
        self.normalize_with(true)
    }

    // `from_given`: the record gives the from-square of every move but a drop, so a move
    // with none is a drop.
    fn normalize_with(&mut self, from_given: bool) -> Result<(), NormalizeError> {
        check_entries(self.moves.get(1..).unwrap_or_default())?;
        normalize_initial(self)?;
        let pos = if let Some(initial) = &self.initial {
            match PartialPosition::try_from(initial) {
                Ok(pos) => pos,
                Err(err) => return Err(NormalizeError::Convert(err.to_string())),
            }
        } else {
            PartialPosition::startpos()
        };
        // KIF and KI2 have no colour on a move line and assign it from the ply's
        // parity, taking Black to move first. When the initial position says
        // otherwise every colour is inverted. This covers the handicap presets, which
        // are always White to move, and a board diagram carrying a `後手番` line. A KIF
        // record that opens with a special has no move here to go by, so the KIF parser
        // inverts that one itself.
        if pos.side_to_move() == shogi_core::Color::White
            && self
                .moves
                .get(1)
                .and_then(|mf| mf.move_.map(|mmf| mmf.color == Color::Black))
                .unwrap_or_default()
        {
            invert_colors(self.moves.get_mut(1..).unwrap_or_default());
        }
        // `moves` may legitimately be empty: the JKF schema puts no lower bound on it,
        // and only `moves[0]` (the initial position's comment slot) would be missing.
        let moves = self.moves.get_mut(1..).unwrap_or_default();
        normalize_moves(moves, pos, [TimeFormat::default(); 2], from_given)?;
        Ok(())
    }
}

// Every entry past `moves[0]` stands for a ply, so it carries a move or a special (a fork
// has no `moves[0]` of its own). The writers have nothing to write for one that does not
// and leave it out, comments, time and forks included, with no way to say so, so it is
// rejected here, where an error can be returned. `normalize_moves` stops at the first
// special, so this walks the whole tree itself.
fn check_entries(moves: &[MoveFormat]) -> Result<(), NormalizeError> {
    for mf in moves {
        if mf.move_.is_none() && mf.special.is_none() {
            return Err(NormalizeError::NoMoveOrSpecial);
        }
        for fork in mf.forks.iter().flatten() {
            check_entries(fork)?;
        }
    }
    Ok(())
}

// A fork replaces a move of the main line, so its plies carry the same numbers and the
// parser gave them colours from the same parity. Inverting has to reach them too, or
// `normalize_moves` walks into a fork whose colours disagree with the position and
// `normalize_move` rejects the whole record. KIF's 反則勝ち was given its side by the
// same parity, so it is inverted along with the moves.
pub(crate) fn invert_colors(moves: &mut [MoveFormat]) {
    for mf in moves {
        if let Some(mmf) = &mut mf.move_ {
            mmf.color = match mmf.color {
                Color::Black => Color::White,
                Color::White => Color::Black,
            };
        }
        mf.special = mf.special.map(|special| match special {
            MoveSpecial::SpecialIllegalActionBlack => MoveSpecial::SpecialIllegalActionWhite,
            MoveSpecial::SpecialIllegalActionWhite => MoveSpecial::SpecialIllegalActionBlack,
            special => special,
        });
        if let Some(forks) = &mut mf.forks {
            for fork in forks.iter_mut() {
                invert_colors(fork);
            }
        }
    }
}

// The side to move in `initial`: White in a handicap, and whoever a board names. A
// position that does not convert is taken as Black to move, like a missing one;
// `normalize` rejects it anyway.
pub(crate) fn initial_side_to_move(initial: Option<&Initial>) -> Color {
    initial
        .and_then(|initial| PartialPosition::try_from(initial).ok())
        .map_or(Color::Black, |pos| pos.side_to_move().into())
}

fn normalize_initial(jkf: &mut JsonKifuFormat) -> Result<(), NormalizeError> {
    if let Some(initial) = &mut jkf.initial {
        // JKF reads `data` only with the OTHER preset, and so does the replay. Next to any
        // other preset a board is not the position, so it is neither folded nor checked.
        if initial.preset != Preset::PresetOther {
            return Ok(());
        }
        // A board that is exactly a handicap layout is stored by its name, whichever entry
        // point it came through.
        if let Some(data) = initial.data {
            if let Some(preset) = PRESETS.into_iter().find(|&p| layout(p) == Some(data)) {
                *initial = Initial { preset, data: None };
            }
        }
        // A KIF hand count is spelled in kanji and the writer stops at 九十九, which is
        // also the most SFEN carries in one token. Reject a larger count here, where it
        // can be reported: the writers return `fmt::Result` and cannot say which piece
        // they could not write.
        if let Some(data) = &initial.data {
            for hand in &data.hands {
                for (kind, num) in [
                    (Kind::FU, hand.FU),
                    (Kind::KY, hand.KY),
                    (Kind::KE, hand.KE),
                    (Kind::GI, hand.GI),
                    (Kind::KI, hand.KI),
                    (Kind::KA, hand.KA),
                    (Kind::HI, hand.HI),
                ] {
                    if num > MAX_HAND_COUNT {
                        return Err(NormalizeError::HandCountOutOfRange(kind, num));
                    }
                }
            }
        }
    }
    Ok(())
}

// Check if the `from` is retrievable from the position
fn calculate_from(
    mmf: &MoveMoveFormat,
    pos: &PartialPosition,
    to: shogi_core::Square,
) -> Result<Option<PlaceFormat>, NormalizeError> {
    let color = pos.side_to_move();
    let bb = LiteLegalityChecker.normal_to_candidates(
        pos,
        to,
        shogi_core::Piece::new(shogi_core::PieceKind::from(mmf.piece), color),
    );
    let mut froms = bb.into_iter().collect::<Vec<_>>();
    match bb.count() {
        0 => Ok(None),
        1 => Ok(bb.into_iter().next().map(|sq| PlaceFormat {
            x: sq.file(),
            y: sq.rank(),
        })),
        2.. => {
            let relative = mmf
                .relative
                .ok_or_else(|| NormalizeError::AmbiguousMoveFrom(froms.clone()))?;
            let (to_rel_file, to_rel_rank) = (to.relative_file(color), to.relative_rank(color));
            match relative {
                Relative::L => froms.retain(|sq| sq.relative_file(color) > to_rel_file),
                Relative::C => froms.retain(|sq| sq.file() == to.file()),
                Relative::R => froms.retain(|sq| sq.relative_file(color) < to_rel_file),
                Relative::U => froms.retain(|sq| sq.relative_rank(color) > to_rel_rank),
                Relative::M => froms.retain(|sq| sq.rank() == to.rank()),
                Relative::D => froms.retain(|sq| sq.relative_rank(color) < to_rel_rank),
                Relative::LU => froms.retain(|sq| {
                    sq.relative_file(color) > to_rel_file && sq.relative_rank(color) > to_rel_rank
                }),
                Relative::LM => froms
                    .retain(|sq| sq.relative_file(color) > to_rel_file && sq.rank() == to.rank()),
                Relative::LD => froms.retain(|sq| {
                    sq.relative_file(color) > to_rel_file && sq.relative_rank(color) < to_rel_rank
                }),
                Relative::RU => froms.retain(|sq| {
                    sq.relative_file(color) < to_rel_file && sq.relative_rank(color) > to_rel_rank
                }),
                Relative::RM => froms
                    .retain(|sq| sq.relative_file(color) < to_rel_file && sq.rank() == to.rank()),
                Relative::RD => froms.retain(|sq| {
                    sq.relative_file(color) < to_rel_file && sq.relative_rank(color) < to_rel_rank
                }),
                _ => {}
            };
            if froms.len() == 1 {
                Ok(Some(PlaceFormat {
                    x: froms[0].file(),
                    y: froms[0].rank(),
                }))
            } else {
                Err(NormalizeError::AmbiguousMoveFrom(froms))
            }
        }
    }
}

fn normalize_move(
    mmf: &mut MoveMoveFormat,
    pos: &PartialPosition,
    from_given: bool,
) -> Result<(), NormalizeError> {
    if matches!(
        (mmf.color, pos.side_to_move()),
        (Color::Black, shogi_core::Color::White) | (Color::White, shogi_core::Color::Black)
    ) {
        return Err(NormalizeError::InvalidColor);
    }
    if mmf.same.is_some() {
        mmf.to = pos
            .last_move()
            .map(|mv| PlaceFormat {
                x: mv.to().file(),
                y: mv.to().rank(),
            })
            .ok_or(NormalizeError::NoLastMove)?;
    }
    let to = match shogi_core::Square::try_from(&mmf.to) {
        Ok(to) => to,
        Err(err) => return Err(NormalizeError::Convert(err.to_string())),
    };
    // KI2 gives no from-square, so one is worked out from the position. A drop has none
    // either, and is one as it stands when the record gives every other move its
    // from-square, as JKF does, or when it is marked 打, as json-kifu-format and this
    // function write a drop that a piece on the board could also make.
    if mmf.from.is_none() && !from_given && mmf.relative != Some(Relative::H) {
        mmf.from = calculate_from(mmf, pos, to)?;
    }
    // `(0, 0)` is how a from-square says "drop": the parsers write it for `打` and for
    // CSA's `00`, and `JsonKifuFormat::try_from(&Position)` for every drop. JKF spells a
    // drop as no from-square at all, so the marker is unwound here. Any other square off
    // the board is malformed input, and taking it for a drop would make up a move the
    // record does not contain.
    if mmf.from == Some(PlaceFormat { x: 0, y: 0 }) {
        mmf.from = None;
    }
    if let Some(pf) = &mmf.from {
        let from = match shogi_core::Square::try_from(pf) {
            Ok(from) => from,
            Err(err) => return Err(NormalizeError::Convert(err.to_string())),
        };
        // Retrieve piece
        let piece = match pos.piece_at(from) {
            Some(piece) => piece,
            None => return Err(NormalizeError::NoPieceAt(from)),
        };
        let from_piece_kind = piece.piece_kind();
        let to_piece_kind = {
            let pk = shogi_core::PieceKind::from(mmf.piece);
            if mmf.promote.unwrap_or_default() {
                pk.promote().unwrap_or(pk)
            } else {
                pk
            }
        };
        mmf.piece = pk2k(from_piece_kind);
        // Set same?
        mmf.same = if pos
            .last_move()
            .map(|last| to == last.to())
            .unwrap_or_default()
        {
            Some(true)
        } else {
            None
        };
        // Set promote?
        mmf.promote = if from_piece_kind.promote().is_some()
            && (from.relative_rank(pos.side_to_move()) <= 3
                || to.relative_rank(pos.side_to_move()) <= 3)
        {
            Some(from_piece_kind != to_piece_kind)
        } else {
            None
        };
        // Set capture?
        mmf.capture = pos.piece_at(to).map(|p| pk2k(p.piece_kind()));
    } else {
        // A drop puts an unpromoted piece on an empty square, so it neither promotes nor
        // captures. One marked as promoting (KIF `成打`, KI2 `打成`) was played as a plain
        // drop, and the CSA writer then wrote a promoted piece dropped.
        if mmf.promote == Some(true) {
            return Err(NormalizeError::PromotingDrop(to));
        }
        mmf.promote = None;
        mmf.capture = None;
    }
    let mv = match shogi_core::Move::try_from(&*mmf) {
        Ok(mv) => mv,
        Err(err) => return Err(NormalizeError::Convert(err.to_string())),
    };
    // Set relative?
    if mmf.relative.is_none() {
        if let Some(display) = display_single_move_kansuji(pos, mv) {
            // A move that could promote ends in `成` or `不成`, written after the
            // disambiguation, so that comes off first.
            let rest = display
                .strip_suffix("不成")
                .or_else(|| display.strip_suffix('成'))
                .unwrap_or(&display);
            let mut tail = rest.chars().rev();
            mmf.relative = match (tail.next(), tail.next()) {
                (Some('左'), _) => Some(Relative::L),
                (Some('直'), _) => Some(Relative::C),
                (Some('右'), _) => Some(Relative::R),
                (Some('上'), Some('左')) => Some(Relative::LU),
                (Some('上'), Some('右')) => Some(Relative::RU),
                (Some('上'), _) => Some(Relative::U),
                (Some('引'), Some('左')) => Some(Relative::LD),
                (Some('引'), Some('右')) => Some(Relative::RD),
                (Some('引'), _) => Some(Relative::D),
                (Some('寄'), Some('左')) => Some(Relative::LM),
                (Some('寄'), Some('右')) => Some(Relative::RM),
                (Some('寄'), _) => Some(Relative::M),
                (Some('打'), _) => Some(Relative::H),
                _ => None,
            };
        }
    }
    Ok(())
}

fn normalize_moves(
    moves: &mut [MoveFormat],
    mut pos: PartialPosition,
    mut totals: [TimeFormat; 2],
    from_given: bool,
) -> Result<(), NormalizeError> {
    for mf in moves {
        // Normalize forks
        if let Some(forks) = &mut mf.forks {
            for v in forks.iter_mut() {
                normalize_moves(v, pos.clone(), totals, from_given)?;
            }
        }
        // Calculate total time
        if let Some(time) = &mut mf.time {
            totals[pos.side_to_move().array_index()] =
                add_timeformat(&totals[pos.side_to_move().array_index()], &time.now);
            time.total = totals[pos.side_to_move().array_index()];
        }
        if let Some(mmf) = &mut mf.move_ {
            normalize_move(mmf, &pos, from_given)?;
            let mv = match shogi_core::Move::try_from(&*mmf) {
                Ok(mv) => mv,
                Err(err) => return Err(NormalizeError::Convert(err.to_string())),
            };
            pos.make_move(mv)
                .ok_or(NormalizeError::MakeMoveFailed(mv))?;
        } else {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_moves_empty() {
        let pos = PartialPosition::startpos();
        assert!(normalize_moves(&mut [], pos, [TimeFormat::default(); 2], false).is_ok());
    }

    #[test]
    fn normalize_moves_invalid_color() {
        {
            let pos = PartialPosition::startpos();
            assert!(
                normalize_moves(
                    &mut [MoveFormat {
                        move_: Some(MoveMoveFormat {
                            color: Color::Black,
                            piece: Kind::FU,
                            from: Some(PlaceFormat { x: 7, y: 7 }),
                            to: PlaceFormat { x: 7, y: 6 },
                            promote: None,
                            capture: None,
                            relative: None,
                            same: None,
                        }),
                        ..Default::default()
                    }],
                    pos,
                    [TimeFormat::default(); 2],
                    false
                )
                .is_ok(),
                "normalize should succeed"
            );
        }
        {
            let pos = PartialPosition::startpos();
            assert!(
                normalize_moves(
                    &mut [MoveFormat {
                        move_: Some(MoveMoveFormat {
                            color: Color::White, // invalid color
                            piece: Kind::FU,
                            from: Some(PlaceFormat { x: 7, y: 7 }),
                            to: PlaceFormat { x: 7, y: 6 },
                            promote: None,
                            capture: None,
                            relative: None,
                            same: None,
                        }),
                        ..Default::default()
                    }],
                    pos,
                    [TimeFormat::default(); 2],
                    false
                )
                .is_err(),
                "normalize should fail"
            );
        }
    }
}
