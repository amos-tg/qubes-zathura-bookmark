use crate::{
    shared_consts::*,
    shared_fn::*,
};
use std::fs;
use qrexec_binds::{
    QrexecServer, 
    QIO,
};
use anyhow::anyhow;

pub fn server_main() -> DRes<()> {
    let dpath = init_dir()?;
    let mut qrx = QrexecServer::<KIB64>::new();

    restore_zathura_fs(&mut qrx, &dpath)?;

    let mut buf = [0u8; BLEN];
    let recv_seq_buf = [1u8];
    loop {
        request_handler(&mut qrx)?;
    }
}

fn request_handler(
    qrx: &mut QrexecServer,
    rbuf: &mut [u8],
) -> DRes<()> {
    let nb = qrx.read(rbuf);
    let id = find_delim(&rbuf[..nb]).ok_or(
        anyhow!(MSG_FORMAT_ERR))?;

    let request = str::from_utf8(&rbuf[..id])?;
    

    return Ok(());
}

// booknames cannot have semicolons. 
// Most books don't use semicolons in their titles so I feel this
// is okay.
//
// format:  
// {number of booknames};{bookname};{bookname}; ...
fn restore_booknames(
    qrx: &mut QrexecServer::<KIB64>,
) -> DRes<()> {
        
    return Ok(());
}


// read bytes format
// {number of state files};{state_filename};{state_file};...
//
// repeating filenames are appended to the same state file
// and do not contribute to the total left to read
fn restore_zathura_fs(
    qrx: &mut QrexecServer::<KIB64>,
    dir_path: &String,
) -> DRes<()> {
    let mut seq_buf: [u8; 1] = [0];
    let mut nb;
    let mut path;

    for file in FILES {
        path = format!("{}/{}", dir_path, file);

        if fs::exists(&path)? {
            let fcont = fs::read_to_string(&path)?;
            let written = format!(
                "{};{}",
                file,
                fcont);

            let writb = written.as_bytes();
            nb = qrx.write(&writb)?;
            assert!(
                nb == writb.len(),
                "{}", WBYTES_NE_LEN_ERR);

            nb = qrx.read(&mut seq_buf)?;
            assert!(
                seq_buf[0] == RECV_SEQ && nb == 1, 
                "{}", RECV_SEQ_ERR);
        } else {
            nb = qrx.write(NONE)?;
            assert!(nb == NONE.len());

            nb = qrx.read(&mut seq_buf)?;
            assert!(
                seq_buf[0] == RECV_SEQ && nb == 1,
                "{}", RECV_SEQ_ERR);
        }
    }

    return Ok(());
}
