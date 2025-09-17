use std::{fs, io};
use crate::shared_consts::*;
use serde::{Serialize, Deserialize};
use serde_yaml;
use anyhow::anyhow;

#[derive(Debug, Serialize, Deserialize)]
pub struct Conf {
    pub state_dir: String,
    pub book_dir: String, 
    pub model: String,
    pub target_vm: String,
}

impl Conf {
    pub fn new() -> DRes<Self> {
        let conf_path = Self::path()?;
        let raw = fs::read_to_string(&conf_path)?;
        let conf: Conf = serde_yaml::from_str(&raw)?;
        Self::init_dirs(&conf)?;
        return Ok(conf);
    } 

    fn path() -> DRes<String> {
        if !fs::exists(CONF_PATH)? {
            Err(anyhow!(CONF_EXISTS_ERR))?
        } else {
            return Ok(CONF_PATH.to_owned());
        }
    }

    fn init_dirs(conf: &Conf) -> io::Result<()> {
        fs::create_dir_all(&conf.book_dir)?;
        fs::create_dir_all(&conf.state_dir)?;

        return Ok(()); 
    }
}
