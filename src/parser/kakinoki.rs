use crate::jkf::*;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{line_ending, none_of, not_line_ending, one_of};
use nom::combinator::{map, map_res, opt, value, verify};
use nom::error::VerboseError;
use nom::multi::{count, many0, many1};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use nom::IResult;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Information {
    Preset(Preset),
    Color(Color),
    HandBlack(Hand),
    HandWhite(Hand),
    KeyValue(String, String),
}

#[derive(Debug, Default, PartialEq, Eq)]
struct InformationData {
    preset: Option<Preset>,
    color: Option<Color>,
    hands: [Hand; 2],
    map: HashMap<String, String>,
}

impl InformationData {
    fn merged(lhs: Self, rhs: Self) -> InformationData {
        InformationData {
            preset: lhs.preset.or(rhs.preset),
            color: lhs.color.or(rhs.color),
            hands: Self::merged_hands(lhs.hands, rhs.hands),
            map: lhs.map.into_iter().chain(rhs.map).collect(),
        }
    }
    fn merged_hands(lhs: [Hand; 2], rhs: [Hand; 2]) -> [Hand; 2] {
        [
            Self::merged_hand(lhs[0], rhs[0]),
            Self::merged_hand(lhs[1], rhs[1]),
        ]
    }
    // A record can carry a hands line both before and after the board, so these add up.
    fn merged_hand(lhs: Hand, rhs: Hand) -> Hand {
        Hand {
            FU: lhs.FU.saturating_add(rhs.FU),
            KY: lhs.KY.saturating_add(rhs.KY),
            KE: lhs.KE.saturating_add(rhs.KE),
            GI: lhs.GI.saturating_add(rhs.GI),
            KI: lhs.KI.saturating_add(rhs.KI),
            KA: lhs.KA.saturating_add(rhs.KA),
            HI: lhs.HI.saturating_add(rhs.HI),
        }
    }
}

fn comment_line(input: &str) -> IResult<&str, String, VerboseError<&str>> {
    map(
        delimited(tag("#"), not_line_ending, line_ending),
        String::from,
    )(input)
}

pub(super) fn not_move_line(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    delimited(none_of(" 0123456789*▲△"), not_line_ending, line_ending)(input)
}

pub(super) fn move_comment_line(input: &str) -> IResult<&str, String, VerboseError<&str>> {
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

pub(super) fn piece_kind(input: &str) -> IResult<&str, Kind, VerboseError<&str>> {
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

fn kansuji_digit(input: &str) -> IResult<&str, u8, VerboseError<&str>> {
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

// Reads a hand count, 一 through 九十九. The upper bound matches what SFEN can carry in
// one token, which is also what the writer is allowed to emit.
//
// Never matches the empty string: the caller reads this through `opt` and takes the
// absence of a count to mean one piece.
fn kansuji(input: &str) -> IResult<&str, u8, VerboseError<&str>> {
    verify(
        map(
            pair(
                opt(terminated(opt(kansuji_digit), tag("十"))),
                opt(kansuji_digit),
            ),
            |(tens, ones)| tens.map_or(0, |d| d.unwrap_or(1) * 10) + ones.unwrap_or_default(),
        ),
        |&n| n > 0,
    )(input)
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
                    // One piece kind can be listed more than once, so the counts add up
                    // and can leave the range a `u8` holds.
                    let slot = match k {
                        Kind::FU => &mut acc.FU,
                        Kind::KY => &mut acc.KY,
                        Kind::KE => &mut acc.KE,
                        Kind::GI => &mut acc.GI,
                        Kind::KI => &mut acc.KI,
                        Kind::KA => &mut acc.KA,
                        Kind::HI => &mut acc.HI,
                        _ => return Err(()),
                    };
                    *slot = slot.checked_add(n).ok_or(())?;
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

/// A bare `先手番` / `後手番` line (`上手番` / `下手番` in a handicap game) states the
/// side to move for a board diagram. It carries no `：`, so it does not reach
/// [`information_line_keyvalue`].
fn information_line_color(input: &str) -> IResult<&str, Information, VerboseError<&str>> {
    terminated(
        map(
            terminated(
                alt((
                    value(Color::Black, tag("先手")),
                    value(Color::White, tag("後手")),
                    value(Color::Black, tag("下手")),
                    value(Color::White, tag("上手")),
                )),
                tag("番"),
            ),
            Information::Color,
        ),
        preceded(many0(one_of(" 　")), line_ending),
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
                information_line_color,
                information_line_hands,
                information_line_keyvalue,
            )),
        )),
        |v| {
            v.iter().fold(InformationData::default(), |mut acc, info| {
                match info {
                    Information::Preset(p) => acc.preset = Some(*p),
                    Information::Color(c) => acc.color = Some(*c),
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
    kansuji_digit(input)
}

pub(super) fn move_to(input: &str) -> IResult<&str, Option<PlaceFormat>, VerboseError<&str>> {
    alt((
        value(None, terminated(tag("同"), opt(tag("　")))),
        map(pair(place_x, place_y), |(x, y)| Some(PlaceFormat { x, y })),
    ))(input)
}

pub(super) fn parse_without_moves(
    input: &str,
) -> IResult<&str, JsonKifuFormat, VerboseError<&str>> {
    map(
        tuple((informations, opt(board), informations)),
        |(info1, opt_board, info2)| {
            let info = InformationData::merged(info1, info2);
            let initial = if let Some(board) = opt_board {
                Some(Initial {
                    preset: Preset::PresetOther,
                    data: Some(StateFormat {
                        color: info.color.unwrap_or(Color::Black),
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
                moves: Vec::new(),
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
    fn parse_not_move_line() {
        assert!(not_move_line("").is_err());
        assert!(not_move_line("* comment line\n").is_err());
        assert!(not_move_line("手数----指手---------消費時間--\n").is_ok());
        assert!(not_move_line("1 ７六歩(77) ( 0:16/00:00:16)").is_err());
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
    fn parse_kansuji() {
        // The writer spells a hand count up to 九十九; the reader used to stop at 十八,
        // so anything above that fell through to `information_line_keyvalue` and the
        // hand was silently lost.
        #[rustfmt::skip]
        let cases = [
            ("一", 1), ("九", 9), ("十", 10), ("十一", 11), ("十八", 18), ("十九", 19),
            ("二十", 20), ("二十一", 21), ("三十", 30), ("九十", 90), ("九十九", 99),
        ];
        for (input, expected) in cases {
            assert_eq!(kansuji(input), Ok(("", expected)), "{input}");
        }
        // Not a count: 百 is past what SFEN carries in one token, and the empty string
        // must not match at all — the caller reads this through `opt` and takes a
        // missing count to mean one piece.
        assert!(kansuji("百").is_err());
        assert!(kansuji("").is_err());
        assert_eq!(
            Ok((
                "",
                Information::HandBlack(Hand {
                    FU: 1,
                    HI: 99,
                    ..Default::default()
                })
            )),
            information_line_hands("先手の持駒：飛九十九　歩　\n")
        );
        // Repeating a kind adds the counts up, which can leave the range of a `u8`.
        assert!(information_line_hands("先手の持駒：歩九十九歩九十九歩九十九\n").is_err());
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
}
