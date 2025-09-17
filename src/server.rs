use crate::{
    shared_consts::*,
    shared_fn::*,
    conf::Conf,
};
use std::fs;
use qrexec_binds::{QrexecServer, QIO};
use anyhow::anyhow;

pub fn server_main(conf: Conf) -> DRes<()> {
    let mut buf = [0u8; BLEN];
    let dpath = init_dir()?;
    let mut qrx = QrexecServer::<KIB64>::new();

    loop {
        request_handler(&mut qrx, &conf, &mut buf)?;
    }
}

fn request_handler(
    qrx: &mut QrexecServer::<KIB64>,
    conf: &Conf,
    rbuf: &mut [u8; BLEN],
) -> DRes<()> {
    let nb = qrx.read(rbuf)?;
    match rbuf {
        rbuf if rbuf.starts_with(VAR_SEND_SFILE) => 
            recv_file(qrx, conf, rbuf, nb)?,

        rbuf if rbuf.starts_with(VAR_GET_BOOK) => 
            send_book()?,

        rbuf if rbuf.starts_with(GET_SFILES) => 
            send_sfile_tree(qrx, conf, rbuf)?,

        rbuf if rbuf.starts_with(GET_BOOKNAMES) => 
            send_booknames(qrx, conf, rbuf)?,

        _ => unreachable!(),
    }
    
    return Ok(());
}

fn send_booknames(
    qrx: &mut QrexecServer::<KIB64>,
    conf: &Conf,
    rbuf: &mut [u8; BLEN],
) -> DRes<()> {
    let mut bnames: Vec<u8> = vec!();
    let bdir_entries = fs::read_dir(&conf.book_dir)?;

    for bentry in bdir_entries {
        bnames.extend_from_slice(
            bentry?.file_name()
                .to_str() 
                .ok_or(anyhow!(INVALID_ENC))?
                .as_bytes());

        bnames.push(b';');
    }

    let (nr_bytes, mut nr) = num_reads_encode(bnames.len())?;

    let mut cursor = set_slice(rbuf, &nr_bytes);
    rbuf[cursor] = b';'; 
    cursor += 1;

    let mut bn_cursor = {
        let buf_len_rem = BLEN - cursor;
        if bnames.len() > buf_len_rem {
            buf_len_rem
        } else {
            bnames.len()
        }
    };

    cursor += set_slice(
        &mut rbuf[cursor..], &bnames[..bn_cursor]);

    qrx.write(&rbuf[..cursor])?;
    nr -= 1;
    let rnb = qrx.read(rbuf)?;
    assert!(
        rnb == 1
        && rbuf[0] == RECV_SEQ[0],
        "{}", RECV_SEQ_ERR);

    let mut rem_nb;
    while nr != 0 { 
        rem_nb = bnames[bn_cursor..].len();
        cursor = if rem_nb > BLEN { 
            bn_cursor + BLEN
        } else {
            rem_nb
        };

        bn_cursor += qrx.write(&bnames[bn_cursor..cursor])?;
        nr -= 1;
    }

    return Ok(());
}

fn send_book() -> DRes<()> {
    return Ok(());
}

fn send_sfile_tree(
    qrx: &mut QrexecServer::<KIB64>,
    conf: &Conf,
    rbuf: &mut [u8; BLEN],
) -> DRes<()> {
    recurse_fsend(
        qrx, rbuf,
        fs::read_dir(&conf.state_dir)?)?;

    return Ok(());
}

fn recurse_fsend(
    qrx: &mut QrexecServer::<KIB64>,
    rbuf: &mut [u8; BLEN],
    read_dir: fs::ReadDir,
) -> DRes<()> {
    for file in read_dir {
        let file = file?;
        let path = file.path();
        if file.file_type()?.is_dir() {
            recurse_fsend(
                qrx, rbuf,
                fs::read_dir(&path)?);
            send_file(qrx, &path, rbuf, true)?;
        } else {
            send_file(qrx, &path, rbuf, false)?;
        } 
    }
    
    return Ok(());
}
