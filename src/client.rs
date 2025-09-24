use std::{
    sync::mpsc,
    path::Path,
    fs,
};
use notify::{
    recommended_watcher,
    Watcher,
    RecursiveMode,
    EventKind,
    event::AccessKind,
};
use anyhow::anyhow;
use crate::{
    shared_consts::*, 
    shared_fn::*,
    conf::Conf,
};
use qrexec_binds::{QrexecClient, QIO};

pub fn client_main(conf: Conf) -> DRes<()> {
    const RPC_SERVICE_NAME: &str = "qubes.ZathuraMgmt";
        "Error: ZATHURA_BMARK_VM env var is not present";

    let mut rbuf = [0u8; BLEN];

    let zstate_path = Path::new(&conf.state_dir);
    let book_path = Path::new(&conf.book_dir);

    let mut qrx = QrexecClient::new::<KIB64>(
        &conf.target_vm, RPC_SERVICE_NAME,
        None, None)?;

    initialize_files(&mut qrx, &conf, &mut rbuf)?;

    let (tx, rx) = mpsc::channel();

    let mut book_watcher = recommended_watcher(tx.clone())?;
    let mut state_watcher = recommended_watcher(tx)?;

    state_watcher.watch(zstate_path, RecursiveMode::Recursive)?;
    book_watcher.watch(book_path, RecursiveMode::Recursive)?;

    loop {
        let event = rx.recv()??;

        match event.paths[0]
            .as_path()
            .parent()
            .ok_or(anyhow!(MISSING_DIRNAME_ERR))?
        {
            path if path == zstate_path => {
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) => {
                        for path in event.paths {
                            let f_d: bool = path.is_dir();
                            send_file(
                                &mut qrx,
                                path.as_path(),
                                &mut rbuf,
                                f_d)?;
                        }
                    } 
                    _ => (),
                }
            }
            path if path == book_path => {
                match event.kind {
                    EventKind::Access(ak) => {
                        if let AccessKind::Close(_) = ak {
                            continue;
                        }

                        for path in event.paths {
                            let bname = path.file_name()
                                .ok_or(anyhow!(MISSING_BASENAME_ERR))?
                                .to_str()
                                .ok_or(anyhow!(INVALID_ENC_ERR))?;

                            get_book(
                                &mut qrx,
                                &conf,
                                bname,
                                &mut rbuf)?;
                        }
                    }
                    _ => (),
                }
            }
            _ => unreachable!(), 
        }
    } 
}

fn initialize_files(
    qrx: &mut QrexecClient,
    conf: &Conf, 
    rbuf: &mut [u8; BLEN],
) -> DRes<()> {
    get_booknames(qrx, conf, rbuf)?;
    get_state_fs(qrx, conf, rbuf)?;

    return Ok(());
}

fn get_booknames(
    qrx: &mut QrexecClient,
    conf: &Conf, 
    rbuf: &mut [u8; BLEN],
) -> DRes<()> {
    let mut bnames = vec!();
    let mut rnb;
    let mut cont;
    let num_reads;

    macro_rules! push_names {
        ($vec_names:expr, $buf:expr) => {
            $vec_names.extend(
                str::from_utf8($buf)?
                    .split(';')
                    .filter_map({ |x| 
                        if !x.is_empty() { 
                            Some(x.to_string())
                        } else {
                            None
                        }
                    }))
        }; 
    }

    qrx.write(GET_BOOKNAMES)?;
    rnb = qrx.read(rbuf)?;
    qrx.write(RECV_SEQ)?;

    if rbuf.starts_with(NONE) {
        return Ok(());
    } 

    cont = &rbuf[..rnb];
    let delim_idx = find_delim(cont, b';').ok_or(
        anyhow!(MSG_FORMAT_ERR))?;

    let split = cont.split_at(delim_idx);
    let num_reads_bytes = split.0;
    cont = &split.1[1..];

    num_reads = num_reads_decode(num_reads_bytes.try_into()?);

    push_names!(bnames, cont);

    for _ in 0..(num_reads - 1) {
        rnb = qrx.read(rbuf)?;
        qrx.write(RECV_SEQ)?;
        push_names!(bnames, &rbuf[..rnb]);
    } 

    for bname in bnames {
        fs::File::create(
            &format!("{}/{}", conf.book_dir, bname))?;
    }

    return Ok(());
}

fn get_book(
    qrx: &mut QrexecClient,
    conf: &Conf,
    bname: &str, 
    rbuf: &mut [u8; BLEN], 
) -> DRes<()> {
    let mut book = Vec::<u8>::new();
    let mut rnb: usize;

    let mut query = vec!();
        query.extend_from_slice(VAR_GET_BOOK);
        query.extend_from_slice(bname.as_bytes());
        query.push(b';');
    let qlen = query.len();
    assert!(qlen < BLEN, "{}", MSG_LEN_WBUF_ERR);

    qrx.write(&query)?; 
    rnb = qrx.read(rbuf)?;
    qrx.write(RECV_SEQ)?;

    let delim_idx = find_delim(&rbuf[..rnb], b';')
        .ok_or(anyhow!(MSG_FORMAT_ERR))?;

    let num_reads = num_reads_decode(
        rbuf[..delim_idx].try_into()?);

    book.extend_from_slice(&rbuf[(delim_idx + 1)..]);

    for _ in 0..num_reads {
        rnb = qrx.read(rbuf)?; 
        qrx.write(RECV_SEQ)?;
        book.extend_from_slice(&rbuf[..rnb]);
    }

    fs::write(&format!("{}/{}", conf.book_dir, bname), book)?; 

    return Ok(());
}

fn get_state_fs(
    qrx: &mut QrexecClient,
    conf: &Conf,
    rbuf: &mut [u8; BLEN],
) -> DRes<()> {
    let mut nb; 

    qrx.write(GET_SFILES)?;
    nb = qrx.read(rbuf)?;
    qrx.write(RECV_SEQ)?;

    let id = find_delim(&rbuf[..nb], b':')
        .ok_or(anyhow!(MSG_FORMAT_ERR))?;

    let mut num_files = num_reads_decode(
        rbuf[(id + 1)..nb].try_into()?);

    while num_files != 0 {
        nb = qrx.read(rbuf)?;
        recv_file(qrx, conf, rbuf, nb)?;
        qrx.write(RECV_SEQ)?;

        num_files -= 1; 
    }

    return Ok(());
}
