use crate::jkf::*;
use crate::normalizer::{normalize, HIRATE_BOARD};
use csa::{GameRecord, Position};
use std::collections::HashMap;

impl From<GameRecord> for JsonKifFormat {
    fn from(record: GameRecord) -> Self {
        // Header
        let mut header = HashMap::new();
        if let Some(s) = record.black_player {
            header.insert(String::from("先手"), s);
        }
        if let Some(s) = record.white_player {
            header.insert(String::from("後手"), s);
        }
        if let Some(s) = record.event {
            header.insert(String::from("棋戦"), s);
        }
        if let Some(s) = record.site {
            header.insert(String::from("場所"), s);
        }
        if let Some(t) = record.start_time {
            header.insert(String::from("開始日時"), t.to_string());
        }
        if let Some(t) = record.end_time {
            header.insert(String::from("終了日時"), t.to_string());
        }
        if let Some(t) = record.time_limit {
            header.insert(String::from("持ち時間"), t.to_string());
        }
        if let Some(s) = record.opening {
            header.insert(String::from("戦型"), s);
        }
        // Initial
        let initial = Some(Initial::from(record.start_pos));
        // Moves
        let mut moves = vec![MoveFormat::default()];
        for m in record.moves {
            moves.push(MoveFormat::from(m));
        }
        // Create JsonKifFormat, and normalize it
        let mut jkf = Self {
            header,
            initial,
            moves,
        };
        normalize(&mut jkf);
        jkf
    }
}

impl From<Position> for Initial {
    fn from(mut pos: Position) -> Self {
        let mut pieces = [
            (Kind::FU, 18),
            (Kind::KY, 4),
            (Kind::KE, 4),
            (Kind::GI, 4),
            (Kind::KI, 4),
            (Kind::KA, 2),
            (Kind::HI, 2),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();
        // split to hands' and board's
        let mut hand_pieces = Vec::new();
        pos.add_pieces.retain(|&(c, sq, pt)| {
            if sq.file == 0 && sq.rank == 0 {
                hand_pieces.push((c, pt));
                false
            } else {
                true
            }
        });
        // Color
        let color = Color::from(pos.side_to_move);
        // Board
        let board = if let Some(grid) = pos.bulk {
            // 一括表現
            let mut b = [[Piece::empty(); 9]; 9];
            for (i, row) in grid.iter().enumerate() {
                for (j, &col) in row.iter().enumerate() {
                    b[8 - j][i] = if let Some((c, pt)) = col {
                        Piece::from((c, pt))
                    } else {
                        Piece::empty()
                    };
                }
            }
            b
        } else if pos.add_pieces.is_empty() {
            // 平手初期配置と駒落ち
            let mut b = HIRATE_BOARD;
            for &(sq, _) in &pos.drop_pieces {
                b[sq.file as usize - 1][sq.rank as usize - 1] = Piece::empty()
            }
            b
        } else {
            // 駒別単独表現
            let mut b = [[Piece::empty(); 9]; 9];
            for &(c, sq, pt) in &pos.add_pieces {
                b[sq.file as usize - 1][sq.rank as usize - 1] = Piece::from((c, pt));
            }
            b
        };
        for row in &board {
            for col in row {
                if let Some(unpromoted) = col.kind.map(Kind::unpromoted) {
                    if unpromoted != Kind::OU {
                        *pieces.get_mut(&unpromoted).unwrap() -= 1;
                    }
                }
            }
        }
        // Hands
        let mut hands = [Hand::default(); 2];
        for &(c, pt) in &hand_pieces {
            if let Ok(kind) = Kind::try_from(pt) {
                hands[Color::from(c) as usize].add(kind);
            } else {
                for (&kind, &num) in &pieces {
                    (0..num).for_each(|_| hands[Color::from(c) as usize].add(kind));
                }
            }
        }
        Self {
            preset: Preset::PresetOther,
            data: Some(StateFormat {
                color,
                board,
                hands,
            }),
        }
    }
}

impl From<csa::MoveRecord> for MoveFormat {
    fn from(m: csa::MoveRecord) -> Self {
        match m.action {
            csa::Action::Move(c, from, to, pt) => MoveFormat {
                move_: Some(MoveMoveFormat {
                    color: Color::from(c),
                    piece: Kind::try_from(pt).expect("invalid piece"),
                    from: PlaceFormat::try_from(from).ok(),
                    to: PlaceFormat::try_from(to).expect("invalid place `to`"),
                    same: None,
                    promote: None,
                    capture: None,
                    relative: None,
                }),
                comments: None,
                special: None,
            },
            action => MoveFormat {
                move_: None,
                comments: None,
                special: Some(String::from(&action.to_string()[1..])),
            },
        }
    }
}

impl From<(csa::Color, csa::PieceType)> for Piece {
    fn from((c, pt): (csa::Color, csa::PieceType)) -> Self {
        Piece {
            color: Some(Color::from(c)),
            kind: Kind::try_from(pt).ok(),
        }
    }
}

impl From<csa::Color> for Color {
    fn from(c: csa::Color) -> Self {
        match c {
            csa::Color::Black => Color::Black,
            csa::Color::White => Color::White,
        }
    }
}

impl TryFrom<csa::Square> for PlaceFormat {
    type Error = ();

