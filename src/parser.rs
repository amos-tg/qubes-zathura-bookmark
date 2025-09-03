use crate::shared_consts::*;
use anyhow::anyhow;

/// returns DRes<(fname, fcontents)>
#[inline(always)]
pub fn parse_buf(buf: &[u8]) -> DRes<(&str, &str)> {
        let read_cont = str::from_utf8(buf)?;
        let (fname, fcont) = read_cont.split_at(
            read_cont.find(';').ok_or(
                anyhow!(NAME_DELIM_ERR))?);
        return Ok((fname, fcont));
}
