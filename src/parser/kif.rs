use crate::jkf::*;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{digit1, line_ending, none_of, not_line_ending, one_of, space0};
use nom::combinator::{map, map_res, opt, value};
use nom::error::VerboseError;
use nom::multi::{count, many0, many1};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use nom::IResult;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Information {
    Preset(Preset),
    HandBlack(Hand),
    HandWhite(Hand),
    KeyValue(String, String),
}

#[derive(Debug, Default, PartialEq, Eq)]
struct InformationData {
    preset: Option<Preset>,
    hands: [Hand; 2],
    map: HashMap<String, String>,
}

impl InformationData {
    fn merged(lhs: Self, rhs: Self) -> InformationData {
        InformationData {
            preset: lhs.preset.or(rhs.preset),
            hands: Self::merged_hands(lhs.hands, rhs.hands),
            map: lhs.map.into_iter().chain(rhs.map.into_iter()).collect(),
        }
    }
    fn merged_hands(lhs: [Hand; 2], rhs: [Hand; 2]) -> [Hand; 2] {
        [
            Self::merged_hand(lhs[0], rhs[0]),
            Self::merged_hand(lhs[1], rhs[1]),
        ]
    }
    fn merged_hand(lhs: Hand, rhs: Hand) -> Hand {
        Hand {
            FU: lhs.FU + rhs.FU,
            KY: lhs.KY + rhs.KY,
            KE: lhs.KE + rhs.KE,
            GI: lhs.GI + rhs.GI,
            KI: lhs.KI + rhs.KI,
            KA: lhs.KA + rhs.KA,
            HI: lhs.HI + rhs.HI,
        }
    }
}

fn comment_line(input: &str) -> IResult<&str, String, VerboseError<&str>> {
    map(
        delimited(tag("#"), not_line_ending, line_ending),
        String::from,
    )(input)
}

fn move_comment_line(input: &str) -> IResult<&str, String, VerboseError<&str>> {
    alt((
        map(
            delimited(tag("*"), not_line_ending, line_ending),
            String::from,
        ),
        map(delimited(tag("&"), not_line_ending, line_ending), |s| {
            String::from("&") + s
        }),
    ))(input)
}

fn kansuji(input: &str) -> IResult<&str, u8, VerboseError<&str>> {
    alt((
        value(18, tag("十八")),
        value(17, tag("十七")),
        value(16, tag("十六")),
        value(15, tag("十五")),
        value(14, tag("十四")),
        value(13, tag("十三")),
        value(12, tag("十二")),
        value(11, tag("十一")),
        value(10, tag("十")),
        value(9, tag("九")),
        value(8, tag("八")),
        value(7, tag("七")),
        value(6, tag("六")),
        value(5, tag("五")),
        value(4, tag("四")),
        value(3, tag("三")),
        value(2, tag("二")),
        value(1, tag("一")),
    ))(input)
}

fn information_value_hand(input: &str) -> IResult<&str, Hand, VerboseError<&str>> {
    alt((
        value(Hand::default(), tag("なし")),
        map_res(
            many1(terminated(
                pair(piece_kind, map(opt(kansuji), |o| o.unwrap_or(1))),
                many0(one_of(" 　")),
            )),
            |v| {
                v.iter().try_fold(Hand::default(), |mut acc, &(k, n)| {
                    match k {
                        Kind::FU => acc.FU += n,
                        Kind::KY => acc.KY += n,
                        Kind::KE => acc.KE += n,
                        Kind::GI => acc.GI += n,
                        Kind::KI => acc.KI += n,
                        Kind::KA => acc.KA += n,
                        Kind::HI => acc.HI += n,
                        _ => return Err(()),
                    }
                    Ok(acc)
                })
            },
        ),
    ))(input)
}

fn information_value_preset(input: &str) -> IResult<&str, Information, VerboseError<&str>> {
    terminated(
        map(
            alt((
                value(Preset::PresetHirate, tag("平手")),
                value(Preset::PresetKY, tag("香落ち")),
                value(Preset::PresetKYR, tag("右香落ち")),
                value(Preset::PresetKA, tag("角落ち")),
                value(Preset::PresetHI, tag("飛車落ち")),
                value(Preset::PresetHIKY, tag("飛香落ち")),
                value(Preset::Preset2, tag("二枚落ち")),
                value(Preset::Preset3, tag("三枚落ち")),
                value(Preset::Preset4, tag("四枚落ち")),
                value(Preset::Preset5, tag("五枚落ち")),
                value(Preset::Preset5L, tag("左五枚落ち")),
                value(Preset::Preset6, tag("六枚落ち")),
                value(Preset::Preset7L, tag("左七枚落ち")),
                value(Preset::Preset7R, tag("右七枚落ち")),
                value(Preset::Preset8, tag("八枚落ち")),
                value(Preset::Preset10, tag("十枚落ち")),
                value(Preset::PresetOther, tag("その他")),
            )),
            Information::Preset,
        ),
        many0(one_of(" 　")),
    )(input)
}

