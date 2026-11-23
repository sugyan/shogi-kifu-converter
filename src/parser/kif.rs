use super::kakinoki::{
    comment_line, move_comment_line, move_to, not_move_line, parse_without_moves, piece_kind,
    special_word,
};
use crate::jkf::*;
use crate::normalizer::{initial_side_to_move, invert_colors};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{digit1, line_ending, multispace0, not_line_ending, space0};
use nom::combinator::{all_consuming, map, map_opt, map_res, opt, value};
use nom::error::VerboseError;
use nom::multi::{many0, many1};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use nom::IResult;

fn move_from(input: &str) -> IResult<&str, Option<PlaceFormat>, VerboseError<&str>> {
    alt((
        // To disambiguate `Normal` move or `Drop` move, "打" will be parsed as `Some(PlaceFormat { x: 0, y: 0 })`
        value(Some(PlaceFormat { x: 0, y: 0 }), tag("打")),
        map(
            delimited(tag("("), map_res(digit1, str::parse), tag(")")),
            |d: u8| {
                Some(PlaceFormat {
                    x: d / 10,
                    y: d % 10,
                })
            },
        ),
    ))(input)
}

fn move_special(input: &str) -> IResult<&str, MoveFormat, VerboseError<&str>> {
    // 反則勝ち's side depends on the ply, so `move_line` settles it.
    map(special_word, |special| MoveFormat {
        special: Some(special),
        ..Default::default()
    })(input)
}

fn move_move(input: &str) -> IResult<&str, MoveFormat, VerboseError<&str>> {
    map(
        // `不成` is written by some producers even though the KIF spec says to leave a
        // declined promotion unmarked. Without an arm for it the move line fails to
        // parse and `many1` silently drops the rest of the game.
        tuple((
            move_to,
            piece_kind,
            opt(alt((value(true, tag("成")), value(false, tag("不成"))))),
            move_from,
        )),
        |(to, kind, promote, from)| {
            MoveFormat {
                move_: Some(MoveMoveFormat {
                    color: Color::Black, // To be replaced
                    from,
                    to: to.unwrap_or_default(), // Might be (0, 0) if it's the same place as previous
                    piece: kind,
                    same: if to.is_none() { Some(true) } else { None },
                    promote,
                    capture: None,
                    relative: None,
                }),
                ..Default::default()
            }
        },
    )(input)
}

fn move_time_format(input: &str) -> IResult<&str, TimeFormat, VerboseError<&str>> {
    alt((
        map(
            tuple((
                terminated(map_res(digit1, str::parse), tag(":")),
                terminated(map_res(digit1, str::parse), tag(":")),
                map_res(digit1, str::parse),
            )),
            |(h, m, s)| TimeFormat { h: Some(h), m, s },
        ),
        map(
            tuple((
                terminated(map_res(digit1, str::parse), tag(":")),
                map_res(digit1, str::parse),
            )),
            |(m, s)| TimeFormat { h: None, m, s },
        ),
    ))(input)
}

fn move_time(input: &str) -> IResult<&str, Time, VerboseError<&str>> {
    delimited(
        tag("("),
        map(
            separated_pair(
                delimited(space0, move_time_format, space0),
                tag("/"),
                delimited(space0, move_time_format, space0),
            ),
            |(now, total)| Time { now, total },
        ),
        tag(")"),
    )(input)
}

fn move_line(input: &str) -> IResult<&str, (usize, MoveFormat), VerboseError<&str>> {
    map(
        delimited(
            space0,
            tuple((
                preceded(space0, map_res(digit1, str::parse)),
                preceded(space0, alt((move_special, move_move))),
                preceded(space0, opt(move_time)),
            )),
            preceded(not_line_ending, line_ending),
        ),
        |(i, mut mf, time)| {
            if let Some(mmf) = &mut mf.move_ {
                mmf.color = [Color::White, Color::Black][i % 2];
            }
            // 反則勝ち: the side to move by the same parity wins, so the foul is the other
            // side's.
            if matches!(
                mf.special,
                Some(
                    MoveSpecial::SpecialIllegalActionBlack | MoveSpecial::SpecialIllegalActionWhite
                )
            ) {
                mf.special = Some(
                    [
                        MoveSpecial::SpecialIllegalActionBlack,
                        MoveSpecial::SpecialIllegalActionWhite,
                    ][i % 2],
                );
            }
            mf.time = time;
            (i, mf)
        },
    )(input)
}

