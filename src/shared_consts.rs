use std::error::Error;

pub type DRes<T> = Result<T, Box<dyn Error>>;

pub const KIB64: usize = 65536;
pub const ZATHURA_PATH_POSTFIX: &str = ".local/share/zathura";
pub const HISTORY_FNAME: &str = "history";
pub const INPUT_HISTORY_FNAME: &str = "input-history";
pub const RECV_SEQ: u8 = 1;
pub const BMARKS_FNAME: &str = "bookmarks";

pub const RECV_SEQ_ERR: &str = 
    "Error: read byte did not match the RECV_SEQ sequence";
pub const WBYTES_NE_LEN_ERR: &str = 
    "Error: bytes write did not equal len"; 
pub const NAME_DELIM_ERR: &str = 
    "Error: read bytes don't contain ';' delim";
pub const HOME_VAR_ERR: &str = "Error: HOME env var is not present";
