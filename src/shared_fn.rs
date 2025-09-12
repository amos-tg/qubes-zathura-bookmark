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
            return Some(i);
        }
    }

    return None;
}
