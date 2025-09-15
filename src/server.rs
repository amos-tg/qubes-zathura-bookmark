use crate::{
    shared_consts::*,
    shared_fn::*,
};
use qrexec_binds::QrexecServer;

pub fn server_main() -> DRes<()> {
    let dpath = init_dir()?;
    let mut qrx = QrexecServer::<KIB64>::new();

    return Ok(());
}

fn request_handler(
    qrx: &mut QrexecServer::<KIB64>,
    rbuf: &mut [u8],
) -> DRes<()> {

    return Ok(());
}

fn restore_booknames(
    qrx: &mut QrexecServer::<KIB64>,
) -> DRes<()> {
        
    return Ok(());
}

fn restore_zathura_fs(
    qrx: &mut QrexecServer::<KIB64>,
    dir_path: &String,
) -> DRes<()> {

    return Ok(());
}
