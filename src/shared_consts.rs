use std::error::Error;

pub type DRes<T> = Result<T, Box<dyn Error>>;

pub const KIB64: usize = 65536;
pub const WBUF_LEN: usize = KIB64 - 8;
pub const BLEN: usize = KIB64;
pub const RECV_SEQ: u8 = 1;
pub const NONE: &[u8] = b"Nothing;";
pub const ZATHURA_PATH_POSTFIX: &str = ".local/share/zathura";
pub const HISTORY_FNAME: &str = "history";
pub const INPUT_HISTORY_FNAME: &str = "input-history";
pub const BMARKS_FNAME: &str = "bookmarks";
pub const FILES: [&str; 3] = 
    [BMARKS_FNAME, INPUT_HISTORY_FNAME, HISTORY_FNAME];

pub const RECV_SEQ_ERR: &str = 
    "Error: read byte did not match the RECV_SEQ sequence";
pub const WBYTES_NE_LEN_ERR: &str = 
    "Error: bytes written did not equal len of msg"; 
pub const NAME_DELIM_ERR: &str = 
    "Error: read bytes don't contain ';' delim";
pub const HOME_VAR_ERR: &str = "Error: HOME env var is not present";
pub const MSG_FORMAT_ERR: &str = 
    "Error: the read bytes have incorrect formatting";
pub const BNAME_NE_SIZE_ERR: &str = 
    "Error: the number of booknames != sent value";
pub const MSG_LEN_WBUF_ERR: &str = 
    "Error: the length written over qrx cannot exceed WBUF_LEN.";
