use nom::IResult;
use nom::number::complete::be_u16;
use nom::bytes::complete::{take, tag_no_case};

pub fn length_value(input: &[u8]) -> IResult<&[u8],&[u8]> {
    let (input, length) = be_u16(input)?;
    take(length)(input)
}


pub fn select(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag_no_case("SELECT")(input)
}

pub fn from(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag_no_case("FROM")(input)
}