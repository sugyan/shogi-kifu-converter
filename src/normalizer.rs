use crate::jkf::{Board, Color, Hands, Initial, JsonKifFormat, Kind, Piece, Preset, StateFormat};

impl StateFormat {
    pub fn hirate() -> Self {
        Self {
            color: Color::Black,
            board: Board::hirate(),
            hands: Hands::default(),
        }
    }
}

impl Board {
    pub fn hirate() -> Self {
        Self([
            [
                Piece {
                    color: Some(Color::White),
                    kind: Some(Kind::KY),
                },
                Piece::default(),
                Piece {
                    color: Some(Color::White),
                    kind: Some(Kind::FU),
                },
                Piece::default(),
                Piece::default(),
                Piece::default(),
                Piece {
                    color: Some(Color::Black),
                    kind: Some(Kind::FU),
                },
                Piece::default(),
                Piece {
                    color: Some(Color::Black),
                    kind: Some(Kind::KY),
                },
            ],
            [
                Piece {
                    color: Some(Color::White),
                    kind: Some(Kind::KE),
                },
                Piece {
                    color: Some(Color::White),
                    kind: Some(Kind::KA),
                },
                Piece {
                    color: Some(Color::White),
                    kind: Some(Kind::FU),
                },
                Piece::default(),
                Piece::default(),
                Piece::default(),
                Piece {
                    color: Some(Color::Black),
                    kind: Some(Kind::FU),
                },
                Piece {
                    color: Some(Color::Black),
                    kind: Some(Kind::HI),
                },
                Piece {
                    color: Some(Color::Black),
                    kind: Some(Kind::KE),
                },
            ],
            [
                Piece {
                    color: Some(Color::White),
                    kind: Some(Kind::GI),
                },
                Piece::default(),
                Piece {
                    color: Some(Color::White),
                    kind: Some(Kind::FU),
                },
                Piece::default(),
                Piece::default(),
                Piece::default(),
                Piece {
                    color: Some(Color::Black),
                    kind: Some(Kind::FU),
                },
                Piece::default(),
                Piece {
                    color: Some(Color::Black),
                    kind: Some(Kind::GI),
                },
            ],
            [
                Piece {
                    color: Some(Color::White),
                    kind: Some(Kind::KI),
                },
                Piece::default(),
                Piece {
                    color: Some(Color::White),
                    kind: Some(Kind::FU),
                },
                Piece::default(),
                Piece::default(),
                Piece::default(),
                Piece {
                    color: Some(Color::Black),
                    kind: Some(Kind::FU),
                },
                Piece::default(),
                Piece {
                    color: Some(Color::Black),
                    kind: Some(Kind::KI),
                },
            ],
            [
                Piece {
                    color: Some(Color::White),
                    kind: Some(Kind::OU),
                },
                Piece::default(),
                Piece {
                    color: Some(Color::White),
                    kind: Some(Kind::FU),
                },
                Piece::default(),
                Piece::default(),
                Piece::default(),
                Piece {
                    color: Some(Color::Black),
                    kind: Some(Kind::FU),
                },
                Piece::default(),
                Piece {
                    color: Some(Color::Black),
                    kind: Some(Kind::OU),
                },
            ],
            [
                Piece {
                    color: Some(Color::White),
                    kind: Some(Kind::KI),
                },
                Piece::default(),
                Piece {
                    color: Some(Color::White),
                    kind: Some(Kind::FU),
                },
                Piece::default(),
                Piece::default(),
                Piece::default(),
                Piece {
                    color: Some(Color::Black),
                    kind: Some(Kind::FU),
                },
                Piece::default(),
                Piece {
                    color: Some(Color::Black),
                    kind: Some(Kind::KI),
                },
            ],
            [
                Piece {
                    color: Some(Color::White),
                    kind: Some(Kind::GI),
                },
                Piece::default(),
                Piece {
                    color: Some(Color::White),
                    kind: Some(Kind::FU),
                },
                Piece::default(),
                Piece::default(),
                Piece::default(),
                Piece {
                    color: Some(Color::Black),
                    kind: Some(Kind::FU),
                },
                Piece::default(),
                Piece {
                    color: Some(Color::Black),
                    kind: Some(Kind::GI),
                },
            ],
            [
                Piece {
                    color: Some(Color::White),
                    kind: Some(Kind::KE),
                },
                Piece {
                    color: Some(Color::White),
                    kind: Some(Kind::HI),
                },
                Piece {
                    color: Some(Color::White),
                    kind: Some(Kind::FU),
                },
                Piece::default(),
                Piece::default(),
                Piece::default(),
                Piece {
                    color: Some(Color::Black),
                    kind: Some(Kind::FU),
                },
                Piece {
                    color: Some(Color::Black),
                    kind: Some(Kind::KA),
                },
                Piece {
                    color: Some(Color::Black),
                    kind: Some(Kind::KE),
                },
            ],
            [
                Piece {
                    color: Some(Color::White),
                    kind: Some(Kind::KY),
                },
                Piece::default(),
                Piece {
                    color: Some(Color::White),
                    kind: Some(Kind::FU),
                },
                Piece::default(),
                Piece::default(),
                Piece::default(),
                Piece {
                    color: Some(Color::Black),
                    kind: Some(Kind::FU),
                },
                Piece::default(),
                Piece {
                    color: Some(Color::Black),
                    kind: Some(Kind::KY),
                },
            ],
        ])
    }
}

pub(crate) fn normalize(jkf: &mut JsonKifFormat) {
    if let Some(initial) = jkf.initial {
        if initial.data == Some(StateFormat::hirate()) {
            jkf.initial = Some(Initial {
                preset: Preset::PresetHirate,
                data: None,
            });
        }
    }
}
