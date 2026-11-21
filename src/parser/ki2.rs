use super::kakinoki::{
    color_prefix, move_comment_line, move_to, parse_without_moves, piece_kind, special_word,
};
use crate::jkf::*;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{digit1, line_ending, multispace0, not_line_ending, space0};
use nom::combinator::{all_consuming, map, opt, value};
use nom::error::VerboseError;
use nom::multi::{many0, many1};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::IResult;

fn single_move(input: &str) -> IResult<&str, MoveFormat, VerboseError<&str>> {
    map(
        tuple((
            alt((value(Color::Black, tag("▲")), value(Color::White, tag("△")))),
            move_to,
            piece_kind,
            // The disambiguation comes before `成` / `不成`, as in the official notation
            // and in this crate's writer: `△５七桂左成`.
            opt(alt((
                value(Relative::LU, tag("左上")),
                value(Relative::LM, tag("左寄")),
                value(Relative::LD, tag("左引")),
                value(Relative::RU, tag("右上")),
                value(Relative::RM, tag("右寄")),
                value(Relative::RD, tag("右引")),
                value(Relative::L, tag("左")),
                value(Relative::C, tag("直")),
                value(Relative::R, tag("右")),
                value(Relative::U, tag("上")),
                value(Relative::M, tag("寄")),
                value(Relative::D, tag("引")),
                value(Relative::H, tag("打")),
            ))),
            opt(alt((value(true, tag("成")), value(false, tag("不成"))))),
            preceded(
                tuple((space0, opt(line_ending))),
                opt(many1(move_comment_line)),
            ),
        )),
        |(c, to, kind, relative, promote, comments)| {
            // To disambiguate `Normal` move or `Drop` move, "打" will be parsed as `Some(PlaceFormat { x: 0, y: 0 })`
            let from = if relative == Some(Relative::H) {
                Some(PlaceFormat { x: 0, y: 0 })
            } else {
                None
            };
            MoveFormat {
                move_: Some(MoveMoveFormat {
                    color: c,
                    from,
                    to: to.unwrap_or_default(), // Might be (0, 0) if it's the same place as previous
                    piece: kind,
                    same: if to.is_none() { Some(true) } else { None },
                    promote,
                    capture: None,
                    relative,
                }),
                comments,
                ..Default::default()
            }
        },
    )(input)
}

// Shogi DB2 writes how the game ended as one more move, in KIF's words (`… ▲７八玉 △投了`),
// before the `まで…` line. Its mark need not be the side to move's (`△５五歩 △投了`), so
// only the word is read.
fn result_token(input: &str) -> IResult<&str, MoveSpecial, VerboseError<&str>> {
    preceded(alt((tag("▲"), tag("△"))), special_word)(input)
}

// KI2 has no move line for how a game ended: the `まで…` line that closes the record says
// it. A wording not listed here still closes the record, since the moves are all there,
// but it gives no special.
//
// Each wording names the side Kifu for Windows and tsshogi name for it, and the name is
// read only for 反則勝ち, where JKF needs to know whose foul it was. json-kifu-format and
// tsshogi read the others without it too.
fn result_line(input: &str) -> IResult<&str, Option<MoveSpecial>, VerboseError<&str>> {
    delimited(
        tuple((tag("まで"), digit1, tag("手"))),
        opt(alt((
            value(
                MoveSpecial::SpecialTimeUp,
                tuple((tag("で時間切れにより"), color_prefix, tag("の勝ち"))),
            ),
            // The winner is named, so the foul was the other side's.
            map(
                delimited(tag("で"), color_prefix, tag("の反則勝ち")),
                |winner| match winner {
                    Color::Black => MoveSpecial::SpecialIllegalActionWhite,
                    Color::White => MoveSpecial::SpecialIllegalActionBlack,
                },
            ),
            value(
                MoveSpecial::SpecialIllegalMove,
                tuple((tag("で"), color_prefix, tag("の反則負け"))),
            ),
            value(
                MoveSpecial::SpecialKachi,
                tuple((tag("で"), color_prefix, tag("の入玉勝ち"))),
            ),
            value(
                MoveSpecial::SpecialToryo,
                tuple((tag("で"), color_prefix, tag("の勝ち"))),
            ),
            value(MoveSpecial::SpecialChudan, tag("で中断")),
            value(MoveSpecial::SpecialJishogi, tag("で持将棋")),
            value(MoveSpecial::SpecialSennichite, tag("で千日手")),
            value(MoveSpecial::SpecialFuzumi, tag("で不詰")),
            value(MoveSpecial::SpecialTsumi, pair(opt(tag("で")), tag("詰"))),
        ))),
        pair(not_line_ending, line_ending),
    )(input)
}