fn move_with_comments(input: &str) -> IResult<&str, (usize, MoveFormat), VerboseError<&str>> {
    map(
        pair(move_line, many0(preceded(multispace0, move_comment_line))),
        |((i, mf), comments)| {
            (
                i,
                MoveFormat {
                    comments: Some(comments).filter(|v| !v.is_empty()),
                    ..mf
                },
            )
        },
    )(input)
}

fn moves_with_index(input: &str) -> IResult<&str, (usize, Vec<MoveFormat>), VerboseError<&str>> {
    map(
        // Blank lines mean nothing between the moves, their comments and the line after
        // them (`まで…`, a 変化 header). They used to: with LF, `not_move_line` read a blank
        // line and the next one as one line, and the move or comment on it was lost; with
        // CRLF it read a blank line on its own, in the place of `まで…`.
        terminated(
            many1(preceded(multispace0, move_with_comments)),
            opt(preceded(multispace0, not_move_line)),
        ),
        |v| (v[0].0, v.into_iter().map(|(_, mf)| mf).collect()),
    )(input)
}

// Also returns the number of the first move line, if there is one.
fn main_moves(input: &str) -> IResult<&str, (Option<usize>, Vec<MoveFormat>), VerboseError<&str>> {
    map(
        pair(opt(many1(move_comment_line)), opt(moves_with_index)),
        |(comments, o)| {
            let (first, moves) = match o {
                Some((i, v)) => (Some(i), v),
                None => (None, Vec::new()),
            };
            let initial = MoveFormat {
                comments,
                ..Default::default()
            };
            (first, [vec![initial], moves].concat())
        },
    )(input)
}

// Also returns the number of the first move line of the main line, if there is one.
fn entire_moves(
    input: &str,
) -> IResult<&str, (Option<usize>, Vec<MoveFormat>), VerboseError<&str>> {
    // The ply a fork branches at is the number printed in the file, so it can name a
    // ply that does not exist in the line it attaches to. Returns `None` in that case
    // instead of indexing out of bounds.
    fn attach(moves: &mut [MoveFormat], index: usize, fork: Vec<MoveFormat>) -> Option<()> {
        let mf = moves.get_mut(index)?;
        if let Some(v) = &mut mf.forks {
            v.push(fork);
        } else {
            mf.forks = Some(vec![fork]);
        }
        Some(())
    }

    fn merge_forks(
        (mut moves, mut forks): (Vec<MoveFormat>, Vec<(usize, Vec<MoveFormat>)>),
    ) -> Option<Vec<MoveFormat>> {
        let mut stack = Vec::new();
        while let Some(fork) = forks.pop() {
            stack.push(fork);
            if let Some((i, last)) = forks.last_mut() {
                while stack.last().map_or(false, |(j, _)| j >= i) {
                    if let Some((j, fork)) = stack.pop() {
                        attach(last, j.checked_sub(*i)?, fork)?;
                    }
                }
            }
        }
        while let Some((i, fork)) = stack.pop() {
            attach(&mut moves, i, fork)?;
        }
        Some(moves)
    }

    map_opt(
        pair(
            // Exactly one line, the 手数 header: `not_move_line` does not exclude `&`,
            // so a greedy `many0` would eat a bookmark comment that belongs to
            // `main_moves` and so to `moves[0]`.
            preceded(opt(not_move_line), main_moves),
            many0(preceded(many0(not_move_line), moves_with_index)),
        ),
        |((first, moves), forks)| Some((first, merge_forks((moves, forks))?)),
    )(input)
}

