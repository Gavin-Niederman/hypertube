use std::{
    io,
    os::fd::{AsRawFd, BorrowedFd},
};

#[derive(Debug)]
pub struct Queue<'a> {
    fd: BorrowedFd<'a>,
}

impl<'a> Queue<'a> {
    pub fn new(fd: BorrowedFd<'a>) -> Self {
        Self { fd }
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        unsafe {
            let len = libc::read(self.fd.as_raw_fd(), buf.as_mut_ptr().cast(), buf.len());
            if len < 0 {
                return Err(io::Error::last_os_error());
            }

            Ok(len as usize)
        }
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<()> {
        unsafe {
            let result = libc::write(self.fd.as_raw_fd(), buf.as_ptr().cast(), buf.len());

            if result < 0 {
                return Err(io::Error::last_os_error());
            }

            Ok(())
        }
    }

    pub fn set_blocking(&self, blocking: bool) -> io::Result<()> {
        let flags = unsafe { libc::fcntl(self.as_raw_fd(), libc::F_GETFL) | if blocking { 0 } else { libc::O_NONBLOCK } };
        if unsafe { libc::fcntl(self.as_raw_fd(), libc::F_SETFL, flags) } < 0 {
            return Err(io::Error::last_os_error())
        }
        Ok(())
    }
}

impl AsRawFd for Queue<'_> {
    fn as_raw_fd(&self) -> std::os::fd::RawFd {
        self.fd.as_raw_fd()
    }
}