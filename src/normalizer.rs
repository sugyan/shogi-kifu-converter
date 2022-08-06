use crate::error::{CoreConvertError, NormalizerError};
use crate::jkf::*;
use shogi_core::{LegalityChecker, PartialPosition, PieceKind};
use shogi_legality_lite::LiteLegalityChecker;
use shogi_official_kifu::display_single_move_kansuji;

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

const STATE_HIRATE: StateFormat = StateFormat {
    color: Color::Black,
    board: HIRATE_BOARD,
    hands: [Hand::empty(); 2],
};

const STATE_KY: StateFormat = {
    let mut board = HIRATE_BOARD;
    board[0][0] = Piece::empty();
    StateFormat {
        color: Color::White,
        board,
        hands: [Hand::empty(); 2],
    }
};

const STATE_KA: StateFormat = {
    let mut board = HIRATE_BOARD;
    board[1][1] = Piece::empty();
    StateFormat {
        color: Color::White,
        board,
        hands: [Hand::empty(); 2],
    }
};

const STATE_HI: StateFormat = {
    let mut board = HIRATE_BOARD;
    board[7][1] = Piece::empty();
    StateFormat {
        color: Color::White,
        board,
        hands: [Hand::empty(); 2],
    }
};

const STATE_HIKY: StateFormat = {
    let mut board = HIRATE_BOARD;
    board[0][0] = Piece::empty();
    board[7][1] = Piece::empty();
    StateFormat {
        color: Color::White,
        board,
        hands: [Hand::empty(); 2],
    }
};

const STATE_2: StateFormat = {
    let mut board = HIRATE_BOARD;
    board[1][1] = Piece::empty();
    board[7][1] = Piece::empty();
    StateFormat {
        color: Color::White,
        board,
        hands: [Hand::empty(); 2],
    }
};

const STATE_4: StateFormat = {
    let mut board = STATE_2.board;
    board[0][0] = Piece::empty();
    board[8][0] = Piece::empty();
    StateFormat {
        color: Color::White,
        board,
        hands: [Hand::empty(); 2],
    }
};

const STATE_6: StateFormat = {
    let mut board = STATE_4.board;
    board[1][0] = Piece::empty();
    board[7][0] = Piece::empty();
    StateFormat {
        color: Color::White,
        board,
        hands: [Hand::empty(); 2],
    }
};

const STATE_8: StateFormat = {
    let mut board = STATE_6.board;
    board[2][0] = Piece::empty();
    board[6][0] = Piece::empty();
    StateFormat {
        color: Color::White,
        board,
        hands: [Hand::empty(); 2],
    }
};

const STATE_10: StateFormat = {
    let mut board = STATE_8.board;
    board[3][0] = Piece::empty();
    board[5][0] = Piece::empty();
    StateFormat {
        color: Color::White,
        board,
        hands: [Hand::empty(); 2],
    }
};

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
    pub(crate) const fn empty() -> Self {
        Hand {
            FU: 0,
            KY: 0,
            KE: 0,
            GI: 0,
            KI: 0,
            KA: 0,
            HI: 0,
        }
    }
    pub(crate) fn increment(&mut self, kind: Kind) {
        match kind {
            Kind::FU => self.FU += 1,
            Kind::KY => self.KY += 1,
            Kind::KE => self.KE += 1,
            Kind::GI => self.GI += 1,
            Kind::KI => self.KI += 1,
            Kind::KA => self.KA += 1,
            Kind::HI => self.HI += 1,
            _ => unreachable!(),
        }
    }
    pub(crate) fn decrement(&mut self, kind: Kind) {
        match kind {
            Kind::FU => self.FU -= 1,
            Kind::KY => self.KY -= 1,
            Kind::KE => self.KE -= 1,
            Kind::GI => self.GI -= 1,
            Kind::KI => self.KI -= 1,
            Kind::KA => self.KA -= 1,
            Kind::HI => self.HI -= 1,
            _ => unreachable!(),
        }
    }
}

fn add_timeformat(lhs: &TimeFormat, rhs: &TimeFormat) -> TimeFormat {
    let s = (lhs.h.unwrap_or_default() + rhs.h.unwrap_or_default()) as u64 * 3600
        + (lhs.m + rhs.m) as u64 * 60
        + (lhs.s + rhs.s) as u64;
    let m = (s / 60) % 60;
    let h = s / 3600;
    TimeFormat {
        h: Some(h as u8),
        m: m as u8,
        s: (s % 60) as u8,
    }
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
    pub fn normalize(&mut self) -> Result<(), NormalizerError> {
        normalize_initial(self)?;
        let pos = if let Some(initial) = &self.initial {
            initial.try_into()?
        } else {
            PartialPosition::startpos()
        };
        normalize_moves(&mut self.moves[1..], pos, [TimeFormat::default(); 2])?;
        Ok(())
    }
}

