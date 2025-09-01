use crate::{
    DRes,
    qrexec::Qrexec,
    shared_consts::*,
};
use std::{
    env::var,
    io::{self, Stdin, Stdout},
    fs,
};
use anyhow::anyhow;

pub fn server_main() -> DRes<()> {
    let (mut stdin, mut stdout) = (
        io::stdin(),
        io::stdout());

    let dpath = init_dir()?;
    restore_zathura_fs(&mut stdin, &mut stdout, &dpath)?;

    return Ok(());
}

fn restore_zathura_fs(
    stdin: &mut Stdin,
    stdout: &mut Stdout,
    dir_path: &String,
    ) -> DRes<()> {
    let fhandler = |
        dir_path: &String,
        fname: &str,
        stdout: &mut Stdout,
        stdin: &mut Stdin,
    | {
        let mut recv_seq_buf = [0u8; 1];
        let path = format!("{}/{}", dir_path, fname);

        if fs::exists(&path)? {
            let fcont = fs::read_to_string(&path)?;
            let written = format!(
                "{};{}",
                fname,
                fcont);
            let writb = written.as_bytes();

            let nb = Qrexec::write(stdout, writb)?;
            if nb != writb.len() {
                panic!("{}", WBYTES_NE_LEN_ERR);
            }

            let _ = Qrexec::read(stdin, &mut recv_seq_buf)?;
            assert!(
                recv_seq_buf[0] == RECV_SEQ,
                "{}", RECV_SEQ_ERR);
        }

        return Ok::<(), io::Error>(());
    };

    fhandler(dir_path, BMARKS_FNAME, stdout, stdin)?;
    fhandler(dir_path, HISTORY_FNAME, stdout, stdin)?;
    fhandler(dir_path, INPUT_HISTORY_FNAME, stdout, stdin)?;

    return Ok(());
}

fn init_dir() -> DRes<String> {
    let path = format!(
        "{}/{}",
        var("HOME").or(Err(anyhow!(HOME_VAR_ERR)))?,
        ZATHURA_PATH_POSTFIX,
    );

    fs::create_dir_all(&path)?;

    return Ok(path);
}
