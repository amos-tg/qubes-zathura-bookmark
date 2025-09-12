use std::error::Error;

pub type DRes<T> = Result<T, Box<dyn Error>>;


// ~~~~~~~ COMMUNICATION SEQUENCES ~~~~~~~ //
//
// <num_reads> = 32 bit / four byte sequence
//               this is not converted to &str
//               but is taken as a single num

// client request
pub const GET_BOOKNAMES: &[u8] = b"booknames;";
//
// server response
// <num_reads>;<bookname>;<bookname>;...
//
// client acknowledgment
// RECV_SEQ
//
// while (num_reads indicates more)  {
//  server response 
//  <bookname>;<bookname>; ...
//  client ack
//  RECV_SEQ
// }

// client request
pub const VAR_GET_BOOK: &[u8] = b"getbook:";//<bookname>;
// server response
// <num_reads>;<book_content>
// client acknowledgment 
// RECV_SEQ
//
// while (num_reads indicates more) {
//  server response
//  <book_content>
//  client ack
//  RECV_SEQ
// }

// client request
pub const VAR_SEND_SFILE: &[u8] = b"sfile:";//<sfilename>:<num_reads>;<sfile_contents>
// server acknowledgment                                            
// &[RECV_SEQ]


// client request
pub const GET_SFILES: &[u8] = b"initsfs;";
// server response 
pub const VAR_SEND_NUM_SFILES: &[u8] = b"numfiles:";//<num_sfiles>;
// client acknowledgment
// &[RECV_SEQ]
//
// loop over <num_sfiles> {
//
// server response
// VAR_SEND_SFILE
//
// client acknowledgment 
// &[RECV_SEQ]
//
// }



pub const CONF_PATH: &str = 
    "/etc/qubes-zathura-bookmark/qzb.conf";
pub const KIB64: usize = 65536;
pub const BLEN: usize = KIB64 - 8;
pub const RECV_SEQ: &[u8] = &[1];
pub const NONE: &[u8] = b"Nothing;";
pub const ZATHURA_PATH_POSTFIX: &str = ".local/share/zathura";
pub const HISTORY_FNAME: &str = "history"; 
pub const INPUT_HISTORY_FNAME: &str = "input-history";
pub const BMARKS_FNAME: &str = "bookmarks";
pub const FILES: [&str; 3] = 
    [BMARKS_FNAME, INPUT_HISTORY_FNAME, HISTORY_FNAME];

pub type RBUF = [u8;BLEN];

pub const RECV_SEQ_ERR: &str = 
    "Error: read byte did not match the RECV_SEQ sequence";
pub const WBYTES_NE_LEN_ERR: &str = 
    "Error: bytes written did not equal len of msg"; 
pub const NAME_DELIM_ERR: &str = 
    "Error: read bytes don't contain ';' delim";
pub const HOME_VAR_ERR: &str = 
    "Error: HOME env var is not present";
pub const MSG_FORMAT_ERR: &str = 
    "Error: the read bytes have incorrect formatting";
pub const BNAME_NE_SIZE_ERR: &str = 
    "Error: the number of booknames != sent value";
pub const MSG_LEN_WBUF_ERR: &str = 
    "Error: the length written over qrx cannot exceed WBUF_LEN.";
pub const CONF_EXISTS_ERR: &str = 
    "Error: the configuration file does not exist";