fn normalize_initial(jkf: &mut JsonKifuFormat) -> Result<(), NormalizerError> {
    if let Some(initial) = &mut jkf.initial {
        *initial = match initial.data {
            Some(STATE_HIRATE) => Initial {
                preset: Preset::PresetHirate,
                data: None,
            },
            Some(STATE_KY) => Initial {
                preset: Preset::PresetKY,
                data: None,
            },
            Some(STATE_KA) => Initial {
                preset: Preset::PresetKA,
                data: None,
            },
            Some(STATE_HI) => Initial {
                preset: Preset::PresetHI,
                data: None,
            },
            Some(STATE_HIKY) => Initial {
                preset: Preset::PresetHIKY,
                data: None,
            },
            Some(STATE_2) => Initial {
                preset: Preset::Preset2,
                data: None,
            },
            Some(STATE_4) => Initial {
                preset: Preset::Preset4,
                data: None,
            },
            Some(STATE_6) => Initial {
                preset: Preset::Preset6,
                data: None,
            },
            Some(STATE_8) => Initial {
                preset: Preset::Preset8,
                data: None,
            },
            Some(STATE_10) => Initial {
                preset: Preset::Preset10,
                data: None,
            },
            _ => *initial,
        };
    }
    Ok(())
}

// Check if the `from` is retrievable from the position
fn calculate_from(
    mmf: &MoveMoveFormat,
    pos: &PartialPosition,
    to: shogi_core::Square,
) -> Result<Option<PlaceFormat>, NormalizerError> {
    let color = pos.side_to_move();
    let bb = LiteLegalityChecker.normal_to_candidates(
        pos,
        to,
        shogi_core::Piece::new(PieceKind::from(mmf.piece), color),
    );
    let mut froms = bb.into_iter().collect::<Vec<_>>();
    match bb.count() {
        0 => Ok(None),
        1 => Ok(bb.into_iter().next().map(|sq| PlaceFormat {
            x: sq.file() as u8,
            y: sq.rank() as u8,
        })),
        2.. => {
            let relative = mmf
                .relative
                .ok_or_else(|| NormalizerError::AmbiguousMoveFrom(froms.clone()))?;
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
                    x: froms[0].file() as u8,
                    y: froms[0].rank() as u8,
                }))
            } else {
                Err(NormalizerError::AmbiguousMoveFrom(froms))
            }
        }
    }
}

fn normalize_move(mmf: &mut MoveMoveFormat, pos: &PartialPosition) -> Result<(), NormalizerError> {
    mmf.color = match pos.side_to_move() {
        shogi_core::Color::Black => Color::Black,
        shogi_core::Color::White => Color::White,
    };
    if mmf.same.is_some() {
        mmf.to = pos
            .last_move()
            .map(|mv| PlaceFormat {
                x: mv.to().file() as u8,
                y: mv.to().rank() as u8,
            })
            .ok_or(NormalizerError::NoLastMove)?;
    }
    let to = shogi_core::Square::try_from(&mmf.to)?;
    if mmf.from.is_none() {
        mmf.from = calculate_from(mmf, pos, to)?;
    }
    if let Some(pf) = &mmf.from {
        if let Ok(from) = pf.try_into() {
            // Retrieve piece
            let piece = pos
                .piece_at(from)
                .ok_or(NormalizerError::MoveInconsistent("no piece to move found"))?;
            let from_piece_kind = piece.piece_kind();
            let to_piece_kind = if mmf.promote.unwrap_or_default() {
                let pk = PieceKind::from(mmf.piece);
                pk.promote().unwrap_or(pk)
            } else {
                mmf.piece.into()
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
            mmf.from = None;
        }
    }
    let mv = (&*mmf).try_into()?;
    // Set relative?
    if mmf.relative.is_none() {
        if let Some(mut display) = display_single_move_kansuji(pos, mv) {
            mmf.relative = match (display.pop(), display.pop()) {
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
) -> Result<(), NormalizerError> {
    for mf in moves {
        // Normalize forks
        if let Some(forks) = &mut mf.forks {
            for v in forks.iter_mut() {
                normalize_moves(v, pos.clone(), totals)?;
            }
        }
        // Calculate total time
        if let Some(time) = &mut mf.time {
            totals[pos.side_to_move().array_index()] =
                add_timeformat(&totals[pos.side_to_move().array_index()], &time.now);
            time.total = totals[pos.side_to_move().array_index()];
        }
        if let Some(mmf) = &mut mf.move_ {
            normalize_move(mmf, &pos)?;
            let mv = (&*mmf).try_into()?;
            pos.make_move(mv).ok_or(NormalizerError::CoreConvert(
                CoreConvertError::InvalidMove(mv),
            ))?;
        } else {
            break;
        }
    }
    Ok(())
}
