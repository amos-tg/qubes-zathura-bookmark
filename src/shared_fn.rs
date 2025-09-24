use crate::{
    shared_consts::*,
    conf::Conf,
};
use anyhow::anyhow;
use std::{
    path::Path,
    num::TryFromIntError,
    fs,
};
use qrexec_binds::QIO;

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
    let mut num_reads = ((bytes + 4) / BLEN).try_into()?;
    if num_reads == 0 {
        num_reads = 1;
    }

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

/// if it is a dir pass true to is_dir
pub fn send_file(
    qrx: &mut impl QIO, 
    fpath: &Path, 
    rbuf: &mut [u8; BLEN],
    is_dir: bool,
) -> DRes<()> {
    let fcont = fs::read(&fpath)?;

    let fpath_ab = fpath.file_name()
        .ok_or(anyhow!(MISSING_BASENAME_ERR))?
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
    cursor += set_slice(&mut rbuf[cursor..], &[b':']);
    cursor += set_slice(&mut rbuf[cursor..], &[is_dir as u8]); 
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

    let mut wslice;
    let mut rem_len;
    let mut max_fit_len;
    let mut nb;
    for _ in 0..(num_reads - 1) {
        wslice = {
            rem_len = fcont[cursor..].len();
            max_fit_len = fcont[cursor..(cursor + BLEN)].len();
            if rem_len > max_fit_len {
                &fcont[cursor..]
            } else {
                &fcont[cursor..(cursor + BLEN)]
            } 
        };

        cursor += qrx.write(wslice)?; 
        nb = qrx.read(rbuf)?;
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
    conf: &Conf,
    rbuf: &mut [u8; BLEN],
    inital_read_num_bytes: usize,
) -> DRes<()> {
    const REQ_LEN: usize = VAR_SEND_SFILE.len();

    let mut fcont = vec!();
    let mut nb;

    let fname_di = find_delim(&rbuf[REQ_LEN..], b':')
        .ok_or(anyhow!(MSG_FORMAT_ERR))?;
    let num_reads_di = find_delim(&rbuf[fname_di..], b':')
        .ok_or(anyhow!(MSG_FORMAT_ERR))?;

    let fname = str::from_utf8(&rbuf[REQ_LEN..fname_di])?
        .to_owned();
    let num_reads = num_reads_decode(
        rbuf[(fname_di + 1)..num_reads_di].try_into()?);
    let is_dir = rbuf[num_reads_di + 1]; 

    fcont.extend_from_slice(
        &rbuf[(num_reads_di + 1)..inital_read_num_bytes]);

    for _ in 0..(num_reads - 1) {
        nb = qrx.read(rbuf)?; 
        fcont.extend_from_slice(&rbuf[..nb]);
        qrx.write(RECV_SEQ)?;
    } 

    if is_dir == 0 {
        fs::write(
            format!("{}/{}", conf.state_dir, fname),
            fcont)?;
    } else {
        fs::create_dir_all(fname)?;
    }
    
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
