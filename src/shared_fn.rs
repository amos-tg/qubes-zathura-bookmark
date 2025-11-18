use crate::shared_consts::*;
use std::num::TryFromIntError;


pub enum Content {
    One(Vec<u8>),
    More(Vec<Vec<u8>>),
    None,
}

/// takes a vector of AsRef<[u8]>, compacts them into a vector
/// in the following format based on data initial vectors indice
/// boundaries. cursor is an offset to correct the ordered_indices
/// based on the context of the buffer in which it is placed, it's 
/// the starting index where the indexed_data will be placed:
///
/// 1 =  num_indexes: u32
/// 2 =  index pairs: (u32, u32): length = 1 (per index, not per pair)
/// 3 =  data: byte stream
///
/// the byte stream can be reassembled into multiple distinct byte
/// sets as seperated by indices
pub fn index_data(
    data: Vec<impl AsRef<[u8]>>,
    mut cursor: usize,
) -> Result<Vec<u8>, TryFromIntError> { 
    let mut compacted: Vec<u8> = vec!();
    let mut ordered_indices: Vec<u8> = vec!();

    for part in data {
        let len = part.as_ref().len();

        ordered_indices.extend_from_slice(
            &u32::to_ne_bytes(cursor.try_into()?));

        ordered_indices.extend_from_slice(
            &u32::to_ne_bytes((len + cursor).try_into()?));

        cursor += len;
    }

    compacted.extend_from_slice(
        &u32::to_ne_bytes(ordered_indices.len().try_into()?));

    compacted.extend_from_slice(&ordered_indices);

    return Ok(compacted);
}

pub fn deindex_data(data: Vec<u8>) -> DRes<Vec<Vec<u8>>> {
    let mut deindexed: Vec<Vec<u8>> = vec!();
    let mut cursor = 4;
    let mut endex = 12;
    let mut nindices = u32::from_ne_bytes(data[..cursor].try_into()?);
    let mut indices = Vec::with_capacity(nindices.try_into()?);
    while nindices != 0 {
        indices.push(
            u32::from_ne_bytes(data[cursor..endex].try_into()?));
        cursor += 4;
        endex += 4;

        indices.push(
            u32::from_ne_bytes(data[cursor..endex].try_into()?));
        cursor += 4;
        endex += 4;
        nindices -= 2;
    } 

    for (idx, val) in indices.iter().enumerate() {
        let mut bounded_data = vec!();
        bounded_data.extend_from_slice(
            &data[(*val as usize)..(indices[idx+1] as usize)]);
        deindexed.push(bounded_data);
    }

    return Ok(deindexed);
}

#[macro_export]
macro_rules! recv_seq {
    ($qrx:expr, $rbuf:expr) => {
        assert!(
            1 == $qrx.read($rbuf)?
            && $rbuf[0] == RECV_SEQ[0],
            "{}", RECV_SEQ_ERR);
    };
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
pub fn num_reads_encode(bytes: usize) -> Result<([u8; 4], u32), TryFromIntError> {
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