pub(crate) fn parse(input: &str) -> IResult<&str, JsonKifuFormat, VerboseError<&str>> {
    // All of the input has to be read. A line no rule takes used to end the moves there,
    // and everything after it was dropped without a word. Only trailing whitespace and
    // `#` comment lines may be left over: IS-SHOGI ends a record with one.
    all_consuming(terminated(
        map(
            pair(parse_without_moves, entire_moves),
            |(mut jkf, (first, moves))| {
                jkf.moves.extend(moves);
                // The plies are coloured by the parity of their numbers, Black at the odd
                // ones. `normalize` inverts them when White is to move and the move at ply
                // 1 is coloured Black, which it is when the record numbers that ply odd. A
                // record that opens with a special has no move there to go by, so it is
                // done here from the number: the side of a 反則勝ち, and the forks at that
                // ply.
                if initial_side_to_move(jkf.initial.as_ref()) == Color::White
                    && first.is_some_and(|i| i % 2 == 1)
                    && jkf.moves.get(1).is_some_and(|mf| mf.move_.is_none())
                {
                    invert_colors(jkf.moves.get_mut(1..).unwrap_or_default());
                }
                jkf
            },
        ),
        pair(many0(preceded(multispace0, comment_line)), multispace0),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_move_time_format() {
        assert!(move_time_format("").is_err());
        assert_eq!(
            Ok((
                "",
                TimeFormat {
                    h: None,
                    m: 0,
                    s: 16
                }
            )),
            move_time_format("0:16")
        );
        assert_eq!(
            Ok((
                "",
                TimeFormat {
                    h: Some(0),
                    m: 0,
                    s: 16
                }
            )),
            move_time_format("00:00:16")
        );
    }

    #[test]
    fn parse_move_move() {
        assert!(move_move("").is_err());
        assert_eq!(
            Ok((
                "",
                MoveFormat {
                    move_: Some(MoveMoveFormat {
                        color: Color::Black,
                        from: Some(PlaceFormat { x: 7, y: 7 }),
                        to: PlaceFormat { x: 7, y: 6 },
                        piece: Kind::FU,
                        same: None,
                        promote: None,
                        capture: None,
                        relative: None,
                    }),
                    ..Default::default()
                }
            )),
            move_move("７六歩(77)")
        );
        assert_eq!(
            Ok((
                "",
                MoveFormat {
                    move_: Some(MoveMoveFormat {
                        color: Color::Black,
                        from: Some(PlaceFormat { x: 3, y: 1 }),
                        to: PlaceFormat { x: 4, y: 2 },
                        piece: Kind::KA,
                        same: None,
                        promote: Some(true),
                        capture: None,
                        relative: None,
                    }),
                    ..Default::default()
                }
            )),
            move_move("４二角成(31)")
        );
    }

    #[test]
    fn parse_move_line() {
        assert!(move_line("").is_err());
        assert_eq!(
            Ok((
                "",
                (
                    1,
                    MoveFormat {
                        move_: Some(MoveMoveFormat {
                            color: Color::Black,
                            from: Some(PlaceFormat { x: 7, y: 7 }),
                            to: PlaceFormat { x: 7, y: 6 },
                            piece: Kind::FU,
                            same: None,
                            promote: None,
                            capture: None,
                            relative: None,
                        }),
                        comments: None,
                        time: Some(Time {
                            now: TimeFormat {
                                h: None,
                                m: 0,
                                s: 16
                            },
                            total: TimeFormat {
                                h: Some(0),
                                m: 0,
                                s: 16
                            }
                        }),
                        special: None,
                        forks: None,
                    }
                )
            )),
            move_line("1 ７六歩(77) ( 0:16/00:00:16)\n")
        );
        assert_eq!(
            Ok((
                "",
                (
                    3,
                    MoveFormat {
                        move_: None,
                        comments: None,
                        time: Some(Time {
                            now: TimeFormat {
                                h: None,
                                m: 0,
                                s: 3
                            },
                            total: TimeFormat {
                                h: Some(0),
                                m: 0,
                                s: 19
                            }
                        }),
                        special: Some(MoveSpecial::SpecialChudan),
                        forks: None,
                    }
                )
            )),
            move_line("3 中断 ( 0:03/ 0:00:19)\n")
        );
        assert_eq!(
            Ok((
                "",
                (
                    1,
                    MoveFormat {
                        move_: Some(MoveMoveFormat {
                            color: Color::Black,
                            from: Some(PlaceFormat { x: 6, y: 9 }),
                            to: PlaceFormat { x: 7, y: 8 },
                            piece: Kind::KI,
                            same: None,
                            promote: None,
                            capture: None,
                            relative: None,
                        }),
                        time: Some(Time {
                            now: TimeFormat {
                                h: None,
                                m: 0,
                                s: 1
                            },
                            total: TimeFormat {
                                h: Some(0),
                                m: 0,
                                s: 1
                            }
                        }),
                        ..Default::default()
                    }
                )
            )),
            move_line("   1 ７八金(69)    (00:01 / 00:00:01)\n")
        )
    }

    #[test]
    fn parse_main_moves() {
        assert_eq!(
            Ok((
                "",
                vec![
                    MoveFormat::default(),
                    MoveFormat {
                        move_: Some(MoveMoveFormat {
                            color: Color::Black,
                            from: Some(PlaceFormat { x: 7, y: 7 }),
                            to: PlaceFormat { x: 7, y: 6 },
                            piece: Kind::FU,
                            same: None,
                            promote: None,
                            capture: None,
                            relative: None,
                        }),
                        comments: None,
                        time: Some(Time {
                            now: TimeFormat {
                                h: None,
                                m: 0,
                                s: 16
                            },
                            total: TimeFormat {
                                h: Some(0),
                                m: 0,
                                s: 16
                            }
                        }),
                        special: None,
                        forks: None,
                    },
                    MoveFormat {
                        move_: Some(MoveMoveFormat {
                            color: Color::White,
                            from: Some(PlaceFormat { x: 3, y: 3 }),
                            to: PlaceFormat { x: 3, y: 4 },
                            piece: Kind::FU,
                            same: None,
                            promote: None,
                            capture: None,
                            relative: None,
                        }),
                        comments: None,
                        time: Some(Time {
                            now: TimeFormat {
                                h: None,
                                m: 0,
                                s: 0
                            },
                            total: TimeFormat {
                                h: Some(0),
                                m: 0,
                                s: 0
                            }
                        }),
                        special: None,
                        forks: None,
                    },
                    MoveFormat {
                        move_: None,
                        comments: None,
                        time: Some(Time {
                            now: TimeFormat {
                                h: None,
                                m: 0,
                                s: 3
                            },
                            total: TimeFormat {
                                h: Some(0),
                                m: 0,
                                s: 19
                            }
                        }),
                        special: Some(MoveSpecial::SpecialChudan),
                        forks: None,
                    },
                ]
            )),
            main_moves(
                &r#"
1 ７六歩(77) ( 0:16/00:00:16)
2 ３四歩(33) ( 0:00/00:00:00)
3 中断 ( 0:03/ 0:00:19)
"#[1..],
            )
            .map(|(rest, (first, moves))| {
                assert_eq!(first, Some(1));
                (rest, moves)
            })
        );
        assert_eq!(
            Ok((
                "",
                vec![
                    MoveFormat {
                        comments: Some(vec![String::from("開始局面のコメント")]),
                        ..Default::default()
                    },
                    MoveFormat {
                        move_: Some(MoveMoveFormat {
                            color: Color::Black,
                            from: Some(PlaceFormat { x: 2, y: 7 }),
                            to: PlaceFormat { x: 2, y: 6 },
                            piece: Kind::FU,
                            same: None,
                            promote: None,
                            capture: None,
                            relative: None,
                        }),
                        time: Some(Time {
                            now: TimeFormat {
                                s: 1,
                                ..Default::default()
                            },
                            total: TimeFormat {
                                h: Some(0),
                                m: 0,
                                s: 1
                            }
                        }),
                        ..Default::default()
                    },
                ]
            )),
            main_moves(
                &r#"
*開始局面のコメント
  1 ２六歩(27) ( 0:01/00:00:01)
"#[1..]
            )
            .map(|(rest, (first, moves))| {
                assert_eq!(first, Some(1));
                (rest, moves)
            })
        )
    }

    #[test]
    fn parse_entire_moves() {
        let input = &r#"
手数----指手---------消費時間--
   1 ７六歩(77)    (00:00 / 00:00:00)
   2 ８四歩(83)    (00:00 / 00:00:00)
   3 ６八銀(79)    (00:00 / 00:00:00)
   4 ３二金(41)    (00:00 / 00:00:00)
   5 ２六歩(27)    (00:00 / 00:00:00)
   6 ８五歩(84)    (00:00 / 00:00:00)
   7 ７七角(88)    (00:00 / 00:00:00)
   8 ３四歩(33)    (00:00 / 00:00:00)
   9 ７八金(69)    (00:00 / 00:00:00)
  10 ７七角成(22)  (00:00 / 00:00:00)
  11 同　銀(68)    (00:00 / 00:00:00)
  12 ２二銀(31)    (00:00 / 00:00:00)

変化：10手
  10 ３三角(22)    (00:00 / 00:00:00)
  11 ６九玉(59)    (00:00 / 00:00:00)
  12 ４二銀(31)    (00:00 / 00:00:00)
  13 ３六歩(37)    (00:00 / 00:00:00)
  14 ７七角成(33)  (00:00 / 00:00:00)

変化：5手
   5 ７七角(88)    (00:00 / 00:00:00)
   6 ３四歩(33)    (00:00 / 00:00:00)
   7 ４八銀(39)    (00:00 / 00:00:00)
   8 ６二銀(71)    (00:00 / 00:00:00)
   9 ３六歩(37)    (00:00 / 00:00:00)
  10 ８五歩(84)    (00:00 / 00:00:00)
  11 ７八金(69)    (00:00 / 00:00:00)
  12 ７四歩(73)    (00:00 / 00:00:00)

変化：9手
   9 １六歩(17)    (00:00 / 00:00:00)
  10 １四歩(13)    (00:00 / 00:00:00)
  11 ２六歩(27)    (00:00 / 00:00:00)
  12 ４二銀(31)    (00:00 / 00:00:00)
  13 ２二角成(77)  (00:00 / 00:00:00)
  14 同　金(32)    (00:00 / 00:00:00)
  15 ７七銀(68)    (00:00 / 00:00:00)
"#[1..];
        let ret = entire_moves(input);
        let (rest, (first, moves)) = ret.expect("failed to parse");
        assert!(rest.is_empty());
        assert_eq!(first, Some(1));
        assert_eq!(13, moves.len());
        for (i, m) in moves.iter().enumerate() {
            match i {
                5 => {
                    let forks = m.forks.as_ref().expect("no forks");
                    assert_eq!(1, forks.len());
                    assert_eq!(8, forks[0].len());
                    assert!(forks[0].iter().any(|m| m.forks.is_none()))
                }
                10 => {
                    let forks = m.forks.as_ref().expect("no forks");
                    assert_eq!(1, forks.len());
                    assert_eq!(5, forks[0].len());
                    assert!(forks[0].iter().all(|m| m.forks.is_none()))
                }
                _ => assert!(m.forks.is_none()),
            }
        }
    }
}
