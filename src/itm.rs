use std::convert::TryFrom;

use nom::bytes::complete::take;
use nom::combinator::map_res;
use nom::IResult;
use nom::multi::count;
use nom::number::complete::{le_u32, le_u8, le_u16};
use nom::sequence::preceded;
use num_enum::TryFromPrimitive;

use crate::gmd::GmdDocument;

#[derive(TryFromPrimitive, Debug)]
#[repr(u16)]
pub enum ItemContainerType {
    Item,
    Material,
    AccountItem,
    AmmoCoating,
    Decoration,
    Furniture,
}

#[derive(TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum ItemSubType {
    None,
    Ammo,
    EndemicLife,
    Unknown3,
    Coating,
    Unknown5,
}

#[derive(Debug)]
pub struct ItmEntry {
    pub id: u32,
    pub subtype: ItemSubType,
    pub container_type: ItemContainerType,
    pub rarity: u8,
    pub carry_limit: u8,
    pub sort_order: u16,
    pub sell_price: u32,
    pub buy_price: u32,
    pub name: Option<String>,
}

#[derive(Debug)]
pub struct ItmDocument {
    pub entries: Vec<ItmEntry>,
}

#[derive(Debug)]
pub enum ItmImportResult {
    Success,
    PartialSuccess(usize),
    NoEntriesImported,
}

impl ItmDocument {
    pub fn import_gmd(&mut self, gmd: &GmdDocument) -> ItmImportResult {
        let mut imported = 0usize;

        for item in &mut self.entries {
            if let Some(entry) = gmd.entries.get((item.id * 2) as usize) {
                item.name = Some(entry.value.clone());
                imported += 1;
            }
        }

        if imported == 0 {
            ItmImportResult::NoEntriesImported
        } else if imported < self.entries.len() {
            ItmImportResult::PartialSuccess(imported)
        } else {
            ItmImportResult::Success
        }
    }
}

pub fn parse(input: &[u8]) -> Result<ItmDocument, &'static str> {
    match parse_document(input) {
        Ok((_, document)) => Ok(document),
        Err(_) => Err("Invalid document"),
    }
}

pub fn parse_document(input: &[u8]) -> IResult<&[u8], ItmDocument> {
    // The `take(6usize)` skips unknown bytes prior to count. Thinking it might be a version number,
    // but regardless of what it is, we don't need it.
    let (input, item_count) = preceded(take(6usize), le_u32)(input)?;
    let (input, entries) = count(parse_entry, item_count as usize)(input)?;

    Ok((input, ItmDocument {
        entries,
    }))
}

pub fn parse_entry(input: &[u8]) -> IResult<&[u8], ItmEntry> {
    let (input, id) = le_u32(input)?;

    // The `take(1usize)` skips an unknown byte prior to the subtype.
    let (input, subtype) = map_res(
        preceded(take(1usize), le_u8),
        ItemSubType::try_from,
    )(input)?;

    let (input, container_type) = map_res(
        le_u16,
        ItemContainerType::try_from,
    )(input)?;

    // The `take(1usize)` skips an unknown byte prior to the rarity.
    let (input, rarity) = preceded(take(1usize), le_u8)(input)?;
    let (input, carry_limit) = le_u8(input)?;

    // The `take(1usize)` skips a duplicate (?) carry limit prior to sort order.
    let (input, sort_order) = preceded(take(1usize), le_u16)(input)?;

    // The `take(10usize)` skips several unknown or unused fields prior to sell price.
    let (input, sell_price) = preceded(take(10usize), le_u32)(input)?;
    let (input, buy_price) = le_u32(input)?;

    Ok((input, ItmEntry {
        id,
        subtype,
        container_type,
        rarity,
        carry_limit,
        sort_order,
        sell_price,
        buy_price,
        name: None,
    }))
}
