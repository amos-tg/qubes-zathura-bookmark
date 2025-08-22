use std::{
    env::var,
    sync::mpsc,
    path::Path,
};
use notify::{
    recommended_watcher,
    Watcher,
    RecursiveMode,
};

const ZATHURA_PATH_POSTFIX: &str =  
    ".local/share/zathura";

fn main() {
    let path_str = format!(
        "{}/{}",
        var("HOME").unwrap(),
        ZATHURA_PATH_POSTFIX,
    );
    let path = Path::new(&path_str);

    let (tx, rx) = mpsc::channel();
    let mut watcher = recommended_watcher(tx).unwrap();

    watcher.watch(&path, RecursiveMode::Recursive).unwrap();

}
