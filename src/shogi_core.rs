mod from;
mod into;

#[cfg(test)]
mod tests {
    use crate::jkf::JsonKifuFormat;
    use shogi_core::{Move, Piece, Position, Square};
    use std::ffi::OsStr;
    use std::fs::{DirEntry, File};
    use std::io::{BufReader, Result};
    use std::path::Path;

    fn visit_dirs(dir: &Path, cb: &dyn Fn(&DirEntry) -> Result<()>) -> Result<()> {
        if dir.is_dir() {
            for entry in dir.read_dir()? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dirs(&path, cb)?;
                } else {
                    cb(&entry)?;
                }
            }
        }
        Ok(())
    }

    #[test]
    fn pos_to_pos() {
        let orig = {
            let mut pos = Position::default();
            let moves = [
                Move::Normal {
                    from: Square::SQ_7G,
                    to: Square::SQ_7F,
                    promote: false,
                },
                Move::Normal {
                    from: Square::SQ_3C,
                    to: Square::SQ_3D,
                    promote: false,
                },
                Move::Normal {
                    from: Square::SQ_8H,
                    to: Square::SQ_2B,
                    promote: true,
                },
                Move::Normal {
                    from: Square::SQ_8B,
                    to: Square::SQ_2B,
                    promote: false,
                },
                Move::Drop {
                    to: Square::SQ_1E,
                    piece: Piece::B_B,
                },
            ];
            for mv in moves {
                pos.make_move(mv).expect("failed to make move");
            }
            pos
        };
        let jkf = JsonKifuFormat::try_from(&orig).expect("failed to convert position to jkf");
        let curr = Position::try_from(&jkf).expect("failed to convert jkf to position");
        assert_eq!(orig, curr, "positions are not equal");
    }

    #[test]
    fn jkf_to_jkf() -> Result<()> {
        visit_dirs(Path::new("data/tests"), &|entry: &DirEntry| -> Result<()> {
            let path = entry.path();
            if path.extension() != Some(OsStr::new("json")) {
                return Ok(());
            }
            // TODO: https://github.com/rust-shogi-crates/shogi_official_kifu/issues/5
            if path == Path::new("data/tests/ki2/simple.json") {
                return Ok(());
            }

            let file = File::open(&path)?;
            let orig = serde_json::from_reader::<_, JsonKifuFormat>(BufReader::new(file))
                .expect("failed to parse json");
            let pos = match Position::try_from(&orig) {
                Ok(pos) => pos,
                Err(err) => panic!(
                    "failed to convert jkf to position {}: {err}",
                    path.display()
                ),
            };
            let curr = match JsonKifuFormat::try_from(&pos) {
                Ok(pos) => pos,
                Err(err) => panic!(
                    "failed to convert position to jkf {}: {err}",
                    path.display()
                ),
            };
            assert_eq!(
                orig.initial,
                curr.initial,
                "different initial from expected: {}",
                path.display()
            );
            assert_eq!(
                orig.moves
                    .iter()
                    .filter_map(|mf| mf.move_)
                    .collect::<Vec<_>>(),
                curr.moves
                    .iter()
                    .filter_map(|mf| mf.move_)
                    .collect::<Vec<_>>(),
                "different moves from expected: {}",
                path.display()
            );
            Ok(())
        })
    }
}
