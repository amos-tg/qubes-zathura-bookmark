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
use qrexec_binds::Qrexec;

pub fn client_main() -> DRes<()> {
    const RPC_SERVICE_NAME: &str = "qubes.ZathuraMgmt";
    const ZATHURA_BMARK_VM_VAR: &str = "ZATHURA_BMARK_VM";
    const ZATHURA_BMARK_VM_VAR_ERR: &str = 
        "Error: ZATHURA_BMARK_VM env var is not present";

    let mut recv_seq_buf = [0u8; 1];

    let dpath = init_dir()?;
    let path = Path::new(&dpath);

    let vm_name = var(ZATHURA_BMARK_VM_VAR)
        .or(Err(anyhow!(ZATHURA_BMARK_VM_VAR_ERR)))?;
    let mut qrx = Qrexec::new(&[&vm_name, RPC_SERVICE_NAME])?;

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
            let fcont = fs::read_to_string(&path)?;

            Qrexec::write(&mut qrx.stdin, fcont.as_bytes())?;
            Qrexec::read(&mut qrx.stdout, &mut recv_seq_buf)?;

            if recv_seq_buf[0] != RECV_SEQ {
                return Err(anyhow!(RECV_SEQ_ERR).into());
            };
        }
    } 
}

fn restore_zathura_fs(
    qrx: &mut Qrexec,
    zstate_dir: &str,
) -> DRes<()> {
    const NUM_FILES: usize = 
        [BMARKS_FNAME, INPUT_HISTORY_FNAME, HISTORY_FNAME].len();

    let mut buf = [0u8; KIB64];
    let recv_seq_buf = [RECV_SEQ; 1];

    for _ in 0..NUM_FILES {
        let _ = Qrexec::read(&mut qrx.stdout, &mut buf)?; 
        let _ = Qrexec::write(&mut qrx.stdin, &recv_seq_buf)?;

        let read_cont = str::from_utf8(&buf)?;
        let (fname, fcont) = read_cont.split_at(
            read_cont.find(';').ok_or(
                anyhow!(NAME_DELIM_ERR))?);

        fs::write(
            format!("{}/{}", zstate_dir, fname),
            fcont)?;
    }

    return Ok(());
}
