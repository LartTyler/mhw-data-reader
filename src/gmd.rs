use std::convert::TryFrom;

use nom::bytes::complete::{tag, take};
use nom::character::complete::char;
use nom::combinator::map_res;
use nom::error::ErrorKind;
use nom::IResult;
use nom::multi::count;
use nom::number::complete::le_u32;
use nom::sequence::terminated;
use nom::tag;
use nom::take;
use nom::take_until;
use num_enum::TryFromPrimitive;

use crate::take_null_terminated_string;

#[derive(Debug, TryFromPrimitive)]
#[repr(u32)]
pub enum Language {
    Japanese,
    English,
    French,
    Spanish,
    German,
    Italian,
    Korean,
    ChineseTraditional,
    ChineseSimplified,
    Russian = 10,
    Polish = 11,
    Portuguese = 21,
    Arabic = 22,
}

#[derive(Debug)]
pub struct GmdHeader {
    pub version: u32,
    pub language: Language,
    pub filename: String,
    key_count: u32,
    string_count: u32,
    key_block_size: u32,
    string_block_size: u32,
}

#[derive(Debug)]
pub struct GmdEntry {
    pub key: Option<String>,
    pub value: String,
}

#[derive(Debug)]
pub struct GmdDocument {
    pub header: GmdHeader,
    pub entries: Vec<GmdEntry>,
}

pub fn parse(input: &[u8]) -> Result<GmdDocument, &'static str> {
    match parse_document(input) {
        Ok((_, document)) => Ok(document),
        Err(e) => Err("Invalid document"),
    }
}

pub fn parse_document(input: &[u8]) -> IResult<&[u8], GmdDocument> {
    let (input, header) = parse_header(input)?;
    let (input, entries) = parse_entries(input, &header)?;

    Ok((input, GmdDocument {
        header,
        entries,
    }))
}

pub fn parse_header(input: &[u8]) -> IResult<&[u8], GmdHeader> {
    // Throw away file type header bytes
    let (input, _) = terminated(tag("GMD"), char('\0'))(input)?;

    let (input, version) = le_u32(input)?;
    let (input, language) = map_res(
        le_u32,
        |value| Language::try_from(value),
    )(input)?;

    // Throw away empty bytes following language code
    let (input, _) = take(8usize)(input)?;

    let (input, key_count) = le_u32(input)?;
    let (input, string_count) = le_u32(input)?;

    let (input, key_block_size) = le_u32(input)?;
    let (input, string_block_size) = le_u32(input)?;

    // Throw away filename_length (we're just gonna take the next null-terminated string instead
    let (input, _) = take(4usize)(input)?;
    let (input, filename) = take_null_terminated_string(input);

    Ok((input, GmdHeader {
        version,
        key_count,
        string_count,
        key_block_size,
        string_block_size,
        filename,
        language,
    }))
}

pub fn parse_entries(input: &[u8], header: &GmdHeader) -> IResult<&[u8], Vec<GmdEntry>> {
    // We'll need the info table (or at least the indexes that start each chunk of the info table)
    // later on.
    let (input, info_indexes) = count(
        |input| {
            let (input, string_index) = le_u32(input)?;
            let (input, _) = take(28usize)(input)?;

            Ok((input, string_index))
        },
        header.key_count as usize,
    )(input)?;

    // There's a big block of data after the info table that I don't know anything about, but I'm
    // about 99.9% certain I don't need it for what I'm doing. We're just gonna throw it out.
    let (input, _) = take(0x800usize)(input)?;

    let (input, keys) = count(
        take_null_terminated_string,
        header.key_count as usize,
    )(input)?;

    let (input, mut strings) = count(
        take_null_terminated_string,
        header.string_count as usize,
    )(input)?;

    let mut entries: Vec<GmdEntry> = Vec::with_capacity(header.string_count as usize);

    for i in 0..header.key_count as usize {
        // For some godsforsaken reason, GMD files can contain _more effing strings_ than keys.
        // Fortunately, the indexes we parsed out of the info table tell us what the index of the
        // string that belongs to each key is.
        //
        // That means that, when we're parsing strings out, all we need to do is add zero or more
        // entries with `None` keys while the index of the string for the key we're on is less than
        // the number of entries we've added so far (minus one because zero indexed sets). Doing so
        // will ensure that keys match up to their associated strings the way they're supposed to.
        for _ in 0..(info_indexes[i] - entries.len() as u32) {
            entries.push(GmdEntry {
                key: None,
                value: strings.remove(0),
            });
        }

        entries.push(GmdEntry {
            key: Some(keys[i].clone()),
            value: strings.remove(0),
        })
    }

    Ok((input, entries))
}
