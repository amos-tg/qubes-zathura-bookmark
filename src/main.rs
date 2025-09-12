mod req; 
mod conf;
mod shared_fn;
mod shared_consts;
mod client;
mod server;

use std::env::var;
use crate::{
    client::client_main,
    server::server_main,
    shared_consts::DRes,
};
use dbuggery::err_append;
use anyhow::anyhow;

const ERR_LOG_DIR_NAME: &str = "zathura-bookmark-service";
const ERR_FNAME: &str = "errors.log";

fn main() {
    let model = Model::new();
    err_append(
        &model,
        ERR_FNAME,
        ERR_LOG_DIR_NAME);
    let model = model.unwrap();

    match model {
        Model::Client => err_append(
            &client_main(),
            ERR_FNAME,
            ERR_LOG_DIR_NAME),
        Model::Server => err_append(
            &server_main(),
            ERR_FNAME,
            ERR_LOG_DIR_NAME),
    };
}

enum Model {
    Client,
    Server,
}

impl Model {
    fn new() -> DRes<Self> {
        const MODEL_IDENT_VAR: &str = "ZBMARK_MODEL";
        const INVALID_IDENT_VAR_ERR: &str = 
            "Error: identifier var != <client> or <server>.";

        let ident = var(MODEL_IDENT_VAR)?;
        match ident.as_str() {
            "client" => return Ok(Self::Client),
            "server" => return Ok(Self::Server),
            _ => Err(anyhow!(INVALID_IDENT_VAR_ERR))?,
        }
    }
}