fn moves(input: &str) -> IResult<&str, Vec<MoveFormat>, VerboseError<&str>> {
    map(
        tuple((
            preceded(many0(line_ending), opt(many1(move_comment_line))),
            // A blank line between moves does not end the game.
            many0(preceded(multispace0, single_move)),
            opt(preceded(multispace0, result_token)),
            opt(preceded(multispace0, result_line)),
            // tsshogi writes a special's comments after the `まで…` line, and this crate's
            // writer leaves a blank line before them when the last move has its own.
            opt(many1(preceded(multispace0, move_comment_line))),
        )),
        |(comments, mut v, token, made, trailing)| {
            // The `まで…` line names the winner, so it is read over the token. The token's
            // 反則勝ち is a win for the side to move, on the foul of whoever moved last.
            let special = made.flatten().or_else(|| match token? {
                MoveSpecial::SpecialIllegalActionBlack | MoveSpecial::SpecialIllegalActionWhite => {
                    v.last()?.move_.map(|mmf| match mmf.color {
                        Color::Black => MoveSpecial::SpecialIllegalActionBlack,
                        Color::White => MoveSpecial::SpecialIllegalActionWhite,
                    })
                }
                special => Some(special),
            });
            if let Some(special) = special {
                v.push(MoveFormat {
                    special: Some(special),
                    ..Default::default()
                });
            }
            let mut moves = [
                vec![MoveFormat {
                    comments,
                    ..Default::default()
                }],
                v,
            ]
            .concat();
            // They belong to the last entry: the special, or else the last move.
            if let (Some(trailing), Some(last)) = (trailing, moves.last_mut()) {
                last.comments.get_or_insert_with(Vec::new).extend(trailing);
            }
            moves
        },
    )(input)
}

