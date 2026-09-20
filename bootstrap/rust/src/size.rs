//! `nx size`: attribute binary bytes to declarations (S5).
//!
//! The program is compiled to an object with `-ffunction-sections
//! -fdata-sections`, so every function and static lands in its own section.
//! Section sizes are then read from the object file (ELF or COFF) and mapped
//! back to Nexium declarations by their C symbol names.

use std::collections::BTreeMap;

pub struct SizeEntry {
    pub symbol: String,
    pub bytes: u64,
    pub kind: &'static str,
}

/// Parse section sizes from an ELF64 or COFF object.
pub fn section_sizes(bytes: &[u8]) -> Result<Vec<(String, u64)>, String> {
    if bytes.len() >= 4 && &bytes[0..4] == b"\x7fELF" {
        return elf64(bytes);
    }
    if bytes.len() >= 2 && (bytes[0..2] == [0x64, 0x86] || bytes[0..2] == [0x64, 0xaa]) {
        return coff(bytes);
    }
    if bytes.len() >= 4 && (bytes[0..4] == [0xcf, 0xfa, 0xed, 0xfe]) {
        return Err("Mach-O objects are not supported by `nx size` yet; run it on a Linux or Windows target (`--target x86_64-linux-gnu`)".into());
    }
    Err("unrecognized object file format".into())
}

fn u16le(b: &[u8], o: usize) -> u64 {
    u16::from_le_bytes([b[o], b[o + 1]]) as u64
}
fn u32le(b: &[u8], o: usize) -> u64 {
    u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]) as u64
}
fn u64le(b: &[u8], o: usize) -> u64 {
    let mut a = [0u8; 8];
    a.copy_from_slice(&b[o..o + 8]);
    u64::from_le_bytes(a)
}

fn cstr(b: &[u8], o: usize) -> String {
    let end = b[o..].iter().position(|&c| c == 0).map(|p| o + p).unwrap_or(b.len());
    String::from_utf8_lossy(&b[o..end]).to_string()
}

fn elf64(b: &[u8]) -> Result<Vec<(String, u64)>, String> {
    if b.len() < 64 || b[4] != 2 {
        return Err("only ELF64 objects are supported".into());
    }
    let shoff = u64le(b, 0x28) as usize;
    let shentsize = u16le(b, 0x3a) as usize;
    let shnum = u16le(b, 0x3c) as usize;
    let shstrndx = u16le(b, 0x3e) as usize;
    if shoff + shnum * shentsize > b.len() {
        return Err("truncated ELF section table".into());
    }
    let sh = |i: usize| shoff + i * shentsize;
    let strtab_off = u64le(b, sh(shstrndx) + 0x18) as usize;
    let mut out = Vec::new();
    for i in 0..shnum {
        let name_off = u32le(b, sh(i)) as usize;
        let flags = u64le(b, sh(i) + 0x08);
        let size = u64le(b, sh(i) + 0x20);
        let name = cstr(b, strtab_off + name_off);
        // SHF_ALLOC only
        if flags & 0x2 != 0 && size > 0 {
            out.push((name, size));
        }
    }
    Ok(out)
}

fn coff(b: &[u8]) -> Result<Vec<(String, u64)>, String> {
    if b.len() < 20 {
        return Err("truncated COFF header".into());
    }
    let nsec = u16le(b, 2) as usize;
    let symoff = u32le(b, 8) as usize;
    let nsym = u32le(b, 12) as usize;
    let strtab = symoff + nsym * 18;
    let mut out = Vec::new();
    for i in 0..nsec {
        let o = 20 + i * 40;
        if o + 40 > b.len() {
            break;
        }
        let raw = &b[o..o + 8];
        let name = if raw[0] == b'/' {
            let idx: usize = String::from_utf8_lossy(&raw[1..]).trim_end_matches('\0').trim().parse().unwrap_or(0);
            if strtab + idx < b.len() {
                cstr(b, strtab + idx)
            } else {
                String::from_utf8_lossy(raw).trim_end_matches('\0').to_string()
            }
        } else {
            String::from_utf8_lossy(raw).trim_end_matches('\0').to_string()
        };
        let size = u32le(b, o + 16); // SizeOfRawData (VirtualSize is zero in objects)
        let chars = u32le(b, o + 36);
        // skip debug/info sections (IMAGE_SCN_MEM_DISCARDABLE) and empty ones
        if chars & 0x0200_0000 == 0
            && size > 0
            && !name.starts_with(".debug")
            && !name.starts_with(".pdata")
            && !name.starts_with(".xdata")
            && !name.starts_with(".rdata$zzz")
            && !name.starts_with(".drectve")
            && !name.starts_with(".llvm")
        {
            out.push((name, size));
        }
    }
    Ok(out)
}

/// Group section sizes by the declaration they belong to.
pub fn attribute(sections: &[(String, u64)], funcs: &[(String, String)]) -> (Vec<SizeEntry>, u64, u64) {
    // funcs: (C symbol, Nexium name)
    let mut by_symbol: BTreeMap<String, (u64, &'static str)> = BTreeMap::new();
    let mut runtime = 0u64;
    let mut total = 0u64;
    for (name, size) in sections {
        total += size;
        let (kind, sym) = if let Some(s) = name.strip_prefix(".text.") {
            ("code", s.to_string())
        } else if let Some(s) = name.strip_prefix(".rodata.") {
            ("data", s.trim_start_matches("str1.").to_string())
        } else if let Some(s) = name.strip_prefix(".text$") {
            ("code", s.to_string())
        } else if let Some(s) = name.strip_prefix(".rdata$") {
            ("data", s.to_string())
        } else if let Some(s) = name.strip_prefix(".data.") {
            ("data", s.to_string())
        } else if let Some(s) = name.strip_prefix(".bss.") {
            ("bss", s.to_string())
        } else if name.starts_with(".text") {
            ("code", "(other code)".to_string())
        } else {
            ("data", format!("(section {})", name))
        };
        let nx_name = funcs.iter().find(|(c, _)| *c == sym).map(|(_, n)| n.clone());
        let key = match nx_name {
            Some(n) => n,
            None if sym.starts_with("nxc_") => format!("const {}", sym.trim_start_matches("nxc_").rsplit_once('_').map(|(a, _)| a).unwrap_or(&sym)),
            None if sym.starts_with("nxg_") => format!("global {}", sym.trim_start_matches("nxg_").rsplit_once('_').map(|(a, _)| a).unwrap_or(&sym)),
            None if sym.starts_with("nx_drop_") || sym.starts_with("nx_clone_") || sym.starts_with("nx_eq_") || sym.starts_with("nx_cmp_") || sym.starts_with("nx_qcmp_") => {
                "(generated glue: drop, clone, compare)".to_string()
            }
            None if sym.starts_with("nx_thunk_") || sym.starts_with("nx_closure_") || sym.starts_with("nx_env_") => "(closures and function values)".to_string(),
            None if sym.starts_with("nx_str_") || sym.starts_with("nx_enum_names_") => "(string literals)".to_string(),
            None if sym.starts_with('(') => sym.clone(),
            None => {
                runtime += size;
                continue;
            }
        };
        let e = by_symbol.entry(key).or_insert((0, kind));
        e.0 += size;
    }
    let mut entries: Vec<SizeEntry> = by_symbol.into_iter().map(|(symbol, (bytes, kind))| SizeEntry { symbol, bytes, kind }).collect();
    entries.sort_by(|a, b| b.bytes.cmp(&a.bytes).then(a.symbol.cmp(&b.symbol)));
    (entries, runtime, total)
}
