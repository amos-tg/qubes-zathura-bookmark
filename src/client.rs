use std::{
    env::var,
    sync::mpsc,
    path::Path,
    fs,
};
use notify::{
    recommended_watcher,
    Watcher,
    RecursiveMode,
    EventKind,
};
use anyhow::anyhow;
use crate::{
    shared_consts::*, 
    shared_fn::*,
};
use qrexec_binds::{QrexecClient, QIO};

pub fn client_main() -> DRes<()> {
    const RPC_SERVICE_NAME: &str = "qubes.ZathuraMgmt";
    const ZATHURA_BMARK_VM_VAR: &str = "ZATHURA_BMARK_VM";
    const ZATHURA_BMARK_VM_VAR_ERR: &str = 
        "Error: ZATHURA_BMARK_VM env var is not present";

    let mut recv_seq_buf = [0u8; 1];

    let dpath = init_dir()?;
    let path = Path::new(&dpath);

    let vm_name = var(ZATHURA_BMARK_VM_VAR).or(
        Err(anyhow!(ZATHURA_BMARK_VM_VAR_ERR)))?;

    let mut qrx = QrexecClient::<KIB64>::new(
        &vm_name, RPC_SERVICE_NAME,
        None, None)?;

    restore_zathura_fs(&mut qrx, &dpath)?;

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

fn restore_booknames(
    qrx: &mut QrexecClient::<KIB64>,
    book_dir: &str, 
) -> DRes<()> {
    let mut rbuf = [0u8; WBUF_LEN];
    let mut bnames;
    let mut bn_len;

    let nb = qrx.read(&mut rbuf)?;

    // have to scope here or rust's borrow checker will complain
    {
    let iref_rbuf = &rbuf[..nb];
    let buf_cont = str::from_utf8(iref_rbuf)?;
    bnames = buf_cont.split(';')
        .map(|x| x.to_owned())
        .collect::<Vec<String>>();
    }

    bn_len = bnames.len();
    assert!(
        bn_len >= 1,
        "{}", MSG_FORMAT_ERR);

    let num_books = bnames[0].parse::<usize>()?;

    while num_books != (bn_len - 1) && nb == WBUF_LEN {
        let old_len = bn_len;

        let nb = qrx.read(&mut rbuf)?;

        let buf_cont = str::from_utf8(&rbuf[..nb])?;
        bnames.extend(
            buf_cont.split(';')
                .map(|x| x.to_owned()));

        bn_len = bnames.len();
        if old_len == bn_len {
            Err(anyhow!(BNAME_NE_SIZE_ERR))?;
        }
    } 

    for bname in bnames {
        fs::write(
            &format!("{book_dir}/{bname}"),
            "")?;
    }

    return Ok(());
}

// format: 
//   query: book;bookname 
//   response: num_reads;content
//   subsequents: content ... 
fn restore_book(
    qrx: &mut QrexecClient::<KIB64>,
    bname: &str, 
    book_dir: &str,
) -> DRes<()> {
    let mut buf = [0u8; WBUF_LEN];
    let mut book = Vec::<u8>::new();
    let mut rnb;

    let query = format!("book;{bname}");
    let qlen = query.len();
    assert!(
        qlen < WBUF_LEN,
        "{}", MSG_LEN_WBUF_ERR);

    let wnb = qrx.write(query.as_bytes())?; 
    assert!(
        wnb == query.len(), 
        "{}", WBYTES_NE_LEN_ERR);

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