pub(crate) fn parse(input: &str) -> IResult<&str, JsonKifuFormat, VerboseError<&str>> {
    // All of the input has to be read, as for KIF: `many0(single_move)` used to stop at
    // anything it could not take, and the rest of the file was dropped without a word.
    all_consuming(terminated(
        map(pair(parse_without_moves, moves), |(mut jkf, moves)| {
            jkf.moves.extend(moves);
            jkf
        }),
        multispace0,
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn parse_empty() {
        assert_eq!(
            Ok((
                "",
                JsonKifuFormat {
                    header: HashMap::new(),
                    initial: Some(Initial {
                        preset: Preset::PresetHirate,
                        data: None,
                    }),
                    moves: vec![MoveFormat::default()],
                }
            )),
            parse("")
        );
    }

    #[test]
    fn parse_single_move() {
        assert_eq!(
            Ok((
                "",
                MoveFormat {
                    move_: Some(MoveMoveFormat {
                        color: Color::White,
                        from: None,
                        to: PlaceFormat { x: 0, y: 0 },
                        piece: Kind::FU,
                        same: Some(true),
                        promote: None,
                        capture: None,
                        relative: None,
                    }),
                    ..Default::default()
                }
            )),
            single_move("△同　歩")
        );
        assert_eq!(
            Ok((
                "",
                MoveFormat {
                    move_: Some(MoveMoveFormat {
                        color: Color::White,
                        from: None,
                        to: PlaceFormat { x: 4, y: 7 },
                        piece: Kind::GI,
                        same: None,
                        promote: Some(false),
                        capture: None,
                        relative: None,
                    }),
                    ..Default::default()
                }
            )),
            single_move("△４七銀不成")
        );
        assert_eq!(
            Ok((
                "",
                MoveFormat {
                    move_: Some(MoveMoveFormat {
                        color: Color::White,
                        from: None,
                        to: PlaceFormat { x: 9, y: 9 },
                        piece: Kind::KA,
                        same: None,
                        promote: Some(true),
                        capture: None,
                        relative: None,
                    }),
                    ..Default::default()
                }
            )),
            single_move("△９九角成")
        );
        assert_eq!(
            Ok((
                "",
                MoveFormat {
                    move_: Some(MoveMoveFormat {
                        color: Color::Black,
                        from: None,
                        to: PlaceFormat { x: 8, y: 2 },
                        piece: Kind::KI,
                        same: None,
                        promote: None,
                        capture: None,
                        relative: Some(Relative::U),
                    }),
                    ..Default::default()
                }
            )),
            single_move("▲８二金上")
        );
        assert_eq!(
            Ok((
                "",
                MoveFormat {
                    move_: Some(MoveMoveFormat {
                        color: Color::Black,
                        from: None,
                        to: PlaceFormat { x: 8, y: 2 },
                        piece: Kind::KI,
                        same: None,
                        promote: None,
                        capture: None,
                        relative: Some(Relative::M),
                    }),
                    ..Default::default()
                }
            )),
            single_move("▲８二金寄")
        );
        assert_eq!(
            Ok((
                "",
                MoveFormat {
                    move_: Some(MoveMoveFormat {
                        color: Color::Black,
                        from: Some(PlaceFormat { x: 0, y: 0 }),
                        to: PlaceFormat { x: 8, y: 2 },
                        piece: Kind::KI,
                        same: None,
                        promote: None,
                        capture: None,
                        relative: Some(Relative::H),
                    }),
                    ..Default::default()
                }
            )),
            single_move("▲８二金打")
        );
        assert_eq!(
            Ok((
                "",
                MoveFormat {
                    move_: Some(MoveMoveFormat {
                        color: Color::Black,
                        from: None,
                        to: PlaceFormat { x: 3, y: 3 },
                        piece: Kind::GI,
                        same: None,
                        promote: Some(true),
                        capture: None,
                        relative: Some(Relative::L),
                    }),
                    ..Default::default()
                }
            )),
            single_move("▲３三銀左成")
        );
    }

    #[test]
    fn parse_moves() {
        assert_eq!(
            Ok((
                "",
                vec![MoveFormat {
                    comments: Some(vec![String::from("comment")]),
                    ..Default::default()
                }]
            )),
            moves("*comment\n")
        );
        assert_eq!(
            Ok((
                "",
                vec![
                    MoveFormat::default(),
                    MoveFormat {
                        move_: Some(MoveMoveFormat {
                            color: Color::Black,
                            from: None,
                            to: PlaceFormat { x: 6, y: 8 },
                            piece: Kind::GI,
                            same: None,
                            promote: None,
                            capture: None,
                            relative: None,
                        }),
                        ..Default::default()
                    },
                    MoveFormat {
                        move_: Some(MoveMoveFormat {
                            color: Color::White,
                            from: None,
                            to: PlaceFormat { x: 3, y: 4 },
                            piece: Kind::FU,
                            same: None,
                            promote: None,
                            capture: None,
                            relative: None,
                        }),
                        ..Default::default()
                    },
                    MoveFormat {
                        move_: Some(MoveMoveFormat {
                            color: Color::Black,
                            from: None,
                            to: PlaceFormat { x: 5, y: 6 },
                            piece: Kind::FU,
                            same: None,
                            promote: None,
                            capture: None,
                            relative: None,
                        }),
                        ..Default::default()
                    }
                ]
            )),
            moves("▲６八銀 △３四歩 ▲５六歩")
        )
    }

    #[test]
    fn parse_moves_with_comments() {
        assert_eq!(
            Ok((
                "",
                vec![
                    MoveFormat::default(),
                    MoveFormat {
                        move_: Some(MoveMoveFormat {
                            color: Color::White,
                            from: None,
                            to: PlaceFormat { x: 7, y: 4 },
                            piece: Kind::FU,
                            same: None,
                            promote: None,
                            capture: None,
                            relative: None,
                        }),
                        comments: Some(vec![
                            String::from("-2732"),
                            String::from("△３二銀(31)▲７六歩(77)△８四歩(83)"),
                        ]),
                        ..Default::default()
                    },
                    MoveFormat {
                        move_: Some(MoveMoveFormat {
                            color: Color::Black,
                            from: None,
                            to: PlaceFormat { x: 7, y: 6 },
                            piece: Kind::FU,
                            same: None,
                            promote: None,
                            capture: None,
                            relative: None,
                        }),
                        comments: Some(vec![
                            String::from("2733"),
                            String::from("▲７六歩(77)△６二銀(71)▲２六歩(27)"),
                        ]),
                        ..Default::default()
                    },
                ]
            )),
            moves(
                &r#"
△７四歩
*-2732
*△３二銀(31)▲７六歩(77)△８四歩(83)
▲７六歩
*2733
*▲７六歩(77)△６二銀(71)▲２六歩(27)
"#[1..]
            )
        );
    }
}
