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

    restore_zathura_fs(&mut stdin, &mut stdout)?;

    return Ok(());
}

fn restore_zathura_fs(stdin: &mut Stdin, stdout: &mut Stdout) -> DRes<usize> {
    if 
}

fn init_dir() -> DRes<()> {
    let path = format!(
        "{}/{}",
        var("HOME").or(Err(anyhow!(HOME_VAR_ERR)))?,
        ZATHURA_PATH_POSTFIX,
    );

    fs::create_dir_all(&path)?;

    return Ok(());
}
