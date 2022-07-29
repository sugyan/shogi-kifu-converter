use crate::jkf::*;
use crate::shogi_core::CoreConvertError;
use shogi_core::{LegalityChecker, PartialPosition, PieceKind};
use shogi_legality_lite::LiteLegalityChecker;
use std::cmp::Ordering;
use std::ops::AddAssign;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NormalizerError {
    #[error("Convert Error: {0}")]
    CoreConvert(#[from] CoreConvertError),
    #[error("Invalid move: {0}")]
    MoveInconsistent(&'static str),
    #[error("Invalid move: {0:?}")]
    MoveError(MoveMoveFormat),
}

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

impl Piece {
    pub(crate) const fn empty() -> Self {
        Self {
            color: None,
            kind: None,
        }
    }
}

impl Kind {
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
    #[allow(dead_code)]
    pub(crate) fn add(&mut self, kind: Kind) {
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
    #[allow(dead_code)]
    pub(crate) fn sub(&mut self, kind: Kind) {
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

impl AddAssign for TimeFormat {
    fn add_assign(&mut self, rhs: Self) {
        let s = (self.h.unwrap_or_default() + rhs.h.unwrap_or_default()) as u64 * 3600
            + (self.m + rhs.m) as u64 * 60
            + (self.s + rhs.s) as u64;
        let m = (s / 60) % 60;
        let h = s / 3600;
        self.h = Some(h as u8);
        self.m = m as u8;
        self.s = (s % 60) as u8;
    }
}

impl From<shogi_core::PieceKind> for Kind {
    fn from(pk: shogi_core::PieceKind) -> Self {
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
}

pub(crate) fn normalize(jkf: &mut JsonKifFormat) -> Result<(), NormalizerError> {
    normalize_initial(jkf)?;
    let pos = if let Some(initial) = jkf.initial {
        initial.try_into()?
    } else {
        PartialPosition::startpos()
    };
    normalize_moves(&mut jkf.moves[1..], pos, [TimeFormat::default(); 2])?;
    Ok(())
}

fn mmf2move(mmf: MoveMoveFormat) -> Result<shogi_core::Move, NormalizerError> {
    if let Some(from) = mmf.from {
        Ok(shogi_core::Move::Normal {
            from: from.try_into()?,
            to: mmf.to.try_into()?,
            promote: mmf.promote.unwrap_or_default(),
        })
    } else {
        Ok(shogi_core::Move::Drop {
            piece: mmf.into(),
            to: mmf.to.try_into()?,
        })
    }
}

fn normalize_initial(jkf: &mut JsonKifFormat) -> Result<(), NormalizerError> {
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
            _ => *initial,
        };
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
        if let Some(forks) = mf.forks.as_mut() {
            for v in forks.iter_mut() {
                normalize_moves(v, pos.clone(), totals)?;
            }
        }
        // Calculate total time
        if let Some(time) = &mut mf.time {
            totals[pos.side_to_move().array_index()] += time.now;
            time.total = totals[pos.side_to_move().array_index()];
        }
        if let Some(mmf) = &mut mf.move_ {
            mmf.color = match pos.side_to_move() {
                shogi_core::Color::Black => Color::Black,
                shogi_core::Color::White => Color::White,
            };
            if mmf.same.is_some() {
                mmf.to = pos
                    .last_move()
                    .map(|mv| PlaceFormat {
                        x: mv.to().file(),
                        y: mv.to().rank(),
                    })
                    .ok_or(CoreConvertError::InvalidPlace((0, 0)))?
            }
            let to = shogi_core::Square::try_from(mmf.to)?;
            if let Some(from) = mmf.from {
                let from = from.try_into()?;
                // Retrieve piece
                let piece = pos
                    .piece_at(from)
                    .ok_or(NormalizerError::MoveInconsistent("no piece to move found"))?;
                let from_piece_kind = piece.piece_kind();
                let to_piece_kind = if mmf.promote.is_some() {
                    let pk = PieceKind::from(mmf.piece);
                    pk.promote().unwrap_or(pk)
                } else {
                    mmf.piece.into()
                };
                mmf.piece = from_piece_kind.into();
                // Set same?
                if pos
                    .last_move()
                    .map(|last| to == last.to())
                    .unwrap_or_default()
                {
                    mmf.same = Some(true);
                }
                // Set promote?
                if from_piece_kind.promote().is_some()
                    && (from.relative_rank(pos.side_to_move()) <= 3
                        || to.relative_rank(pos.side_to_move()) <= 3)
                {
                    mmf.promote = Some(from_piece_kind != to_piece_kind)
                }
                // Set capture?
                if let Some(p) = pos.piece_at(to) {
                    mmf.capture = Some(p.piece_kind().into());
                }
                // Set relative?
                // TODO
                let candidates = LiteLegalityChecker.normal_to_candidates(&pos, to, piece);
                if candidates.count() > 1 {
                    mmf.relative = Some(
                        match from
                            .relative_file(pos.side_to_move())
                            .cmp(&to.relative_file(pos.side_to_move()))
                        {
                            Ordering::Less => Relative::R,
                            Ordering::Equal => Relative::C,
                            Ordering::Greater => Relative::L,
                        },
                    );
                }
            } else {
                // TODO
            }
            pos.make_move(mmf2move(*mmf)?)
                .ok_or(NormalizerError::MoveError(*mmf))?;
        } else {
            break;
        }
    }
    Ok(())
}
