use crate::{
    shared_consts::*,
    shared_fn::*,
    conf::Conf,
    recv_seq,
};
use std::{
    fs::{self, FileType},
    path::{PathBuf, Path},

};
use qrexec_binds::{QrexecServer, QIO};
use anyhow::anyhow;

pub fn server_main(conf: Conf) -> DRes<()> {
    let mut buf = [0u8; BLEN];
    let mut qrx = QrexecServer::new();
    loop {
        request_handler(&mut qrx, &conf, &mut buf)?;
    }
}

fn request_handler(
    qrx: &mut QrexecServer,
    conf: &Conf,
    rbuf: &mut [u8; BLEN],
) -> DRes<()> {
    let nb = qrx.read(rbuf)?;
    match rbuf {
        rbuf if rbuf.starts_with(VAR_SEND_SFILE) => 
            recv_file(qrx, conf, rbuf, nb)?,

        rbuf if rbuf.starts_with(VAR_GET_BOOK) => 
            send_book(qrx, conf, rbuf)?,

        rbuf if rbuf.starts_with(GET_SFILES) => 
            send_sfile_tree(qrx, conf, rbuf)?,

        rbuf if rbuf.starts_with(GET_BOOKNAMES) => 
            send_booknames(qrx, conf, rbuf)?,

        _ => unreachable!(),
    }
    
    return Ok(());
}

fn send_booknames(
    qrx: &mut QrexecServer,
    conf: &Conf,
    rbuf: &mut [u8; BLEN],
) -> DRes<()> {
    let mut bnames: Vec<u8> = vec!();
    let bdir_entries = fs::read_dir(&conf.book_dir)?;

    for bentry in bdir_entries {
        bnames.extend_from_slice(
            bentry?.file_name()
                .to_str() 
                .ok_or(anyhow!(INVALID_ENC_ERR))?
                .as_bytes());

        bnames.push(b';');
    }

    if bnames.is_empty() {
        qrx.write(NONE)?; 
        recv_seq!(qrx, rbuf);
        return Ok(());
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
    recv_seq!(qrx, rbuf);

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

fn send_book(
    qrx: &mut QrexecServer,
    conf: &Conf,
    rbuf: &mut [u8; BLEN],
) -> DRes<()> {
    let delim1 = find_delim(rbuf, b':')
        .ok_or(anyhow!(MSG_FORMAT_ERR))?;
    let delim2 = find_delim(rbuf, b';') 
        .ok_or(anyhow!(MSG_FORMAT_ERR))?;

    let bname = str::from_utf8(&rbuf[(delim1 + 1)..delim2])?;
    let bpath = find_book(Path::new(&conf.book_dir), bname)?;

    send_file(qrx, &bpath, rbuf, false)?;

    return Ok(());
}

fn find_book(book_dir: &Path, bname: &str) -> DRes<PathBuf> {
    let path = book_dir.join(bname).to_owned();
    if fs::exists(&path)? {
        return Ok(path);
    } else {
        for file in fs::read_dir(book_dir)? {
            let file = file?;
            if file.file_type()?.is_dir() {
                return find_book(&file.path(), bname);
            } 
        }
        return Err(anyhow!(BOOK_UNAVAILABLE_ERR))?;
    }                
}

fn send_sfile_tree(
    qrx: &mut QrexecServer,
    conf: &Conf,
    rbuf: &mut [u8; BLEN],
) -> DRes<()> {
    let mut file_paths: Vec<(PathBuf, FileType)> = vec!();
    let mut cursor = 0usize;

    recurse_files(
        fs::read_dir(&conf.state_dir)?,
        &mut file_paths)?;

    if file_paths.is_empty() {
        qrx.write(NONE)?;
        recv_seq!(qrx, rbuf);
        return Ok(());
    }

    let (num_files_bytes, _) = num_reads_encode(
        file_paths.len())?;

    cursor += set_slice(rbuf, VAR_SEND_NUM_SFILES);
    cursor += set_slice(&mut rbuf[cursor..], &num_files_bytes);
    cursor += set_slice(&mut rbuf[cursor..], &[b';']); 

    qrx.write(&rbuf[..cursor])?;
    recv_seq!(qrx, rbuf);

    for (fpath, ftype) in file_paths {
        let ftype = ftype.is_dir();
        send_file(qrx, fpath.as_path(), rbuf, ftype)?;
    }

    return Ok(());
}

fn recurse_files(
    read_dir: fs::ReadDir, files: &mut Vec<(PathBuf, FileType)>,) -> DRes<()> {
    for file in read_dir {
        let file = file?;
        let path = file.path();
        let file_type = file.file_type()?;

        if file_type.is_file() {
            files.push((path, file_type));
        } else if file_type.is_dir() {
            recurse_files(
                fs::read_dir(&path)?,
                files)?;
            files.push((path, file_type));
        } else if file_type.is_symlink() {
            Err(anyhow!(SYMLINK_ERR))?;
        }
    }
    
    return Ok(());
}
