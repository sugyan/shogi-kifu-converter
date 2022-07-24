use crate::jkf::{Board, Color, Hand, Hands, Initial, JsonKifFormat, Preset, StateFormat};
use crate::normalizer::normalize;
use csa::GameRecord;
use std::collections::HashMap;

impl From<GameRecord> for JsonKifFormat {
    fn from(record: GameRecord) -> Self {
        // Headers
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
        // Initial state
        let board = Board::hirate(); // TODO
        let hands = Hands([Hand::default(), Hand::default()]);
        let initial = Some(Initial {
            preset: Preset::PresetOther,
            data: Some(StateFormat {
                color: match record.start_pos.side_to_move {
                    csa::Color::Black => Color::Black,
                    csa::Color::White => Color::White,
                },
                board,
                hands,
            }),
        });
        // Moves
        let moves = Vec::new();
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

#[cfg(test)]
mod tests {
    use crate::jkf::*;
    use std::ffi::OsStr;
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;

    #[test]
    fn csa_to_jkf() {
        let dir = Path::new("data/tests/csa");
        for entry in dir.read_dir().expect("failed to read dir") {
            let entry = entry.expect("failed to read entry");
            let path = entry.path();
            if path.extension() != Some(OsStr::new("csa")) {
                continue;
            }
            let mut file = File::open(&path).expect("failed to open file");
            let mut buf = String::new();
            file.read_to_string(&mut buf).expect("failed to read file");
            let record = csa::parse_csa(&buf).expect("failed to parse csa");
            let _jkf = JsonKifFormat::from(record);
            // TODO: compare to .json files
        }
    }
}
