use crate::shared_consts::*;
use anyhow::anyhow;
use std::{
    path::Path,
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
pub fn num_reads_encode(
    bytes: usize,
) -> Result<([u8; 4], u32), TryFromIntError> {
    let num_reads = 
        ((bytes + 4) / BLEN).try_into()?;

    #[cfg(target_endian = "big")]
    return Ok((u32::to_be_bytes(num_reads), num_reads));

    #[cfg(target_endian = "little")]
    return Ok((u32::to_le_bytes(num_reads), num_reads));
}

pub fn num_reads_decode(bytes: [u8; 4]) -> u32 {
    #[cfg(target_endian = "big")]
    return u32::from_be_bytes(bytes);

    #[cfg(target_endian = "little")]
    return u32::from_le_bytes(bytes);
}

pub fn send_file(
    qrx: &mut impl QIO, 
    fpath: &Path, 
    rbuf: &mut [u8; BLEN],
) -> DRes<()> {
    let fcont = fs::read_to_string(&fpath)?.into_bytes();

    let fpath_ab = fpath.file_name()
        .ok_or(anyhow!(MISSING_BASENAME))?
        .as_encoded_bytes();

    const NUM_DELIMS: usize = 2;
    let (num_reads_arr, num_reads)  = num_reads_encode(
        VAR_SEND_SFILE.len() 
        + fpath_ab.len() 
        + fcont.len()
        + NUM_DELIMS)?;

    let mut cursor = 0usize;
    cursor += set_slice(&mut rbuf[cursor..], VAR_SEND_SFILE); 
    cursor += set_slice(&mut rbuf[cursor..], fpath_ab);
    cursor += set_slice(&mut rbuf[cursor..], &[b':']);
    cursor += set_slice(&mut rbuf[cursor..], &num_reads_arr);
    cursor += set_slice(&mut rbuf[cursor..], &[b';']);

    if num_reads == 1 {
        cursor += set_slice(&mut rbuf[cursor..], &fcont); 
        qrx.write(&rbuf[..cursor])?;
        let nb = qrx.read(rbuf)?;
        assert!(
            rbuf[0] == RECV_SEQ[0]
            && nb == 1,
            "{}", RECV_SEQ_ERR);

        return Ok(());
    }

    let remaining = BLEN - cursor;
    cursor = set_slice(&mut rbuf[cursor..], &fcont[..=remaining]); 

    qrx.write(rbuf)?;
    let nb = qrx.read(rbuf)?;
    assert!(
        rbuf[0] == RECV_SEQ[0]
        && nb == 1,
        "{}", RECV_SEQ_ERR);

    let mut end_i;
    for _ in 0..(num_reads - 1) {
        end_i = cursor + BLEN;
        qrx.write(&fcont[cursor..(end_i)])?; 
        cursor = end_i;

        let nb = qrx.read(rbuf)?;
        assert!(
            rbuf[0] == RECV_SEQ[0]
            && nb == 1,
            "{}", RECV_SEQ_ERR);
    }
    
    return Ok(());
}

/// reads a VAR_SEND_FILE request;
/// the first read must be loaded into rbuf
/// this is the only way the server
/// will know what request it is answering
pub fn recv_file(
    qrx: &mut impl QIO, 
    state_dir: &str, 
    rbuf: &mut [u8; BLEN],
    inital_read_num_bytes: usize,
) -> DRes<()> {
    const REQ_LEN: usize = VAR_SEND_SFILE.len();

    let mut fcont = vec!();
    let mut nb;
    let (fname_di, num_reads_di) = {
        let di_fn = find_delim(&rbuf[REQ_LEN..], b':')
            .ok_or(anyhow!(MSG_FORMAT_ERR))?;
        let di_nr = find_delim(&rbuf[di_fn..], b';')
            .ok_or(anyhow!(MSG_FORMAT_ERR))?;
        (di_fn, di_nr)
    };

    let fname = str::from_utf8(&rbuf[REQ_LEN..fname_di])?
        .to_owned();

    let num_reads = num_reads_decode(
        rbuf[(fname_di + 1)..num_reads_di].try_into()?);

    fcont.extend_from_slice(
        &rbuf[(num_reads_di + 1)..inital_read_num_bytes]);

    for _ in 0..(num_reads - 1) {
        nb = qrx.read(rbuf)?; 
        fcont.extend_from_slice(&rbuf[..nb]);
        qrx.write(RECV_SEQ)?;
    } 

    fs::write(format!("{}/{}", state_dir, fname), fcont)?;
    
    return Ok(());
}

/// returns the index of the first byte 
/// of the slice, not set by the function,
/// another way to think of this is that  
/// the function returns the number of bytes
/// that were set one based.
pub fn set_slice(
    slice: &mut [u8],
    set: &[u8],
) -> usize {
    let mut i = 0usize;
    for val in set {
        slice[i] = *val; 
        i += 1;
    }  
    return i;
}