    fn try_from(sq: csa::Square) -> Result<Self, Self::Error> {
        if sq.file == 0 && sq.rank == 0 {
            Err(())
        } else {
            Ok(PlaceFormat {
                x: sq.file,
                y: sq.rank,
            })
        }
    }
}

impl TryFrom<csa::PieceType> for Kind {
    type Error = ();

    fn try_from(pt: csa::PieceType) -> Result<Self, Self::Error> {
        match pt {
            csa::PieceType::Pawn => Ok(Kind::FU),
            csa::PieceType::Lance => Ok(Kind::KY),
            csa::PieceType::Knight => Ok(Kind::KE),
            csa::PieceType::Silver => Ok(Kind::GI),
            csa::PieceType::Gold => Ok(Kind::KI),
            csa::PieceType::Bishop => Ok(Kind::KA),
            csa::PieceType::Rook => Ok(Kind::HI),
            csa::PieceType::King => Ok(Kind::OU),
            csa::PieceType::ProPawn => Ok(Kind::TO),
            csa::PieceType::ProLance => Ok(Kind::NY),
            csa::PieceType::ProKnight => Ok(Kind::NK),
            csa::PieceType::ProSilver => Ok(Kind::NG),
            csa::PieceType::Horse => Ok(Kind::UM),
            csa::PieceType::Dragon => Ok(Kind::RY),
            csa::PieceType::All => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::jkf::*;
    use std::ffi::OsStr;
    use std::fs::File;
    use std::io::{BufReader, Read};
    use std::path::Path;

    #[test]
    fn csa_to_jkf() {
        let dir = Path::new("data/tests/csa");
        for entry in dir.read_dir().expect("failed to read dir") {
            // Parse and convert CSA to JKF
            let entry = entry.expect("failed to read entry");
            let mut path = entry.path();
            if path.extension() != Some(OsStr::new("csa")) {
                continue;
            }
            let mut file = File::open(&path).expect("failed to open file");
            let mut buf = String::new();
            file.read_to_string(&mut buf).expect("failed to read file");
            let record = csa::parse_csa(&buf).expect("failed to parse csa");
            let jkf = JsonKifFormat::from(record);

            // Load exptected JSON
            let mut json_filename = path
                .file_name()
                .expect("failed to get file name")
                .to_os_string();
            json_filename.push(".json");
            path.set_file_name(json_filename);
            let file = File::open(&path).expect("failed to open file");
            let mut expected = serde_json::from_reader::<_, JsonKifFormat>(BufReader::new(file))
                .expect("failed to parse json");
            // Rename header key
            if let Some(v) = expected.header.remove("OPENING") {
                expected.header.insert(String::from("戦型"), v);
            }
            // Remove all move comments (they cannot be restored from csa...)
            expected.moves.iter_mut().for_each(|m| m.comments = None);

            assert_eq!(expected, jkf, "different from expected: {}", path.display());
        }
    }
}
