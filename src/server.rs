use crate::{
    shared_consts::*,
    shared_fn::*,
    conf::Conf,
};
use std::{
    io,
    fs::{self, FileType},
    path::{PathBuf, Path},
    num::TryFromIntError,
};
use qrexec_binds::{QrexecServer, QIO};
use anyhow::anyhow;

pub fn server_main(conf: Conf) -> DRes<()> {
    let qrx = QrexecServer::new();
    let mut responder = Responder::new(qrx);
    loop {
        responder.poll_send(&conf)?; 
    }
}

struct Responder {
    qrx: QrexecServer,
    buf: [u8; BLEN],
    cursor: usize,
}

impl Responder {
    /// First available byte index in Responder.buf not 
    /// used by the header.
    const CONT_IDX: usize = 4;
    
    fn new(qrx: QrexecServer) -> Self {
        Self { qrx, buf: [0u8; BLEN], cursor: 0 }
    }

    /// initially the cursor of the Responder object is 
    /// set to the number of bytes from the first read.
    fn poll_send(&mut self, conf: &Conf) -> DRes<()> {
        self.cursor = self.qrx.read(&mut self.buf)?;
        match self.buf[0] {
            // I'm going to use the function in request
            // on the client side for this so I don't write
            // the same thing twice. I will have to write this 
            // to parse the StateFiles::send functions response.
            // Ideally it should handle multiple changed files
            // anyways.
            //VAR_SEND_SFILE => recv_file(qrx, conf, rbuf, nb)?,

            Book::ID => Book::send(self, &conf)?,

            StateFiles::ID => StateFiles::send(self, &conf)?,

            BookNames::ID => BookNames::send(self, &conf)?,

            _ => unreachable!(),
        }

        return Ok(());
    }  
}

enum Content {
    One(Vec<u8>),
    More(Vec<Vec<u8>>),
    None,
}

trait Response<const ID: u8> {
    fn send(tx: &mut Responder, conf: &Conf) -> DRes<()> {
        let cont = Self::contents(conf, tx)?;
        return match cont {
            Content::One(cont) => Self::send_one(tx, cont),
            Content::More(cont) => Self::send_more(tx, cont),
            Content::None => Ok(()),
        };
    }

    fn send_more(tx: &mut Responder, conts: Vec<Vec<u8>>) -> DRes<()> {
        for cont in conts {      
            Self::send_one(tx, cont)?;
        }

        return Ok(());
    }

    fn send_one(tx: &mut Responder, cont: Vec<u8>) -> DRes<()> {
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

    fn contents(conf: &Conf, tx: &mut Responder) -> DRes<Content>;
}

struct BookNames;
impl BookNames {
    const ID: u8 = b'0';
}

impl Response<{Self::ID}> for BookNames {
    fn contents(conf: &Conf, _: &mut Responder) -> DRes<Content> {
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

        return Ok(Content::One(bnames));
    }
}

struct Book;
impl Book {
    const ID: u8 = b'2';

    fn find_book(book_dir: &Path, bname: &str) -> io::Result<Option<PathBuf>> {
        let path = book_dir.join(bname).to_owned();
        if fs::exists(&path)? {
            return Ok(Some(path));
        } else {
            for file in fs::read_dir(book_dir)? {
                let file = file?;
                if file.file_type()?.is_dir() {
                    return Self::find_book(&file.path(), bname);
                } 
            }
            return Ok(None);
        }                
    }
}

impl Response<{Book::ID}> for Book {
    /// returns the book cont if it exists in book dir, else
    /// returns an empty vector if it doesn't exist.
    fn contents(conf: &Conf, tx: &mut Responder) -> DRes<Content> {
        let bname = str::from_utf8(&tx.buf[1..tx.cursor])?;
        let bpath = Self::find_book(Path::new(&conf.book_dir), bname)?;
        if let Some(bpath) = bpath { 
            return Ok(Content::One(fs::read(&bpath)?));
        } else {
            return Ok(Content::None);
        }
    }
} 


struct StateFiles;
impl StateFiles {
    const ID: u8 = b'3'; 

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
                Self::recurse_files(
                    fs::read_dir(&path)?,
                    files)?;
                files.push((path, file_type));
            } else if file_type.is_symlink() {
                Err(anyhow!(SYMLINK_ERR))?;
            }
        }

        return Ok(());
    }
}

impl Response<{Self::ID}> for StateFiles {
    /// directories are denoted by zero contents, ie. the accompanying 
    /// vector of like ordered names will not have a content indice set
    /// Paths and Contents are sent in this order (the paths sent are 
    /// stripped of the prefix conf.state_dir): 
    /// * 1: Paths, 
    /// * 2: Contents,
    fn contents(conf: &Conf, _: &mut Responder) -> DRes<Content> {
        let mut file_paths: Vec<(PathBuf, FileType)> = vec!();
        let mut contents: Vec<u8> = vec!();
        let mut fpaths: Vec<u8> = vec!();

        Self::recurse_files(
            fs::read_dir(&conf.state_dir)?,
            &mut file_paths)?;

        if file_paths.is_empty() {
            return Ok(Content::None);
        }

        for (path, ftype) in file_paths {
            let fpath_bytes = path
                .as_path()
                .strip_prefix(&conf.state_dir)?
                .as_os_str()
                .as_encoded_bytes();

            if ftype.is_dir() {
                fpaths.extend_from_slice(fpath_bytes);
            } else if ftype.is_file() {
                fpaths.extend_from_slice(fpath_bytes);
                let fcont_bytes = fs::read(path)?;
                contents.extend_from_slice(&fcont_bytes);
            }
        }

        return Ok(Content::More(vec![fpaths, contents]));
    } 
}
