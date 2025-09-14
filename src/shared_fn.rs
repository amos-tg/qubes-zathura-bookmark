use crate::{
    shared_consts::*,
    conf::Conf,
};
use anyhow::anyhow;
use std::{
    fs,
    env::var,
    num::TryFromIntError,
};
use qrexec_binds::QIO;

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

/// bytes needs to be an accurate representation of the 
/// number of bytes being written over the
/// course of the entire message response chain that
/// comprises the request, this function takes into account
/// the length added by the num_reads array itself, 4 bytes.  
pub fn num_reads_encode(bytes: usize) -> Result<[u8; 4], TryFromIntError> {
    let num_reads = 
        ((bytes + 4) / BLEN).try_into()?;

    #[cfg(target_endian = "big")]
    return Ok(u32::to_be_bytes(num_reads));

    #[cfg(target_endian = "little")]
    return Ok(u32::to_le_bytes(num_reads));
}

pub fn num_reads_decode(bytes: [u8; 4]) -> u32 {
    #[cfg(target_endian = "big")]
    return u32::from_be_bytes(bytes);

    #[cfg(target_endian = "little")]
    return u32::from_le_bytes(bytes);
}

/// this function assumes that the fpath
/// absolute path exists in the file system.
pub fn send_file(
    qrx: impl QIO, 
    fpath: &str, 
    conf: &Conf,
    rbuf: &mut [u8; BLEN],
) -> DRes<()> {
    let fcont = fs::read_to_string(&fpath)?.into_bytes();
    let fpath_ab = fpath.as_bytes();

    let num_reads = num_reads_encode(
        VAR_SEND_SFILE.len() + fpath_ab.len() + fcont.len())?;

    let mut query = vec!();
    query.extend_from_slice(VAR_SEND_SFILE); 
    query.extend_from_slice(fpath_ab);



    return Ok(());
}

pub fn recv_file(
    qrx: impl QIO, 
    conf: &Conf,
    rbuf: &mut [u8; BLEN],
) -> DRes<()> {
    
    return Ok(());
}
