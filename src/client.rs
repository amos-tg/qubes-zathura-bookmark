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
    DRes,
    qrexec::Qrexec,
    shared_consts::*,
};

const KIB64: usize = 65536;

pub fn client_main() -> DRes<()> {
    const RPC_SERVICE_NAME: &str = "qubes.ZathuraMgmt";
    const ZATHURA_BMARK_VM_VAR: &str = "ZATHURA_BMARK_VM";
    const ZATHURA_BMARK_VM_VAR_ERR: &str = 
        "Error: ZATHURA_BMARK_VM env var is not present";

    let mut recv_seq_buf = [0u8; 1];

    let path_str = format!(
        "{}/{}",
        var("HOME").or(Err(anyhow!(HOME_VAR_ERR)))?,
        ZATHURA_PATH_POSTFIX,
    );
    let path = Path::new(&path_str);

    let vm_name = var(ZATHURA_BMARK_VM_VAR)
        .or(Err(anyhow!(ZATHURA_BMARK_VM_VAR_ERR)))?;
    let mut qrx = Qrexec::new(&[&vm_name, RPC_SERVICE_NAME])?;

    restore_zathura_fs(&mut qrx, &path_str)?;

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
    let mut buf = [0u8; KIB64];

    let nb = Qrexec::read(&mut qrx.stdout, &mut buf)?;
    let file1 = str::from_utf8(&buf[..nb])?; 
    fs::write(
        format!("{}/{}", zstate_dir, ),
        file1)?;

    buf[0] = 1; 
    Qrexec::write(&mut qrx.stdin, &mut buf[0..1])?; 

    let nb = Qrexec::read(&mut qrx.stdout, &mut buf)?; 
    let file2 = str::from_utf8(&buf[..nb])?;
    fs::write(
        format!("{}/{}", zstate_dir, FILE2_NAME), 
        file2)?;

    buf[0] = 1; 
    Qrexec::write(&mut qrx.stdin, &mut buf[0..1])?; 

    let nb = Qrexec::read(&mut qrx.stdout, &mut buf)?;
    let file3 = str::from_utf8(&buf[..nb])?; 
    fs::write(
        format!("{}/{}", zstate_dir, FILE3_NAME),
        file3)?;

    return Ok(());
}
