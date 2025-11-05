use crate::{
    shared_consts::*,
    shared_fn::*,
    conf::Conf,
    recv_seq,
};
use std::{
    error::Error,
    fs::{self, FileType},
    path::{PathBuf, Path},
    num::TryFromIntError,
};
use qrexec_binds::{QrexecServer, QIO};
use anyhow::anyhow;

pub fn server_main(conf: Conf) -> DRes<()> {
    let mut buf = [0u8; BLEN];
    let mut qrx = QrexecServer::new();
    loop {
        request_handler(&mut qrx, &conf, &mut buf)?;
    }
}

fn request_handler(
    qrx: &mut QrexecServer,
    conf: &Conf,
    rbuf: &mut [u8; BLEN],
) -> DRes<()> {
    let nb = qrx.read(rbuf)?;
    match rbuf {
        rbuf if rbuf.starts_with(VAR_SEND_SFILE) => 
            recv_file(qrx, conf, rbuf, nb)?,

        rbuf if rbuf.starts_with(VAR_GET_BOOK) => 
            send_book(qrx, conf, rbuf)?,

        rbuf if rbuf.starts_with(GET_SFILES) => 
            send_sfile_tree(qrx, conf, rbuf)?,

        rbuf if rbuf.starts_with(GET_BOOKNAMES) => 
            send_booknames(qrx, conf, rbuf)?,

        _ => unreachable!(),
    }
    
    return Ok(());
}

struct Responder {
    qrx: QrexecServer,
    buf: [u8; BLEN],
    cursor: usize,
}

impl Responder {
    fn new(qrx: QrexecServer) -> Self {
        Self { qrx, buf: [0u8; BLEN], cursor: 0 }
    }

    //data includes:
    //  number of reads, 
    //  payload, 

    /// initially the cursor of the Responder object is 
    /// set to the number of bytes from the first read.
    fn poll_send(&mut self, conf: &Conf) -> DRes<()> {
        self.cursor = self.qrx.read(&mut self.buf)?;
        match self.buf[0] {
            VAR_SEND_SFILE => 
                recv_file(qrx, conf, rbuf, nb)?,

            VAR_GET_BOOK => 
                send_book(qrx, conf, rbuf)?,

            GET_SFILES => 
                send_sfile_tree(qrx, conf, rbuf)?,

            BookNames::ID => BookNames::send(
                &mut self, &conf)?,

            _ => unreachable!(),
        }

        return Ok(());
    }  
}

// associated ID forces conformity in impl
trait Response<const ID: u8> {
    fn send(tx: &mut Responder, conf: &Conf) -> DRes<()> {
        let cont = Self::contents(conf, tx)?;
        let mut num_reads = Self::set_numreads(tx, cont.len())?;
        let mut max_cursor;
        while num_reads != 0 {
            if num_reads == 1 {
                max_cursor = cont.len();
            } else {
                max_cursor = BLEN;
            }

            tx.cursor += set_slice(
                &mut tx.buf[tx.cursor..],
                &cont[..max_cursor]);

            tx.qrx.write(&tx.buf[..tx.cursor])?;
            tx.cursor = 0;

            num_reads -= 1;
        }

        return Ok(());
    }

    /// returns the number of reads, sets buf header
    /// with the number of reads required for a response
    /// of msg_len length. Sets the cursor to point to 
    /// next available byte.
    fn set_numreads(
        tx: &mut Responder,
        msg_len: usize,
    ) -> Result<u32, TryFromIntError> {
        let (nrb, nr) = num_reads_encode(msg_len)?;
        tx.cursor += set_slice(&mut tx.buf, &nrb); 
        return Ok(nr);
    }

    fn contents(conf: &Conf, tx: &mut Responder) -> DRes<Vec<u8>>;
}

struct BookNames;
impl BookNames {
    const ID: u8 = b'0';
}

impl Response<{Self::ID}> for BookNames {
    fn contents(conf: &Conf, _: &mut Responder) -> DRes<Vec<u8>> {
        let mut bnames: Vec<u8> = vec!();
        let bdir_entries = fs::read_dir(&conf.book_dir)?;
        for bentry in bdir_entries {
            bnames.extend_from_slice(
                bentry?.file_name()
                    .to_str() 
                    .ok_or(anyhow!(INVALID_ENC_ERR))?
                    .as_bytes());
            bnames.push(b';');
        }

        if bnames.is_empty() {
            bnames.push(NONE);
        }

        return Ok(bnames);
    }
}

struct Book;
impl Book {
    const ID: u8 = b'2';
}

impl Response<{Book::ID}> for Book {
    fn contents(conf: &Conf, tx: &mut Responder) -> DRes<Vec<u8>> {
        tx.buf
         
        return Ok();
    }
} 

fn send_book(
    qrx: &mut QrexecServer,
    conf: &Conf,
    rbuf: &mut [u8; BLEN],
) -> DRes<()> {
    let delim1 = find_delim(rbuf, b':')
        .ok_or(anyhow!(MSG_FORMAT_ERR))?;
    let delim2 = find_delim(rbuf, b';') 
        .ok_or(anyhow!(MSG_FORMAT_ERR))?;

    let bname = str::from_utf8(&rbuf[(delim1 + 1)..delim2])?;
    let bpath = find_book(Path::new(&conf.book_dir), bname)?;

    send_file(qrx, &bpath, rbuf, false)?;

    return Ok(());
}

fn find_book(book_dir: &Path, bname: &str) -> DRes<PathBuf> {
    let path = book_dir.join(bname).to_owned();
    if fs::exists(&path)? {
        return Ok(path);
    } else {
        for file in fs::read_dir(book_dir)? {
            let file = file?;
            if file.file_type()?.is_dir() {
                return find_book(&file.path(), bname);
            } 
        }
        return Err(anyhow!(BOOK_UNAVAILABLE_ERR))?;
    }                
}

fn send_sfile_tree(
    qrx: &mut QrexecServer,
    conf: &Conf,
    rbuf: &mut [u8; BLEN],
) -> DRes<()> {
    let mut file_paths: Vec<(PathBuf, FileType)> = vec!();
    let mut cursor = 0usize;

    recurse_files(
        fs::read_dir(&conf.state_dir)?,
        &mut file_paths)?;

    if file_paths.is_empty() {
        qrx.write(NONE)?;
        recv_seq!(qrx, rbuf);
        return Ok(());
    }

    // this is wrong 
    let num_files_bytes = u32::to_ne_bytes(file_paths.len().try_into()?);

    cursor += set_slice(rbuf, VAR_SEND_NUM_SFILES);
    cursor += set_slice(&mut rbuf[cursor..], &num_files_bytes);

    qrx.write(&rbuf[..cursor])?;
    recv_seq!(qrx, rbuf);

    for (fpath, ftype) in file_paths {
        let ftype = ftype.is_dir();
        send_file(qrx, fpath.as_path(), rbuf, ftype)?;
    }

    return Ok(());
}

fn recurse_files(
    read_dir: fs::ReadDir,
    files: &mut Vec<(PathBuf, FileType)>,
) -> DRes<()> {
    for file in read_dir {
        let file = file?;
        let path = file.path();
        let file_type = file.file_type()?;

        if file_type.is_file() {
            files.push((path, file_type));
        } else if file_type.is_dir() {
            recurse_files(
                fs::read_dir(&path)?,
                files)?;
            files.push((path, file_type));
        } else if file_type.is_symlink() {
            Err(anyhow!(SYMLINK_ERR))?;
        }
    }
    
    return Ok(());
}
