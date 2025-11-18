use std::error::Error;

pub type DRes<T> = Result<T, Box<dyn Error>>;


// ~~~~~~~ COMMUNICATION SEQUENCES ~~~~~~~ //
//
// <num_reads> = 32 bit / four byte sequence
//               this is not converted to &str
//               but is taken as a single num
//
// all the recv functions are a result of 
// a request having been read and matched 
// to one of the protocols below, as a result
// of this pre-req, the recv functions assume 
// that rbuf (read buffer) already contains 
// the contents of the first read call which 
// was used to match the request.
//

// client request
pub const GET_BOOKNAMES: &[u8] = b"0";
//
// server response
// <num_reads>;<bookname>;<bookname>;...
//
// or 
//
pub const NONE: u8 = b'1';
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
pub const VAR_GET_BOOK: &[u8] = b"2";//<bookname>;
//                                            
// server response
// <num_reads>;<book_content>
//
// client acknowledgment 
// RECV_SEQ
//
// while (num_reads indicates more) {
//  server response
//  <book_content>
//
//  client ack
//  RECV_SEQ
// }

// client request
pub const VAR_SEND_SFILE: &[u8] = b"3";//<sfilename>:<num_reads>:<is_dir>;<sfile_contents>

// server acknowledgment 
// RECV_SEQ

// loop (while num_reads indicates) {
// 
// client response
// <sfile_contents>
//      
// server ack
// RECV_SEQ
//
// }


// client request
pub const GET_SFILES: &[u8] = b"4";

// server response 
pub const VAR_SEND_NUM_SFILES: &[u8] = b"5";//<num_sfiles>

// client acknowledgment
// RECV_SEQ
//
// loop over <num_sfiles> {
//
// server response
// VAR_SEND_SFILE
//
// client acknowledgment 
// RECV_SEQ
//
// }

// zathura notification message
pub const ZBOOK_READ_NOTIFY: &[u8] = b"6";//<book_name>
// client acknowledgement
// RECV_SEQ
//
// client requests book from server using
// VAR_GET_BOOK message sequence detailed above

pub const CONF_PATH: &str = 
    "/etc/qubes-zathura-bookmark/qzb.conf";
pub const KIB64: usize = 65536;
pub const BLEN: usize = KIB64 - 8;
pub const RECV_SEQ: &[u8] = &[1];
pub const CLIENT_ZATH_SOCK_PATH: &str = "/tmp/qubes_zath.sock";
pub const NUM_READS_LEN: usize = 4;

pub const RECV_SEQ_ERR: &str = 
    "Error: read byte did not match the RECV_SEQ sequence";
pub const MSG_FORMAT_ERR: &str = 
    "Error: the read bytes have incorrect formatting";
pub const MSG_LEN_WBUF_ERR: &str = 
    "Error: the length written over qrx cannot exceed\
    WBUF_LEN.";
pub const CONF_EXISTS_ERR: &str = 
    "Error: the configuration file does not exist";
pub const MISSING_BASENAME_ERR: &str = 
    "Error: the path doesn't contain a basename";
pub const INVALID_ENC_ERR: &str = 
    "Error: the OsStr did not yield a utf8 string";
pub const SYMLINK_ERR: &str = 
    "Error: recurse_files found a symlink, logic for\
    handling symlinks is not defined.";
pub const BOOK_UNAVAILABLE_ERR: &str = 
    "Error: the book does not exist in the configured\
    book directory";
pub const INVALID_MODEL_ERR: &str = 
    "Error: the model (client/server, identifier) is invalid";
