use crate::{
    shared_consts::*,
    shared_fn::*,
};
use std::{
    io::{
        self,
        Stdin,
        Stdout,
    },
    fs,
};
use qrexec_binds::{Qrexec, errors::*};

pub fn server_main() -> DRes<()> {
    let (mut stdin, mut stdout) = (
        io::stdin(),
        io::stdout());

    let dpath = init_dir()?;
    restore_zathura_fs(&mut stdin, &mut stdout, &dpath)?;

    let mut buf = [0u8; KIB64];
    let recv_seq_buf = &[1u8];
    loop {
        let nb = Qrexec::read(&mut stdin, &mut buf)?;
        let _ = Qrexec::write(&mut stdout, recv_seq_buf)?;

        let (fname, fcont) = parse_buf(&buf[..nb])?;
        fs::write(
            &format!("{dpath}/{fname}"), 
            fcont)?;
    }
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
        recv_seq_buf: &mut [u8; 1],
    | -> QRXRes<()> {
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

            let _ = Qrexec::read(stdin, recv_seq_buf)?;
            assert!(
                recv_seq_buf[0] == RECV_SEQ, 
                "{}", RECV_SEQ_ERR);
            recv_seq_buf[0] = 0;
        }

        return Ok(());
    };

    let mut recv_seq_buf: [u8; 1] = [0];
    fhandler(dir_path, BMARKS_FNAME, stdout, stdin, &mut recv_seq_buf)?;
    fhandler(dir_path, HISTORY_FNAME, stdout, stdin, &mut recv_seq_buf)?;
    fhandler(dir_path, INPUT_HISTORY_FNAME, stdout, stdin, &mut recv_seq_buf)?;

    return Ok(());
}
