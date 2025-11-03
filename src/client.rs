use crate::{
    shared_consts::*, 
    shared_fn::*,
    conf::Conf,
    ERR_FNAME,
    ERR_LOG_DIR_NAME,
};
use std::{
    collections::HashMap, ffi::OsStr, fs::{self, ReadDir}, io::{self, Read}, os::unix::net::UnixStream, path::{Path, PathBuf}, thread::park_timeout
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

//FEATURE LIST
//
// * HashSet powered file diffing for state file updates | |
//                                                     
// * Bookname forwarding from   
//

pub fn client_main(conf: Conf) -> DRes<()> {
    const RPC_SERVICE_NAME: &str = "qubes.ZathuraMgmt";

    let mut rbuf = [0u8; BLEN];

    let zstate_path = Path::new(&conf.state_dir);
    let book_path = Path::new(&conf.book_dir);

    let mut qrx = QrexecClient::new::<KIB64>(
        &conf.target_vm, RPC_SERVICE_NAME,
        None, None)?;

    initialize_files(&mut qrx, &conf, &mut rbuf)?;

    loop {
             
    }

    //let mut inotify = Inotify::init()?;
    //let mut watches = inotify.watches(); 

    //let mut state_wfds = vec!();
    //recurse_dir_watch(
    //    WatchMask::CREATE | WatchMask::MODIFY,
    //    &mut watches, zstate_path, &mut state_wfds)?;

    //let mut book_wfds = vec!();
    //recurse_dir_watch(
    //    WatchMask::ACCESS,
    //    &mut watches, book_path, &mut book_wfds)?;

    //let mut event_buf = [0u8; 8192];
    //let mut events;

    //loop {
    //    park_timeout(SECONDS_2);

    //    get_booknames(&mut qrx, &conf, &mut rbuf)?;

    //    events = match inotify.read_events(&mut event_buf) {
    //        Ok(events) => Some(events),
    //        Err(err) if err.kind() == io::ErrorKind::Interrupted => None, 
    //        Err(err) if err.kind() == io::ErrorKind::WouldBlock => None,
    //        Err(err) => {
    //            let err = Err(err);
    //            err_append(&err, ERR_FNAME, ERR_LOG_DIR_NAME);
    //            err?;
    //            unreachable!();
    //        }
    //    };

    //    if events.is_none() { continue }

    //    for event in events.unwrap() {
    //        if state_wfds.contains(&event.wd) { 
    //            err_append(
    //                &state_noti(&mut qrx, event, &mut rbuf), 
    //                ERR_FNAME, ERR_LOG_DIR_NAME);
    //        } else if book_wfds.contains(&event.wd) { 
    //            err_append(
    //                &book_noti(&mut qrx, event, &mut rbuf, &conf),
    //                ERR_FNAME, ERR_LOG_DIR_NAME);
    //        }
    //    }
    //}
}

fn handle_book_comms<'a>(
    zsock: &mut UnixStream,
    rbuf: &'a mut [u8; BLEN]
) -> Result<Option<&'a [u8]>, io::Error> {
    let nb = zsock.read(rbuf)?;
    if nb > 0 {
        return Ok(Some(&rbuf[..nb]));
    } else {
        return Ok(None);
    }
}

fn handle_state_fs_comms(
    fs_states: &mut HashMap<PathBuf, String>,
    qrx: &mut QrexecClient,
    rbuf: &mut [u8; BLEN],
    conf: &Conf,
) -> DRes<()> {
    let fchanged = state_fs_changes(
        fs_states, fs::read_dir(&conf.state_dir)?)?; 

    for file in fchanged {
        send_file(qrx, &file, rbuf, file.is_dir())?;
    }

    return Ok(());
}

// returns a vector of PathBuf's which have been changed
// inside of the conf.state_dir fields indicated directory
// which is monitored recursively.
fn state_fs_changes(
    fs_states: &mut HashMap<PathBuf, String>,
    read_dir: ReadDir, 
) -> DRes<Vec<PathBuf>> {
    let mut fupdates = vec!();
    let mut current_files = vec!();
    for entry in read_dir {
        let file = entry?;
        let fpath = file.path();

        if fpath.is_dir() {
            let changes = state_fs_changes(fs_states, fs::read_dir(&fpath)?)?;
            fupdates.extend_from_slice(&changes[..changes.len()]);
        }

        current_files.push(fpath.clone());

        let file_string = fs::read_to_string(&fpath)?;
        let mref_kval = fs_states.get_mut(&fpath);
        if let Some(mref_kval) = mref_kval {
            if *mref_kval != file_string {
                fupdates.push(fpath); 
            }
        } else {
            let _ = fs_states.insert(fpath.clone(), file_string);
            fupdates.push(fpath);
        }
    }

    // going to avoid filter here because it could be really expensive
    let mut del_list = vec!();
    for (key, _) in fs_states.iter() {
        if !current_files.contains(key) {
            del_list.push(key.clone());
        } 
    }

    for key in del_list {
        let _ = fs_states.remove(&key);
    }

    return Ok(fupdates);
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
                    }
                )
            ) 
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

    let (header, cont) = (
        &rbuf[..delim_idx],
        &rbuf[(delim_idx+1)..rnb]); 

    let num_reads_bytes = header
        .split(|x|  *x == b':')
        .skip(2)
        .next()
        .ok_or(anyhow!(MSG_FORMAT_ERR))?;

    let num_reads = num_reads_decode(
        num_reads_bytes.try_into()?);

    book.extend_from_slice(cont);

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
