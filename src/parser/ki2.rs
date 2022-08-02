use super::kakinoki::parse_without_moves;
use crate::jkf::*;
use nom::combinator::map;
use nom::error::VerboseError;
use nom::sequence::pair;
use nom::IResult;

fn entire_moves(input: &str) -> IResult<&str, Vec<MoveFormat>, VerboseError<&str>> {
    todo!()
}

pub(crate) fn parse(input: &str) -> IResult<&str, JsonKifuFormat, VerboseError<&str>> {
    map(
        pair(parse_without_moves, entire_moves),
        |(mut jkf, moves)| {
            jkf.moves.extend(moves);
            jkf
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        assert_eq!(Ok(("", JsonKifuFormat::default())), parse(""));
    }
}
