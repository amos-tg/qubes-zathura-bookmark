use std::{
    env::var,
    sync::mpsc,
    path::Path,
    process::{Child, Command},
    fs,
};
use notify::{
    recommended_watcher,
    Watcher,
    RecursiveMode,
    EventKind,
};
use anyhow::anyhow;
use crate::DRes;

pub fn client_main() -> DRes<()> {
    const ZATHURA_PATH_POSTFIX: &str =  
        ".local/share/zathura";
    const HOME_VAR_ERR: &str = 
        "Error: HOME env var is not present";

    let path_str = format!(
        "{}/{}",
        var("HOME").or(Err(anyhow!(HOME_VAR_ERR)))?,
        ZATHURA_PATH_POSTFIX,
    );
    let path = Path::new(&path_str);

    let qrx = Qrexec::new()?;
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

        }
    } 
}

struct Qrexec(Child);

impl Qrexec {
    fn new() -> DRes<Self> {
        const ZATHURA_BMARK_VM_VAR: &str = "ZATHURA_BMARK_VM";
        const ZATHURA_BMARK_VM_VAR_ERR: &str = 
            "Error: ZATHURA_BMARK_VM env var is not present";
        const RPC_SERVICE_NAME: &str = "qubes.ZathuraMgmt";

        let vm_name = var(ZATHURA_BMARK_VM_VAR)
            .or(Err(anyhow!(ZATHURA_BMARK_VM_VAR_ERR)))?;
        Ok(Self(Command::new("qrexec-client-vm")
            .args([
                &vm_name,
                RPC_SERVICE_NAME,
            ])
            .spawn()?))
    }
}

impl Drop for Qrexec {
    fn drop(&mut self) {
        let _ = self.0.kill();   
    }
}
