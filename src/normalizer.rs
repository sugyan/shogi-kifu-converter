use crate::jkf::*;
use shogi_core::{LegalityChecker, PartialPosition};
use shogi_legality_lite::LiteLegalityChecker;
use std::cmp::Ordering;
use std::ops::AddAssign;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NormalizerError {
    #[error("Invalid Place: {0:?}")]
    InvalidPlace((u8, u8)),
    #[error("Invalid initial board: no data with preset `OTHER`")]
    InitialBoardNoDataWithPresetOTHER,
    #[error("Invalid initial hands: {0:?}")]
    InitialHands(Kind),
    #[error("Invalid move: {0}")]
    MoveInconsistent(&'static str),
    #[error("Invalid Move")]
    MoveError,
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

impl TryFrom<MoveMoveFormat> for shogi_core::Move {
    type Error = NormalizerError;

    fn try_from(mmf: MoveMoveFormat) -> Result<Self, Self::Error> {
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
}

impl TryFrom<PlaceFormat> for shogi_core::Square {
    type Error = NormalizerError;

    fn try_from(pf: PlaceFormat) -> Result<Self, Self::Error> {
        shogi_core::Square::new(pf.x, pf.y).ok_or(NormalizerError::InvalidPlace((pf.x, pf.y)))
    }
}

impl From<MoveMoveFormat> for shogi_core::Piece {
    fn from(mmf: MoveMoveFormat) -> Self {
        shogi_core::Piece::new(mmf.piece.into(), mmf.color.into())
    }
}

impl From<Kind> for shogi_core::PieceKind {
    fn from(piece: Kind) -> Self {
        match piece {
            Kind::FU => shogi_core::PieceKind::Pawn,
            Kind::KY => shogi_core::PieceKind::Lance,
            Kind::KE => shogi_core::PieceKind::Knight,
            Kind::GI => shogi_core::PieceKind::Silver,
            Kind::KI => shogi_core::PieceKind::Gold,
            Kind::KA => shogi_core::PieceKind::Bishop,
            Kind::HI => shogi_core::PieceKind::Rook,
            Kind::OU => shogi_core::PieceKind::King,
            Kind::TO => shogi_core::PieceKind::ProPawn,
            Kind::NY => shogi_core::PieceKind::ProLance,
            Kind::NK => shogi_core::PieceKind::ProKnight,
            Kind::NG => shogi_core::PieceKind::ProSilver,
            Kind::UM => shogi_core::PieceKind::ProBishop,
            Kind::RY => shogi_core::PieceKind::ProRook,
        }
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

impl From<Color> for shogi_core::Color {
    fn from(c: Color) -> Self {
        match c {
            Color::Black => shogi_core::Color::Black,
            Color::White => shogi_core::Color::White,
        }
    }
}

pub(crate) fn normalize(jkf: &mut JsonKifFormat) -> Result<(), NormalizerError> {
    normalize_initial(jkf)?;
    normalize_moves(jkf)?;
    Ok(())
}

fn normalize_initial(jkf: &mut JsonKifFormat) -> Result<(), NormalizerError> {
    if let Some(initial) = jkf.initial {
        jkf.initial = match initial.data {
            Some(STATE_HIRATE) => Some(Initial {
                preset: Preset::PresetHirate,
                data: None,
            }),
            Some(STATE_KY) => Some(Initial {
                preset: Preset::PresetKY,
                data: None,
            }),
            Some(STATE_KA) => Some(Initial {
                preset: Preset::PresetKA,
                data: None,
            }),
            Some(STATE_HI) => Some(Initial {
                preset: Preset::PresetHI,
                data: None,
            }),
            Some(STATE_2) => Some(Initial {
                preset: Preset::Preset2,
                data: None,
            }),
            Some(STATE_4) => Some(Initial {
                preset: Preset::Preset4,
                data: None,
            }),
            Some(STATE_6) => Some(Initial {
                preset: Preset::Preset6,
                data: None,
            }),
            _ => jkf.initial,
        };
    }
    Ok(())
}

fn normalize_moves(jkf: &mut JsonKifFormat) -> Result<(), NormalizerError> {
    let mut pos = if let Some(initial) = jkf.initial {
        match initial.preset {
            Preset::PresetHirate => PartialPosition::startpos(),
            Preset::PresetOther => {
                let data = initial
                    .data
                    .ok_or(NormalizerError::InitialBoardNoDataWithPresetOTHER)?;
                let mut pos = PartialPosition::empty();
                // Board
                for (i, v) in data.board.iter().enumerate() {
                    for (j, p) in v.iter().enumerate() {
                        let sq = PlaceFormat {
                            x: i as u8 + 1,
                            y: j as u8 + 1,
                        }
                        .try_into()?;
                        if let (Some(kind), Some(color)) = (p.kind, p.color) {
                            pos.piece_set(
                                sq,
                                Some(shogi_core::Piece::new(kind.into(), color.into())),
                            );
                        }
                    }
                }
                // Hands
                for (hand, c) in data.hands.iter().zip(shogi_core::Color::all()) {
                    let h = pos.hand_of_a_player_mut(c);
                    for (num, pk) in [
                        (hand.FU, shogi_core::PieceKind::Pawn),
                        (hand.KY, shogi_core::PieceKind::Lance),
                        (hand.KE, shogi_core::PieceKind::Knight),
                        (hand.GI, shogi_core::PieceKind::Silver),
                        (hand.KI, shogi_core::PieceKind::Gold),
                        (hand.KA, shogi_core::PieceKind::Bishop),
                        (hand.HI, shogi_core::PieceKind::Rook),
                    ] {
                        for _ in 0..num {
                            *h = h
                                .added(pk)
                                .ok_or_else(|| NormalizerError::InitialHands(pk.into()))?;
                        }
                    }
                }
                // Color
                pos.side_to_move_set(data.color.into());
                pos
            }
            _ => {
                let mut pos = PartialPosition::startpos();
                pos.side_to_move_set(shogi_core::Color::White);
                // TODO
                pos
            }
        }
    } else {
        PartialPosition::startpos()
    };

    let mut totals = [TimeFormat::default(); 2];
    let mut to_prev = None;
    for mf in jkf.moves[1..].iter_mut() {
        // Calculate total time
        if let Some(time) = &mut mf.time {
            totals[pos.side_to_move().array_index()] += time.now;
            time.total = totals[pos.side_to_move().array_index()];
        }
        if let Some(m) = &mut mf.move_ {
            if m.same.is_some() {
                m.to = to_prev.ok_or(NormalizerError::InvalidPlace((0, 0)))?
            }
            let to: shogi_core::Square = m.to.try_into()?;
            to_prev = Some(m.to);
            if let Some(from) = m.from {
                let from: shogi_core::Square = from.try_into()?;
                // Retrieve piece
                let piece = pos
                    .piece_at(from)
                    .ok_or(NormalizerError::MoveInconsistent("no piece to move found"))?;
                let from_piece_kind = piece.piece_kind();
                let to_piece_kind = m.piece.into();
                m.piece = from_piece_kind.into();
                // Set same?
                if pos
                    .last_move()
                    .map(|last| to == last.to())
                    .unwrap_or_default()
                {
                    m.same = Some(true);
                }
                // Set promote?
                if from_piece_kind.unpromote().is_none()
                    && (from.relative_rank(pos.side_to_move()) <= 3
                        || to.relative_rank(pos.side_to_move()) <= 3)
                {
                    m.promote = Some(from_piece_kind != to_piece_kind)
                }
                // Set capture?
                if let Some(p) = pos.piece_at(to) {
                    m.capture = Some(p.piece_kind().into());
                }
                // Set relative?
                // TODO
                let candidates = LiteLegalityChecker.normal_to_candidates(&pos, to, piece);
                if candidates.count() > 1 {
                    m.relative = Some(
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
            pos.make_move((*m).try_into()?)
                .ok_or(NormalizerError::MoveError)?;
        } else {
            break;
        }
    }
    Ok(())
}
