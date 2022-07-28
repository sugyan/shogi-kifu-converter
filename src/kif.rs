use crate::jkf::*;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{digit1, line_ending, none_of, not_line_ending, one_of, space0};
use nom::combinator::{map, map_res, opt, value};
use nom::error::VerboseError;
use nom::multi::{count, many0};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use nom::IResult;
use std::collections::HashMap;

fn comment_line(input: &str) -> IResult<&str, String, VerboseError<&str>> {
    map(
        delimited(tag("#"), not_line_ending, line_ending),
        String::from,
    )(input)
}

fn move_comment_line(input: &str) -> IResult<&str, String, VerboseError<&str>> {
    map(
        delimited(tag("*"), not_line_ending, line_ending),
        String::from,
    )(input)
}

fn information_line(input: &str) -> IResult<&str, (String, String), VerboseError<&str>> {
    map(
        preceded(
            many0(comment_line),
            terminated(
                separated_pair(is_not("：\r\n"), tag("："), not_line_ending),
                line_ending,
            ),
        ),
        |(k, v)| (k.into(), v.into()),
    )(input)
}

fn informations(input: &str) -> IResult<&str, HashMap<String, String>, VerboseError<&str>> {
    map(
        preceded(many0(comment_line), many0(information_line)),
        |v| v.into_iter().collect(),
    )(input)
}

fn piece_kind(input: &str) -> IResult<&str, Kind, VerboseError<&str>> {
    alt((
        value(Kind::FU, tag("歩")),
        value(Kind::KY, tag("香")),
        value(Kind::KE, tag("桂")),
        value(Kind::GI, tag("銀")),
        value(Kind::KI, tag("金")),
        value(Kind::KA, tag("角")),
        value(Kind::HI, tag("飛")),
        value(Kind::OU, tag("玉")),
        value(Kind::TO, tag("と")),
        value(Kind::NY, alt((tag("杏"), tag("成香")))),
        value(Kind::NK, alt((tag("圭"), tag("成桂")))),
        value(Kind::NG, alt((tag("全"), tag("成銀")))),
        value(Kind::UM, tag("馬")),
        value(Kind::RY, alt((tag("龍"), tag("竜")))),
    ))(input)
}

fn board_piece_color(input: &str) -> IResult<&str, Color, VerboseError<&str>> {
    alt((value(Color::Black, tag(" ")), value(Color::White, tag("v"))))(input)
}

fn board_piece(input: &str) -> IResult<&str, Piece, VerboseError<&str>> {
    alt((
        value(Piece::empty(), tag(" ・")),
        map(pair(board_piece_color, piece_kind), |(c, k)| Piece {
            color: Some(c),
            kind: Some(k),
        }),
    ))(input)
}

fn board_row(input: &str) -> IResult<&str, Vec<Piece>, VerboseError<&str>> {
    terminated(
        delimited(
            tag("|"),
            count(board_piece, 9),
            preceded(tag("|"), one_of("一二三四五六七八九")),
        ),
        line_ending,
    )(input)
}

fn board(input: &str) -> IResult<&str, [[Piece; 9]; 9], VerboseError<&str>> {
    delimited(
        tuple((
            terminated(tag("  ９ ８ ７ ６ ５ ４ ３ ２ １"), line_ending),
            terminated(tag("+---------------------------+"), line_ending),
        )),
        map(count(board_row, 9), |v| {
            let mut ret = [[Piece::empty(); 9]; 9];
            for (i, row) in v.into_iter().enumerate() {
                for (j, p) in row.into_iter().enumerate() {
                    ret[8 - j][i] = p;
                }
            }
            ret
        }),
        terminated(tag("+---------------------------+"), line_ending),
    )(input)
}

fn not_move_line(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    delimited(none_of("0123456789*"), not_line_ending, line_ending)(input)
}

fn place_x(input: &str) -> IResult<&str, u8, VerboseError<&str>> {
    alt((
        value(1, tag("１")),
        value(2, tag("２")),
        value(3, tag("３")),
        value(4, tag("４")),
        value(5, tag("５")),
        value(6, tag("６")),
        value(7, tag("７")),
        value(8, tag("８")),
        value(9, tag("９")),
    ))(input)
}

fn place_y(input: &str) -> IResult<&str, u8, VerboseError<&str>> {
    alt((
        value(1, tag("一")),
        value(2, tag("二")),
        value(3, tag("三")),
        value(4, tag("四")),
        value(5, tag("五")),
        value(6, tag("六")),
        value(7, tag("七")),
        value(8, tag("八")),
        value(9, tag("九")),
    ))(input)
}

