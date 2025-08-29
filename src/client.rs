use std::{
    io::{self, Read, Write},
    env::var,
    sync::mpsc,
    path::Path,
    process::{
        Child, 
        ChildStdout,
        ChildStdin,
        ChildStderr,
        Command,
    },
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
    FILE1_NAME,
    FILE2_NAME,
    FILE3_NAME,
};

const KIB64: usize = 65536;
const RECV_SEQ: u8 = 1;

pub fn client_main() -> DRes<()> {
    const RPC_SERVICE_NAME: &str = "qubes.ZathuraMgmt";
    const ZATHURA_BMARK_VM_VAR: &str = "ZATHURA_BMARK_VM";
    const ZATHURA_PATH_POSTFIX: &str =  
        ".local/share/zathura";
    const HOME_VAR_ERR: &str = 
        "Error: HOME env var is not present";
    const ZATHURA_BMARK_VM_VAR_ERR: &str = 
        "Error: ZATHURA_BMARK_VM env var is not present";
    const RECV_SEQ_ERR: &str =  
        "Error: read byte did not match the RECV_SEQ sequence";

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
            if recv_seq_buf[0] != 1 {
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
        format!("{}/{}", zstate_dir, FILE1_NAME),
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

struct Qrexec { 
    child: Child,
    stdout: ChildStdout,
    stdin: ChildStdin,
    stderr: ChildStderr,
}

impl Qrexec {
    pub fn new(args: &[&str]) -> DRes<Self> {
        const STDOUT_ERR: &str = 
            "Error: child proc failed to produce stdout";
        const STDIN_ERR: &str = 
            "Error: child proc failed to produce stdin";
        const STDERR_ERR: &str =
            "Error: child proc failed to produce stderr";

        let mut child = Command::new("qrexec-client-vm")
            .args(args)
            .spawn()?;
        return Ok(Self {
            stdout: child.stdout.take().ok_or(
                anyhow!(STDOUT_ERR))?,
            stdin: child.stdin.take().ok_or(
                anyhow!(STDIN_ERR))?,
            stderr: child.stderr.take().ok_or(
                anyhow!(STDERR_ERR))?,
            child,
        })
    }

    /// returns the number of bytes read into the buffer, 
    /// retries the read once on interruption io::Error before returning.
    #[inline(always)]
    pub fn read(
        mut read: impl Read,
        buf: &mut [u8],
    ) -> Result<usize, io::Error> {
        match read.read(buf) {
            Ok(nb) => Ok(nb),
            Err(e) if e.kind() == 
                io::ErrorKind::Interrupted => read.read(buf),
            Err(e) => Err(e),
        }
    } 

    /// returns the number of bytes written into the buffer,
    /// retries the read once on interruption io::Error before returning.
    #[inline(always)]
    pub fn write(
        mut written: impl Write,
        buf: &[u8],
    ) -> Result<usize, io::Error> {
        match written.write(buf) {
            Ok(nb) => Ok(nb),
            Err(e) if e.kind() == 
                io::ErrorKind::Interrupted => written.write(buf),
            Err(e) => Err(e),
        }
    }
}

impl Drop for Qrexec {
    fn drop(&mut self) {
        let _ = self.child.kill();   
    }
}
