use crate::{
    shared_consts::*, 
    shared_fn::*,
    conf::Conf,
    ERR_FNAME,
    ERR_LOG_DIR_NAME,
};
use std::{
    path::Path,
    ffi::OsStr,
    fs,
};
use inotify::{
    Inotify,
    Event, 
    Watches,
    WatchMask,
    WatchDescriptor,
};
use qrexec_binds::{QrexecClient, QIO};
use anyhow::anyhow;
use dbuggery::err_append;

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

    let mut inotify = Inotify::init()?;
    let mut watches = inotify.watches(); 

    let mut state_wfds = vec!();
    recurse_dir_watch(
        WatchMask::CREATE | WatchMask::MODIFY,
        &mut watches, zstate_path, &mut state_wfds)?;

    let mut book_wfds = vec!();
    recurse_dir_watch(
        WatchMask::ACCESS,
        &mut watches, book_path, &mut book_wfds)?;

    let mut event_buf = [0u8; 8192];
    let mut events;

    loop {
        events = inotify.read_events_blocking(&mut event_buf)?;

        for event in events {
            if state_wfds.contains(&event.wd) { 
                err_append(
                    &state_noti(&mut qrx, event, &mut rbuf), 
                    ERR_FNAME, ERR_LOG_DIR_NAME);
            } else if book_wfds.contains(&event.wd) { 
                err_append(
                    &book_noti(&mut qrx, event, &mut rbuf, &conf),
                    ERR_FNAME, ERR_LOG_DIR_NAME);
            }
        }
    }
}

fn state_noti(
    qrx: &mut QrexecClient,
    event: Event<&OsStr>,
    rbuf: &mut [u8; BLEN], 
) -> DRes<()> {
    let path = Path::new(
        event.name
            .ok_or(anyhow!(MISSING_FNAME_ERR))?
            .to_str()
            .ok_or(anyhow!(INVALID_ENC_ERR))?);

    let is_dir = path.is_dir();
    
    send_file(qrx, path.into(), rbuf, is_dir)?;

    return Ok(());
}

fn book_noti(
    qrx: &mut QrexecClient,
    event: Event<&OsStr>,
    rbuf: &mut [u8; BLEN],
    conf: &Conf,
) -> DRes<()> {
    let path = Path::new(
        event.name
            .ok_or(anyhow!(MISSING_FNAME_ERR))?
            .to_str()
            .ok_or(anyhow!(INVALID_ENC_ERR))?);

    if path.is_dir() { return Ok(()) }

    let bname = path.file_name()
        .ok_or(anyhow!(INVALID_ENC_ERR))?
        .to_str()
        .ok_or(anyhow!(INVALID_ENC_ERR))?;

    get_book(qrx, conf, bname, rbuf)?;

    return Ok(());
}

fn recurse_dir_watch(
    watch_mask: WatchMask,
    watches: &mut Watches,
    dir_path: &Path,
    wfd_vec: &mut Vec<WatchDescriptor>,
) -> DRes<()> {
    wfd_vec.push(watches.add(dir_path, watch_mask)?);

    for file in fs::read_dir(dir_path)? {
        let file = file?;
        if file.file_type()?.is_dir() {
            recurse_dir_watch(
                watch_mask, watches,
                &file.path(), wfd_vec)?
        }
    }
    
    return Ok(());
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
        let path = format!("{}/{}", conf.book_dir, bname);
        if fs::exists(&path)? { continue; }
        fs::File::create(&path)?;
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

    if rbuf[..nb].starts_with(NONE) {
        return Ok(()); 
    }

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
