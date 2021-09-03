# Filesystem
The filesystem disk is a list of 64Kb pages.  All files are a certain number of
pages.  The first page is system information.  The Ardaku WasmFS supports drives
with up to almost 64Tb (2⁴⁶ Bytes, 2³⁰ Pages) of indexable storage (which at the
time of writing, don't exist).

 - Byte `[u32]` (Cannot be indexed directly, but sub-index of page index)
 - Sector `[u64]` 4 Kb
 - Subpage `[u32]` 16 Kb (Smallest 32-bit addressable data size)
 - Page `[u32]` Load size: 64 Kb / 16 Sectors / 4 Sub-pages

 0. Store system information
   - System Name: `1 Kb`
   - Filesystem header: `u64`
   - Number of page table pages (One page for each 32GB of storage): `u16`
   - Number of pages on last page table: `u16`
   - Next empty page index: `u32`
   - Number of pages required for **File Infix Tree**: `u32`
   - @2Kb Page indices for **File Infix Tree**: `[u32; _]`
 1. Starting at page 1, store page table (up to 512 pages / 32Gb)
   - List of bit flags for each page as to whether or not it's used.

 - **File Infix Tree**: Trie data structure mapping unicode (UTF-8) filename to
   `(file_metadata_page_index: u32)`.

 - **File Metadata**: `16Kb`.
   - Application/FileType: u256 `[u8; 32]`
   - Number of Tags: `u16`
   - Number of Pages: `u16`
   - Related Tags (File Metadata): `[u32; 4060]`

## System Information Block (1 Page, 64 Kb)
```
# System Name: 1Kb

name_size: u16[1:1020]
name: [u8; 1020] # unicode

# Variables: 1Kb
next_empty_page_index: u32
file_crc32: 
_: RESERVED

```

## Page Usage Table (2047 pages, 128 Mb - 64 Kb)
Supports up to about 1 billion pages.

```
[i1; 1_073_741_824 - 524_288] # 2³⁰ - 2¹⁹
```

## File Project (1 Subpage, 16 Kb)
```
# File Name: 0.25Kb
name: [char; 64] # unicode U32 file name.
name_bytes: u16[1:1020]
name_chars: u8[1:255]
metadata_type: u8
 - 0: End of metadata
 - 1: Filename Tag
 - 2: Related Project Tag
 - 3: Label Tag

```

================================================================================

## System Information (1024 pages / 4 Chunks, 64 MB)
```
# System Name: 1Kb
name: [u8; 1020] # unicode
name_bytes: u16[1:1020]
name_chars: u8[1:255]
hashmap_chunk_size: u8[1:256] # chunk unit is "256 pages" (16 MB)

# Hashmap (Filename => @FileMeta) chunk indices: 1Kb
chunks: [u32; 256]

# Layout Information
pages: u32 # Number of pages available for allocation

@

# Used Pages Bitfield (32Kb)
page: [u1; 4_294_967_296] # 536_870_912 bytes => 512 MB





# Files List
file_metadata_pointer: NonZero[u64]

...

page_ext: u32
```

## File Metadata
```
# Name: 1Kb
name: [u8; 1020] # unicode
name_RESERVED: u8[0:0]
name_chars: u8[1:255]
name_bytes: u16[1:1020]


```

```
# Page(s) Info
page_id: NonZero[u32]
checksum: u32

...

page_ext: u32
```
