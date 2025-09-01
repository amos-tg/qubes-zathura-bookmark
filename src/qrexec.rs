use std::{
    io::{Read, Write, self},
    process::{
        Command,
        Child,
        ChildStdout,
        ChildStdin,
        ChildStderr,
    },
};
use crate::DRes;
use anyhow::anyhow;

pub struct Qrexec { 
    pub child: Child,
    pub stdout: ChildStdout,
    pub stdin: ChildStdin,
    pub _stderr: ChildStderr,
}

impl Qrexec {
    pub fn new(args: &[&str]) -> DRes<Self> {
        const STDOUT_ERR: &str = 
            "Error: child proc failed to produce stdout";
        const STDIN_ERR: &str = 
            "Error: child proc failed to produce stdin";
        const STDERR_ERR: &str =
            "Error: child proc failed to produce stderr";

        let mut child = Command::new("qrexec-client-vm")
            .args(args)
            .spawn()?;
        return Ok(Self {
            stdout: child.stdout.take().ok_or(
                anyhow!(STDOUT_ERR))?,
            stdin: child.stdin.take().ok_or(
                anyhow!(STDIN_ERR))?,
            _stderr: child.stderr.take().ok_or(
                anyhow!(STDERR_ERR))?,
            child,
        })
    }

    /// returns the number of bytes read into the buffer, 
    /// retries the read once on interruption io::Error before returning.
    #[inline(always)]
    pub fn read(
        mut read: impl Read,
        buf: &mut [u8],
    ) -> Result<usize, io::Error> {
        match read.read(buf) {
            Ok(nb) => Ok(nb),
            Err(e) if e.kind() == 
                io::ErrorKind::Interrupted => read.read(buf),
            Err(e) => Err(e),
        }
    } 

    /// returns the number of bytes written into the buffer,
    /// retries the read once on interruption io::Error before returning.
    #[inline(always)]
    pub fn write(
        mut written: impl Write,
        buf: &[u8],
    ) -> Result<usize, io::Error> {
        match written.write(buf) {
            Ok(nb) => Ok(nb),
            Err(e) if e.kind() == 
                io::ErrorKind::Interrupted => written.write(buf),
            Err(e) => Err(e),
        }
    }
}

impl Drop for Qrexec {
    fn drop(&mut self) {
        let _ = self.child.kill();   
    }
}
