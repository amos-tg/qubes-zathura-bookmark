use std::{
    env::var,
    sync::mpsc,
    path::Path,
    fs,
};
use notify::{
    recommended_watcher,
    Watcher,
    RecursiveMode,
    EventKind,
};

const ZATHURA_PATH_POSTFIX: &str =  
    ".local/share/zathura";

pub fn client_main(){
    let path_str = format!(
        "{}/{}",
        var("HOME").unwrap(),
        ZATHURA_PATH_POSTFIX,
    );
    let path = Path::new(&path_str);

    let (tx, rx) = mpsc::channel();
    let mut watcher = recommended_watcher(tx).unwrap();

    watcher.watch(&path, RecursiveMode::Recursive).unwrap();
    loop {
        let event = rx.recv().unwrap().unwrap();
        match event.kind { 
            EventKind::Remove(_) => continue,
            EventKind::Access(_) => continue, 
            _ => (),
        }
        for path in event.paths {
            let fcont = fs::read_to_string(&path).unwrap();
        }
    } 
}
