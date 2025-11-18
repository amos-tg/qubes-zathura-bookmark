use crate::{
    recv_seq,
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
    let mut qx = Qmunnicate::new(qrx);
    loop {
        qx.server_recv(&conf)?; 
    }
}

struct Qmunnicate<T: QIO> {
    qrx: T,
    buf: [u8; BLEN],
    cursor: usize,
}

impl<T: QIO> Qmunnicate<T> {
    /// First available byte index in Responder.buf not 
    /// used by the header.
    const CONT_IDX: usize = 4;
    
    fn new(qrx: T) -> Self {
        Self { qrx, buf: [0u8; BLEN], cursor: 0 }
    }

    /// returns the number of reads, sets buf header
    /// with the number of reads required for a response
    /// of msg_len length. Sets the cursor to point to 
    /// next available byte.
    fn set_numreads(
        &mut self,
        msg_len: usize,
    ) -> Result<u32, TryFromIntError> {
        let (nrb, nr) = num_reads_encode(msg_len)?;
        self.cursor += set_slice(&mut self.buf[self.cursor..], &nrb); 
        return Ok(nr);
    }

    /// initially the cursor of the Responder object is 
    /// set to the number of bytes from the first read.
    fn server(&mut self, conf: &Conf) -> DRes<()> {
        self.cursor = self.qrx.read(&mut self.buf)?;
        match self.buf[0] {
            //VAR_SEND_SFILE => recv_file(qrx, conf, rbuf, nb)?,
            StateFiles::ID => StateFiles::send(self, &conf, None)?,

            Book::ID => Book::send(self, &conf, None)?,
            BookNames::ID => BookNames::send(self, &conf, None)?,

            _ => unreachable!(),
        }

        return Ok(());
    }  
}

fn inner_recv<T: QIO>(qc: &mut Qmunnicate<T>) -> DRes<Vec<u8>> {
    let mut content = vec!();
    qc.cursor = 0;

    qc.cursor = qc.qrx.read(&mut qc.buf)?;
    let mut num_reads = num_reads_decode(
        qc.buf[..NUM_READS_LEN].try_into()?);
    num_reads -= 1;
    content.push(&qc.buf[NUM_READS_LEN..qc.cursor]);
    qc.qrx.write(RECV_SEQ)?;

    while num_reads != 0 {
        qc.cursor = qc.qrx.read(&mut qc.buf)?;  
        qc.qrx.write(RECV_SEQ)?;
        content.extend_from_slice(qc.buf[..qc.cursor]);
        num_reads -= 1;
    }

    return Ok(content);
}

trait RecvOne<T: QIO> {
    // StateFiles:
    //  Decode and store server side,
    //  Decode and store client side,
    //  Requesting adaption (includes id) client side,
    //
    // BookNames:
    //  Request, client side
    //  storage, client side
    //
    // BookContent:
    //  Request, client side
    //  Storage, client side

    ///  I am aware this is not optimized.
    ///  I don't think that matters. 
    ///
    ///  The if statement is the simplest and 
    ///  fastest way to get this working. 
    ///
    /// how to recv Content::More
    /// how to recv Content::One
    ///
    /// use if statement
    ///
    /// Recieves the request on the client side 
    fn recv(qc: &mut Qmunnicate<T>, conf: &Conf) -> DRes<()> {
        let content = inner_recv(qc)?;
        Self::handle(conf, content)?;
        return Ok(());
    }

    fn handle(conf: &Conf, cont: Vec<u8>) -> DRes<()>; 
}

trait RecvMore<T: QIO> {
    fn recv_more<const ITERS: usize>(
        qc: &mut Qmunnicate<T>, conf: &Conf,
    ) -> DRes<()> { 
        let mut content = vec!();
        for _ in 0..ITERS {
            content.push(inner_recv(qc)?); 
        }

        Self::handle(conf, content)?;
        return Ok(());
    }

    fn handle(conf: &Conf, cont: Vec<Vec<u8>>) -> DRes<()>;
}

trait Send<T: QIO> {
    fn send(qc: &mut Qmunnicate<T>, conf: &Conf, identifier: Option<u8>) -> DRes<()> {
        let cont = Self::contents(conf, qc)?;
        if let Some(identifier) = identifier {
            qc.buf[0] = identifier;
            qc.cursor += 1;
        }

        match cont {
            Content::One(cont) => Self::send_one(qc, cont)?,
            Content::More(cont) => Self::send_more(qc, cont)?,
            Content::None => _ = qc.qrx.write(&qc.buf)?,
        }

        qc.cursor = 0;
        return Ok(());
    }

    fn send_one(qc: &mut Qmunnicate<T>, cont: Vec<u8>) -> DRes<()> {
        let mut num_reads = qc.set_numreads(cont.len())?;
        let mut max_cursor;
        while num_reads != 0 {
            if num_reads == 1 {
                max_cursor = cont.len();
            } else {
                max_cursor = BLEN;
            }

            qc.cursor += set_slice(
                &mut qc.buf[qc.cursor..],
                &cont[..max_cursor]);

            qc.qrx.write(&qc.buf[..qc.cursor])?;
            recv_seq!(qc.qrx, qc.buf);
            qc.cursor = 0;

            num_reads -= 1;
        }

        return Ok(());
    }

    fn send_more(
        qc: &mut Qmunnicate<T>,
        conts: Vec<Vec<u8>>,
    ) -> DRes<()> {
        for cont in conts {      
            Self::send_one(qc, cont)?;
        }

        return Ok(());
    }

    fn contents(conf: &Conf, qc: &mut Qmunnicate<T>) -> DRes<Content>;
}

struct BookNames;
impl BookNames {
    const ID: u8 = b'0';
}

impl<T: QIO> Send<T> for BookNames {
    fn contents(conf: &Conf, _: &mut Qmunnicate<T>) -> DRes<Content> {
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

impl<T: QIO> Recv<T> for BookNames {
    fn recv(qc: &mut Qmunnicate<T>, conf: &Conf) -> DRes<()> {

        return Ok(());
    }

}

struct Book;
impl Book {           
    const ID: u8 = b'2';

    fn find_book(
        book_dir: &Path,
        bname: &str,
    ) -> io::Result<Option<PathBuf>> {
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

impl Send<{Book::ID}> for Book {
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

impl Send<{Self::ID}> for StateFiles {
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
