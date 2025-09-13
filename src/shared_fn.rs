use crate::shared_consts::*;
use anyhow::anyhow;
use std::{fs, env::var};

/// returns DRes<(fname, fcontents)>
#[inline(always)]
pub fn parse_buf(buf: &[u8]) -> DRes<(&str, &str)> {
        let read_cont = str::from_utf8(buf)?;
        let (fname, fcont) = read_cont.split_at(
            read_cont.find(';').ok_or(
                anyhow!(NAME_DELIM_ERR))?);
        return Ok((fname, fcont));
}

pub fn init_dir() -> DRes<String> {
    let path = format!(
        "{}/{}",
        var("HOME").or(Err(anyhow!(HOME_VAR_ERR)))?,
        ZATHURA_PATH_POSTFIX,
    );

    fs::create_dir_all(&path)?;

    return Ok(path);
}

pub fn find_delim(buf: &[u8], pat: u8) -> Option<usize> {
    for (i, char) in buf.iter().enumerate() {
        if *char == pat {
            return Some(i); } }

    return None;
}

pub fn num_reads_encode(bytes: u32) -> [u8; 4] {
    #[cfg(target_endian = "big")]
    return u32::to_be_bytes(bytes);

    #[cfg(target_endian = "little")]
    return u32::to_le_bytes(bytes);
}

pub fn num_reads_decode(bytes: [u8; 4]) -> u32 {
    #[cfg(target_endian = "big")]
    return u32::from_be_bytes(bytes);

    #[cfg(target_endian = "little")]
    return u32::from_le_bytes(bytes);
}


