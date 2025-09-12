use std::{
    env::var,
    sync::mpsc,
    path::Path,
    str::Utf8Error,
    fs,
};
use notify::{
    recommended_watcher,
    Watcher,
    RecursiveMode,
    EventKind,
    Event,
};
use anyhow::anyhow;
use crate::{
    shared_consts::*, 
    shared_fn::*,
    conf::Conf,
};
use qrexec_binds::{QrexecClient, QIO};

pub fn client_main() -> DRes<()> {
    const RPC_SERVICE_NAME: &str = "qubes.ZathuraMgmt";
    const ZATHURA_BMARK_VM_VAR: &str = "ZATHURA_BMARK_VM";
    const ZATHURA_BMARK_VM_VAR_ERR: &str = 
        "Error: ZATHURA_BMARK_VM env var is not present";

    let mut recv_seq_buf = [0u8; 1];
    let mut rbuf = [0u8; BLEN];
    let conf = Conf::new()?;

    let zstate_path_string = init_dir()?;
    let zstate_path = Path::new(&zstate_path_string);


    let mut qrx = QrexecClient::<KIB64>::new(
        &conf.target_vm, RPC_SERVICE_NAME,
        None, None)?;

    initialize_files(&mut qrx, &conf, &mut rbuf)?;

    let (tx, rx) = mpsc::channel();
    let mut watcher = recommended_watcher(tx)?;
    watcher.watch(&path, RecursiveMode::Recursive)?;
    loop {
        let event = rx.recv()??;
        match event.kind { 
            EventKind::Remove(_) => continue,
            EventKind::Access(_) => continue, 
            _ => (),
        }
        for path in event.paths {
            let fcont = fs::read_to_string(&path)?
                .as_bytes()
                .to_owned();
            let fc_len = fcont.len();

            qrx.write(&fcont[..fc_len])?;
            qrx.read(&mut recv_seq_buf)?;

            if recv_seq_buf[0] != RECV_SEQ {
                return Err(anyhow!(RECV_SEQ_ERR).into());
            };
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

fn request_handler(
    qrx: &mut QrexecClient,
    conf: &Conf,
    rbuf: &mut [u8; BLEN],
    event: Event,
) -> DRes<()> {
    match event

    return Ok(());
}

fn get_booknames(
    qrx: &mut QrexecClient::<KIB64>,
    conf: &Conf, 
    rbuf: &mut [u8; BLEN],
) -> DRes<()> {
    let mut bnames = vec!();
    let mut rnb;
    let mut cont;

    qrx.write(GET_BOOKNAMES)?;
    rnb = qrx.read(rbuf)?;
    qrx.write(RECV_SEQ)?;

    cont = &rbuf[..rnb];
    let delim_idx = find_delim(cont, b';').ok_or(
        anyhow!(MSG_FORMAT_ERR))?;
    let split = cont.split_at(delim_idx);
    let header = split.0;
    cont = split.1;

    let num_reads = {
        let delim_idx = find_delim(header, b':').ok_or(
            anyhow!(MSG_FORMAT_ERR))?;
        let (_, u32) = header.split_at(delim_idx + 1);
        u32::from_be_bytes(u32.try_into()?)
    };

    let mut push_names = || -> Result<(), Utf8Error> {
        bnames.extend(str::from_utf8(cont)?
            .split(';')
            .map(|x| x.to_string()));
        Ok(())
    };
    push_names()?;

    for _ in 0..(num_reads - 1) {
        rnb = qrx.read(rbuf)?;
        let cont = &rbuf[..nb];
        push_names()?;
    } 

    for bname in bnames {
        fs::File::create(
            &format!("{}/{}", conf.book_dir, bname))?;
    }

    return Ok(());
}

fn get_book(
    qrx: &mut QrexecClient::<KIB64>,
    conf: &Conf,
    bname: &str, 
    rbuf: &mut [u8; BLEN], 
) -> DRes<()> {
    let mut book = Vec::<u8>::new();
    let mut rnb;

    let mut query = vec!();
        query.extend_from_slice(VAR_GET_BOOK);
        query.extend_from_slice(bname.as_bytes());

    let qlen = query.len();
    assert!(
        qlen < BLEN,
        "{}", MSG_LEN_WBUF_ERR);
    qrx.write(&query)?; 

    rnb = qrx.read(&mut buf)?;

    let delim_idx = find_delim(&buf[..rnb]).ok_or(
        anyhow!(MSG_FORMAT_ERR))?;
    let num_reads = str::from_utf8(&buf[..delim_idx])?
        .parse::<usize>()?;

    book.extend_from_slice(&buf[(delim_idx + 1)..]);

    for _ in 0..num_reads {
        rnb = qrx.read(&mut buf)?; 
        book.extend_from_slice(&buf[..rnb]);
    }

    fs::write(
        &format!("{book_dir}/{bname}"),
        book)?; 

    return Ok(());
}

fn restore_zathura_fs(
    qrx: &mut QrexecClient::<KIB64>,
    zstate_dir: &str,
) -> DRes<()> {
    let mut buf = [0u8; WBUF_LEN];
    let recv_seq_buf = [RECV_SEQ; 1];
    let (mut nb, mut rnb);

    for _ in FILES {
        rnb = qrx.read(&mut buf[..BLEN])?; 
        nb = qrx.write(&recv_seq_buf)?;
        assert!(nb == 1);

        if buf.starts_with(NONE) {
            continue;
        }   

        let read_cont = str::from_utf8(&buf[..rnb])?;
        let (fname, fcont) = read_cont.split_at(
            read_cont.find(';').ok_or(
                anyhow!(NAME_DELIM_ERR))?);

        fs::write(
            format!("{}/{}", zstate_dir, fname),
            fcont)?;
    }

    return Ok(());
}