fn information_line_preset(input: &str) -> IResult<&str, Information, VerboseError<&str>> {
    terminated(
        preceded(tag("手合割："), information_value_preset),
        line_ending,
    )(input)
}

fn information_line_hands(input: &str) -> IResult<&str, Information, VerboseError<&str>> {
    terminated(
        map(
            pair(
                terminated(
                    alt((
                        value(Color::Black, tag("先手")),
                        value(Color::White, tag("後手")),
                        value(Color::Black, tag("下手")),
                        value(Color::White, tag("上手")),
                    )),
                    tag("の持駒："),
                ),
                information_value_hand,
            ),
            |(c, h)| match c {
                Color::Black => Information::HandBlack(h),
                Color::White => Information::HandWhite(h),
            },
        ),
        line_ending,
    )(input)
}

fn information_line_keyvalue(input: &str) -> IResult<&str, Information, VerboseError<&str>> {
    terminated(
        map(
            separated_pair(
                map(is_not("：\r\n"), String::from),
                tag("："),
                map(not_line_ending, String::from),
            ),
            |(k, v)| Information::KeyValue(k, v),
        ),
        line_ending,
    )(input)
}

fn informations(input: &str) -> IResult<&str, InformationData, VerboseError<&str>> {
    map(
        many0(preceded(
            many0(comment_line),
            alt((
                information_line_preset,
                information_line_hands,
                information_line_keyvalue,
            )),
        )),
        |v| {
            v.iter().fold(InformationData::default(), |mut acc, info| {
                match info {
                    Information::Preset(p) => acc.preset = Some(*p),
                    Information::HandBlack(h) => acc.hands[0] = *h,
                    Information::HandWhite(h) => acc.hands[1] = *h,
                    Information::KeyValue(k, v) => {
                        acc.map.insert(k.to_owned(), v.to_owned());
                    }
                }
                acc
            })
        },
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
        value(Kind::OU, alt((tag("玉"), tag("王")))),
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
    delimited(none_of(" 0123456789*"), not_line_ending, line_ending)(input)
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
        tuple((move_to, piece_kind, opt(tag("成")), move_from)),
        |(to, kind, promote, from)| {
            MoveFormat {
                move_: Some(MoveMoveFormat {
                    color: Color::Black, // To be replaced
                    from,
                    to: to.unwrap_or_default(), // Might be (0, 0) if it's the same place as previous
                    piece: kind,
                    same: if to.is_none() { Some(true) } else { None },
                    promote: promote.map(|_| true),
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
            mf.time = time;
            (i, mf)
        },
    )(input)
}

fn move_with_comments(input: &str) -> IResult<&str, (usize, MoveFormat), VerboseError<&str>> {
    map(
        pair(move_line, many0(move_comment_line)),
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
        terminated(many1(move_with_comments), opt(not_move_line)),
        |v| (v[0].0, v.into_iter().map(|(_, mf)| mf).collect()),
    )(input)
}

fn main_moves(input: &str) -> IResult<&str, Vec<MoveFormat>, VerboseError<&str>> {
    map(
        pair(
            map(many0(move_comment_line), |comments| MoveFormat {
                comments: Some(comments).filter(|v| !v.is_empty()),
                ..Default::default()
            }),
            opt(moves_with_index),
        ),
        |(m0, o)| [vec![m0], o.map_or(Vec::new(), |(_, v)| v)].concat(),
    )(input)
}

fn entire_moves(input: &str) -> IResult<&str, Vec<MoveFormat>, VerboseError<&str>> {
    fn merge_forks(
        (mut moves, mut forks): (Vec<MoveFormat>, Vec<(usize, Vec<MoveFormat>)>),
    ) -> Vec<MoveFormat> {
        let mut stack = Vec::new();
        while let Some(fork) = forks.pop() {
            stack.push(fork);
            if let Some((i, last)) = forks.last_mut() {
                while stack.last().map_or(false, |(j, _)| j >= i) {
                    if let Some((j, fork)) = stack.pop() {
                        if let Some(v) = &mut last[j - *i].forks {
                            v.push(fork);
                        } else {
                            last[j - *i].forks = Some(vec![fork]);
                        }
                    }
                }
            }
        }
        while let Some((i, fork)) = stack.pop() {
            if let Some(v) = &mut moves[i].forks {
                v.push(fork);
            } else {
                moves[i].forks = Some(vec![fork]);
            }
        }
        moves
    }

    map(
        pair(
            preceded(opt(not_move_line), main_moves),
            many0(preceded(many0(not_move_line), moves_with_index)),
        ),
        merge_forks,
    )(input)
}

pub(crate) fn parse(input: &str) -> IResult<&str, JsonKifuFormat, VerboseError<&str>> {
    map(
        tuple((informations, opt(board), informations, entire_moves)),
        |(info1, opt_board, info2, moves)| {
            let info = InformationData::merged(info1, info2);
            let initial = if let Some(board) = opt_board {
                Some(Initial {
                    preset: Preset::PresetOther,
                    data: Some(StateFormat {
                        color: Color::Black,
                        board,
                        hands: info.hands,
                    }),
                })
            } else {
                Some(Initial {
                    preset: info.preset.unwrap_or(Preset::PresetHirate),
                    data: None,
                })
            };
            JsonKifuFormat {
                header: info.map,
                initial,
                moves,
            }
        },
    )(input)
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
    fn parse_information_preset() {
        assert!(information_line_preset("").is_err());
        assert_eq!(
            Ok(("", Information::Preset(Preset::PresetHirate))),
            information_line_preset("手合割：平手　　\n")
        );
        assert_eq!(
            Ok(("", Information::Preset(Preset::PresetKY))),
            information_line_preset("手合割：香落ち\n")
        );
        assert_eq!(
            Ok(("", Information::Preset(Preset::PresetOther))),
            information_line_preset("手合割：その他\n")
        );
    }

    #[test]
    fn parse_information_hand() {
        assert!(information_line_hands("").is_err());
        assert_eq!(
            Ok((
                "",
                Information::HandBlack(Hand {
                    KE: 1,
                    KI: 1,
                    ..Default::default()
                })
            )),
            information_line_hands("先手の持駒：金　桂　\n")
        );
        assert_eq!(
            Ok((
                "",
                Information::HandWhite(Hand {
                    FU: 15,
                    KY: 2,
                    KE: 3,
                    GI: 2,
                    KI: 3,
                    KA: 1,
                    HI: 0
                })
            )),
            information_line_hands("後手の持駒：角　金三　銀二　桂三　香二　歩十五　\n")
        );
        assert_eq!(
            Ok((
                "",
                Information::HandWhite(Hand {
                    FU: 10,
                    KY: 3,
                    KE: 1,
                    GI: 0,
                    KI: 1,
                    KA: 0,
                    HI: 0
                })
            )),
            information_line_hands("後手の持駒：金　桂　香三　歩十　\n")
        );
        assert_eq!(
            Ok((
                "",
                Information::HandBlack(Hand {
                    KA: 1,
                    ..Default::default()
                })
            )),
            information_line_hands("下手の持駒：角　\n")
        );
    }

    #[test]
    fn parse_information_keyvalue() {
        assert!(information_line_keyvalue("").is_err());
        assert!(information_line_keyvalue("# comment\n").is_err());
        assert!(information_line_keyvalue("key：value with not line ending").is_err());
        assert_eq!(
            Ok((
                "",
                Information::KeyValue(String::from("key"), String::from("value"))
            )),
            information_line_keyvalue("key：value\n")
        );
    }

    #[test]
    fn parse_informations() {
        assert_eq!(Ok(("", InformationData::default())), informations(""));
        assert_eq!(
            Ok((
                "",
                InformationData {
                    map: [(String::from("key"), String::from("value"))]
                        .into_iter()
                        .collect(),
                    ..Default::default()
                }
            )),
            informations("# comment\n# comment：comment\nkey：value\n")
        );
    }

    #[test]
    fn parse_piece_kind() {
        assert!(piece_kind("").is_err());
        assert_eq!(Ok(("", Kind::FU)), piece_kind("歩"));
        assert_eq!(Ok(("", Kind::OU)), piece_kind("玉"));
        assert_eq!(Ok(("", Kind::OU)), piece_kind("王"));
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
        let (rest, moves) = ret.expect("failed to parse");
        assert!(rest.is_empty());
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
