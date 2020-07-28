This repo contains prototype code for parsing game data out of chunks for Monster Hunter: World. Code is very much in
flux, and will likely be completely replaced once all the files and formats we need have been mapped out.

# Conventions
Unless specified otherwise, the following conventions will be adhered to for this document.

- Byte offsets are noted as base 10 integers, unless prefixed by `0x`, which indicates hexidecimal.
- Data types are written to match [Rust's scalar types](https://doc.rust-lang.org/book/ch03-02-data-types.html#scalar-types).

# File Types
|Extension|Description|
|---|---|
|.itm|Stores data for items, such as rarity and sell price|
|.gmd|Contains strings in the language the file name is suffixed with|

## Files of Interest
|Path|Description|
|---|---|
|/common/text/steam/item_*.gmd|Item strings|
|/common/item/itemData.itm|Item data|

# Item Data
There are two main files (or groups of files) used for item data.

- /common/text/steam/item_*.gmd<sup>[1](#item-data-1)</sup>
- /common/item/itemData.itm

<a name="item-data-1"></a><sup>1</sup> As an aside, what appears to be almost identical files can also be found at
`/common/text/item_*.gmd`. A note [on this forum post](https://forums.nexusmods.com/index.php?/topic/6966336-tutorial-itemdataitm/)
indicates that the files in the `steam` sub-directory contain more information.

The `.gmd` files store the strings for each item, while the `.itm` file stores the item's numeric data. All files are
indexed in a related order; that is to say, an item's name is at offset `itemId * 2`, and an item's description is at
offset `itemId * 2 + 1` in the `.gmd` files.

## itemData.itm Layout
This file is made up of two main parts: the header, which contains information on how many items are listed in the file,
and the body, which is broken up into 18 byte chunks.

The header begins with 6 bytes of junk data which we can throw away (which appears to be a version number), followed by
a u32 indicating how many chunks are contained in the body of the file. The entire header is 10 bytes long.

Each chunk contains the data for a single item, identified by it's `id` field. The structure of each chunk is detailed
in the table below.

|Offset|Name|Type|
|---|---|---|
|0|id|u32|
|4|subtype|u8|
|5|container_type<sup>[1](#itemData-fields-1)</sup>|u8|
|6|rarity|u8|
|7|carry_limit|u8|
|8|sort_order|u16|
|10|sell_price|u32|
|14|buy_price|u32|

<a name="itemData-fields-1"></a><sup>1</sup> Field name comes from [this forum post](https://forums.nexusmods.com/index.php?/topic/6966336-tutorial-itemdataitm/),
as the field seems to indicate which section of the inventory UI the item can be found in.
