mod client;
mod server;

use std::{
    env::{args, Args},
    error::Error,
};
use crate::{
    client::client_main,
    server::server_main,
};

type DRes<T> = Result<T, Box<dyn Error>>;

fn main() {
    let model = Model::new(args());
    match model {
        Model::Client => client_main(),
        Model::Server => server_main(),
    }
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
