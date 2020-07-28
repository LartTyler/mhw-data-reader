use nom::IResult;
use nom::sequence::terminated;
use nom::bytes::complete::take_until;

pub mod gmd;

fn take_null_terminated_string(input: &[u8]) -> IResult<&[u8], String> {
    let (input, value) = terminated(take_until("\0"), char('\0'))(input)?;

    Ok((input, bytes_to_string(value)))
}

fn bytes_to_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}
