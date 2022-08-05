use super::kakinoki::{move_comment_line, move_to, parse_without_moves, piece_kind};
use crate::jkf::*;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{line_ending, space0};
use nom::combinator::{map, opt, value};
use nom::error::VerboseError;
use nom::multi::{many0, many1};
use nom::sequence::{pair, preceded, tuple};
use nom::IResult;

fn single_move(input: &str) -> IResult<&str, MoveFormat, VerboseError<&str>> {
    map(
        tuple((
            alt((value(Color::Black, tag("▲")), value(Color::White, tag("△")))),
            move_to,
            piece_kind,
            opt(alt((value(true, tag("成")), value(false, tag("不成"))))),
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
            preceded(
                tuple((space0, opt(line_ending))),
                opt(many1(move_comment_line)),
            ),
        )),
        |(c, to, kind, promote, relative, comments)| {
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

fn moves(input: &str) -> IResult<&str, Vec<MoveFormat>, VerboseError<&str>> {
    map(
        pair(
            preceded(many0(line_ending), opt(many1(move_comment_line))),
            many0(single_move),
        ),
        |(comments, v)| {
            [
                vec![MoveFormat {
                    comments,
                    ..Default::default()
                }],
                v,
            ]
            .concat()
        },
    )(input)
}

pub(crate) fn parse(input: &str) -> IResult<&str, JsonKifuFormat, VerboseError<&str>> {
    map(pair(parse_without_moves, moves), |(mut jkf, moves)| {
        jkf.moves.extend(moves);
        jkf
    })(input)
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
