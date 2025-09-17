#[cfg(test)]
mod test;

mod conf;
mod shared_fn;
mod shared_consts;
mod client;
mod server;

use crate::{
    client::client_main,
    server::server_main,
    shared_consts::*,
    conf::Conf,
};
use dbuggery::{err_append, append};

const ERR_LOG_DIR_NAME: &str = "zathura-bookmark-service";
const ERR_FNAME: &str = "errors.log";

fn main() {
    let conf = Conf::new();
    err_append(
        &conf,
        ERR_FNAME,
        ERR_LOG_DIR_NAME);
    let conf = conf.unwrap();

    match conf.model.as_str() {
        "client" => err_append(
            &client_main(conf),
            ERR_FNAME,
            ERR_LOG_DIR_NAME),
        "vault" => err_append(
            &server_main(conf),
            ERR_FNAME,
            ERR_LOG_DIR_NAME),
        _ => append(
            INVALID_MODEL_ERR,
            ERR_FNAME,
            ERR_LOG_DIR_NAME),
    };
}