fn move_from(input: &str) -> IResult<&str, Option<PlaceFormat>, VerboseError<&str>> {
    alt((
        value(None, tag("打")),
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

fn move_to(input: &str) -> IResult<&str, Option<PlaceFormat>, VerboseError<&str>> {
    alt((
        value(None, tag("同　")),
        map(pair(place_x, place_y), |(x, y)| Some(PlaceFormat { x, y })),
    ))(input)
}

fn move_special(input: &str) -> IResult<&str, MoveFormat, VerboseError<&str>> {
    map(
        alt((
            value(MoveSpecial::SpecialToryo, tag("投了")),
            value(MoveSpecial::SpecialChudan, tag("中断")),
            value(MoveSpecial::SpecialSennichite, tag("千日手")),
            value(MoveSpecial::SpecialTimeUp, tag("切れ負け")),
            value(MoveSpecial::SpecialIllegalMove, tag("反則負け")),
            value(MoveSpecial::SpecialJishogi, tag("持将棋")),
            value(MoveSpecial::SpecialKachi, tag("入玉勝ち")),
            value(MoveSpecial::SpecialTsumi, tag("詰み")),
        )),
        |special| MoveFormat {
            special: Some(special),
            ..Default::default()
        },
    )(input)
}

fn move_move(input: &str) -> IResult<&str, MoveFormat, VerboseError<&str>> {
    map(
        tuple((move_to, piece_kind, move_from)),
        |(to, kind, from)| {
            MoveFormat {
                move_: Some(MoveMoveFormat {
                    color: Color::Black, // To be replaced
                    from,
                    to: to.unwrap_or_default(), // Might be (0, 0) if it's the same place as previous
                    piece: kind,
                    same: if to.is_none() { Some(true) } else { None },
                    promote: None,
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
                delimited(space0, map_res(digit1, str::parse), tag(":")),
                delimited(space0, map_res(digit1, str::parse), tag(":")),
                preceded(space0, map_res(digit1, str::parse)),
            )),
            |(h, m, s)| TimeFormat { h: Some(h), m, s },
        ),
        map(
            tuple((
                delimited(space0, map_res(digit1, str::parse), tag(":")),
                preceded(space0, map_res(digit1, str::parse)),
            )),
            |(m, s)| TimeFormat { h: None, m, s },
        ),
    ))(input)
}

fn move_time(input: &str) -> IResult<&str, Time, VerboseError<&str>> {
    delimited(
        tag("("),
        map(
            separated_pair(move_time_format, tag("/"), move_time_format),
            |(now, total)| Time { now, total },
        ),
        tag(")"),
    )(input)
}

fn move_line(input: &str) -> IResult<&str, MoveFormat, VerboseError<&str>> {
    map(
        delimited(
            space0,
            tuple((
                preceded(space0, map_res(digit1, str::parse)),
                preceded(space0, alt((move_special, move_move))),
                preceded(space0, opt(move_time)),
            )),
            line_ending,
        ),
        |(i, mut mf, time): (u32, _, _)| {
            if let Some(mmf) = mf.move_.as_mut() {
                mmf.color = if i % 2 == 1 {
                    Color::Black
                } else {
                    Color::White
                };
            }
            mf.time = time;
            mf
        },
    )(input)
}

fn move_with_comments(input: &str) -> IResult<&str, MoveFormat, VerboseError<&str>> {
    map(
        pair(move_line, many0(move_comment_line)),
        |(mf, comments)| MoveFormat {
            comments: Some(comments).filter(|v| !v.is_empty()),
            ..mf
        },
    )(input)
}

fn moves(input: &str) -> IResult<&str, Vec<MoveFormat>, VerboseError<&str>> {
    map(
        pair(
            map(many0(move_comment_line), |comments| MoveFormat {
                comments: Some(comments).filter(|v| !v.is_empty()),
                ..Default::default()
            }),
            many0(move_with_comments),
        ),
        |(m0, v)| [vec![m0], v].concat(),
    )(input)
}

fn entire_moves(input: &str) -> IResult<&str, Vec<MoveFormat>, VerboseError<&str>> {
    preceded(many0(not_move_line), moves)(input)
}

pub(crate) fn parse(input: &str) -> IResult<&str, JsonKifFormat, VerboseError<&str>> {
    let mut header = HashMap::new();
    let (input, info) = informations(input)?;
    header.extend(info);
    let (input, opt_board) = opt(board)(input)?;
    let (input, info) = informations(input)?;
    header.extend(info);
    let (input, moves) = entire_moves(input)?;

    let color = Color::Black; // TODO
    let hands = [Hand::default(); 2]; // TODO
    let initial = if let Some(board) = opt_board {
        Some(Initial {
            preset: Preset::PresetOther,
            data: Some(StateFormat {
                color,
                board,
                hands,
            }),
        })
    } else {
        let preset = match header.remove(&String::from("手合割")).as_deref() {
            Some("香落ち") => Preset::PresetKY,
            Some("右香落ち") => Preset::PresetKYR,
            Some("角落ち") => Preset::PresetKA,
            Some("飛車落ち") => Preset::PresetHI,
            Some("飛香落ち") => Preset::PresetHIKY,
            Some("二枚落ち") => Preset::Preset2,
            Some("三枚落ち") => Preset::Preset3,
            Some("四枚落ち") => Preset::Preset4,
            Some("五枚落ち") => Preset::Preset5,
            Some("左五枚落ち") => Preset::Preset5L,
            Some("六枚落ち") => Preset::Preset6,
            Some("左七枚落ち") => Preset::Preset7L,
            Some("右七枚落ち") => Preset::Preset7R,
            Some("八枚落ち") => Preset::Preset8,
            Some("十枚落ち") => Preset::Preset10,
            _ => Preset::PresetHirate,
        };
        Some(Initial { preset, data: None })
    };
    Ok((
        input,
        JsonKifFormat {
            header,
            initial,
            moves,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalizer::HIRATE_BOARD;

    #[test]
    fn parse_comment_line() {
        assert!(comment_line("").is_err());
        assert!(comment_line("# comment with not line ending").is_err());
        assert!(comment_line("not comment\n").is_err());
        assert_eq!(
            Ok(("", String::from(" comment"))),
            comment_line("# comment\n")
        );
    }

    #[test]
    fn parse_information_line() {
        assert!(information_line("").is_err());
        assert!(information_line("# comment\n").is_err());
        assert!(information_line("# comment：comment\n").is_err());
        assert!(information_line("key：value with not line ending").is_err());
        assert_eq!(
            Ok(("", (String::from("key"), String::from("value")))),
            information_line("key：value\n")
        );
    }

    #[test]
    fn parse_informations() {
        assert_eq!(Ok(("", HashMap::new())), informations(""));
        assert_eq!(Ok(("", HashMap::new())), informations("# comment\n"));
        assert_eq!(
            Ok(("", HashMap::new())),
            informations("# comment：comment\n")
        );
        assert_eq!(
            Ok(("key：value with not line ending", HashMap::new())),
            informations("key：value with not line ending")
        );
        assert_eq!(
            Ok((
                "",
                [(String::from("key"), String::from("value"))]
                    .into_iter()
                    .collect::<HashMap<_, _>>()
            )),
            informations("key：value\n")
        );
    }

    #[test]
    fn parse_piece_kind() {
        assert!(piece_kind("").is_err());
        assert!(piece_kind("王").is_err());
        assert_eq!(Ok(("", Kind::FU)), piece_kind("歩"));
        assert_eq!(Ok(("", Kind::RY)), piece_kind("龍"));
        assert_eq!(Ok(("", Kind::RY)), piece_kind("竜"));
        assert_eq!(Ok(("", Kind::NY)), piece_kind("成香"));
        assert_eq!(Ok(("", Kind::NK)), piece_kind("成桂"));
        assert_eq!(Ok(("", Kind::NG)), piece_kind("成銀"));
        assert_eq!(Ok(("", Kind::NY)), piece_kind("杏"));
        assert_eq!(Ok(("", Kind::NK)), piece_kind("圭"));
        assert_eq!(Ok(("", Kind::NG)), piece_kind("全"));
    }

    #[test]
    fn parse_board_piece() {
        assert!(board_piece("").is_err());
        assert_eq!(Ok(("", Piece::empty())), board_piece(" ・"));
        assert_eq!(
            Ok((
                "",
                Piece {
                    color: Some(Color::Black),
                    kind: Some(Kind::FU)
                }
            )),
            board_piece(" 歩")
        );
        assert_eq!(
            Ok((
                "",
                Piece {
                    color: Some(Color::White),
                    kind: Some(Kind::FU)
                }
            )),
            board_piece("v歩")
        );
    }

    #[test]
    fn parse_board_row() {
        let rows = (0..9)
            .map(|i| (0..9).map(|j| HIRATE_BOARD[8 - j][i]).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        #[rustfmt::skip]
        assert_eq!(Ok(("", rows[0].clone())), board_row("|v香v桂v銀v金v玉v金v銀v桂v香|一\n"));
        #[rustfmt::skip]
        assert_eq!(Ok(("", rows[1].clone())), board_row("| ・v飛 ・ ・ ・ ・ ・v角 ・|二\n"));
        #[rustfmt::skip]
        assert_eq!(Ok(("", rows[2].clone())), board_row("|v歩v歩v歩v歩v歩v歩v歩v歩v歩|三\n"));
        #[rustfmt::skip]
        assert_eq!(Ok(("", rows[3].clone())), board_row("| ・ ・ ・ ・ ・ ・ ・ ・ ・|四\n"));
        #[rustfmt::skip]
        assert_eq!(Ok(("", rows[4].clone())), board_row("| ・ ・ ・ ・ ・ ・ ・ ・ ・|五\n"));
        #[rustfmt::skip]
        assert_eq!(Ok(("", rows[5].clone())), board_row("| ・ ・ ・ ・ ・ ・ ・ ・ ・|六\n"));
        #[rustfmt::skip]
        assert_eq!(Ok(("", rows[6].clone())), board_row("| 歩 歩 歩 歩 歩 歩 歩 歩 歩|七\n"));
        #[rustfmt::skip]
        assert_eq!(Ok(("", rows[7].clone())), board_row("| ・ 角 ・ ・ ・ ・ ・ 飛 ・|八\n"));
        #[rustfmt::skip]
        assert_eq!(Ok(("", rows[8].clone())), board_row("| 香 桂 銀 金 玉 金 銀 桂 香|九\n"));
    }

    #[test]
    fn parse_board() {
        assert_eq!(
            Ok(("", HIRATE_BOARD)),
            board(
                &r#"
  ９ ８ ７ ６ ５ ４ ３ ２ １
+---------------------------+
|v香v桂v銀v金v玉v金v銀v桂v香|一
| ・v飛 ・ ・ ・ ・ ・v角 ・|二
|v歩v歩v歩v歩v歩v歩v歩v歩v歩|三
| ・ ・ ・ ・ ・ ・ ・ ・ ・|四
| ・ ・ ・ ・ ・ ・ ・ ・ ・|五
| ・ ・ ・ ・ ・ ・ ・ ・ ・|六
| 歩 歩 歩 歩 歩 歩 歩 歩 歩|七
| ・ 角 ・ ・ ・ ・ ・ 飛 ・|八
| 香 桂 銀 金 玉 金 銀 桂 香|九
+---------------------------+
"#[1..]
            )
        );
    }

    #[test]
    fn parse_not_move_line() {
        assert!(not_move_line("").is_err());
        assert!(not_move_line("* comment line\n").is_err());
        assert!(not_move_line("手数----指手---------消費時間--\n").is_ok());
        assert!(not_move_line("1 ７六歩(77) ( 0:16/00:00:16)").is_err());
    }

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
            move_time_format(" 0:16")
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
        assert_eq!(
            Ok((
                "",
                TimeFormat {
                    h: Some(0),
                    m: 0,
                    s: 19
                }
            )),
            move_time_format(" 0:00:19")
        );
    }

    #[test]
    fn parse_move_line() {
        assert!(move_line("").is_err());
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
            )),
            move_line("1 ７六歩(77) ( 0:16/00:00:16)\n")
        );
        assert_eq!(
            Ok((
                "",
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
            )),
            move_line("3 中断 ( 0:03/ 0:00:19)\n")
        );
    }

    #[test]
    fn parse_moves() {
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
            moves(
                &r#"
1 ７六歩(77) ( 0:16/00:00:16)
2 ３四歩(33) ( 0:00/00:00:00)
3 中断 ( 0:03/ 0:00:19)
"#[1..],
            )
        );
        assert_eq!(
            Ok((
                "",
                vec![
                    MoveFormat {
                        comments: Some(vec![String::from("ヒント:こじあける感覚")]),
                        ..Default::default()
                    },
                    MoveFormat {
                        move_: Some(MoveMoveFormat {
                            color: Color::Black,
                            from: Some(PlaceFormat { x: 2, y: 5 }),
                            to: PlaceFormat { x: 2, y: 4 },
                            piece: Kind::GI,
                            same: None,
                            promote: None,
                            capture: None,
                            relative: None,
                        }),
                        time: Some(Time {
                            now: TimeFormat::default(),
                            total: TimeFormat {
                                h: Some(0),
                                ..Default::default()
                            }
                        }),
                        ..Default::default()
                    },
                ]
            )),
            moves(
                &r#"
*ヒント:こじあける感覚
  1 ２四銀(25)   ( 0:00/00:00:00)
"#[1..]
            )
        )
    }
}
