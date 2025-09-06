mod shared_fn;
mod shared_consts;
mod client;
mod server;

use std::env::{args, Args};
use crate::{
    client::client_main,
    server::server_main,
};
use dbuggery::err_append;

const ERR_LOG_DIR_NAME: &str = "zathura-bookmark-service";
const ERR_FNAME: &str = "errors.log";

fn main() {
    let model = Model::new(args());
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
    fn new(args: Args) -> Self {
        const ARG_ERR: &str = 
            "Error: incorrect number of args given please \
            pass in --server or --client to indicate behavior";

        if args.len() != 2 { panic!("{}", ARG_ERR) }
        if let Some(model) = args.skip(1).next() {
            match model.as_str() {
                "--server" => return Self::Server,
                "--client" => return Self::Client,
                _ => panic!("{}", ARG_ERR),
            }
        } else {
            panic!("{}", ARG_ERR);
        }
    }
}
