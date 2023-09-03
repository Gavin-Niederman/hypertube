use std::{
    io,
    os::fd::{AsRawFd, BorrowedFd}, task::Poll,
};

#[derive(Debug)]
pub struct Queue<'a, const BLOCKING: bool> {
    fd: BorrowedFd<'a>,
}

impl<'a, const BLOCKING: bool> Queue<'a, BLOCKING> {
    pub fn new(fd: BorrowedFd<'a>) -> Self {
        Self { fd }
    }

    pub fn set_blocking(&self, blocking: bool) -> io::Result<()> {
        let mut flags = unsafe { libc::fcntl(self.as_raw_fd(), libc::F_GETFL) };
        if blocking {
            flags &= !libc::O_NONBLOCK;
        } else {
            flags |= libc::O_NONBLOCK;
        }

        if unsafe { libc::fcntl(self.as_raw_fd(), libc::F_SETFL, flags) } < 0 {
            return Err(io::Error::last_os_error())
        }
        Ok(())
    }
}

impl<'a> Queue<'a, true> {
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
}

impl<'a> Queue<'a, false> {
    pub fn read(&self, buf: &mut [u8]) -> io::Result<Poll<usize>> {
        unsafe {
            let len = libc::read(self.fd.as_raw_fd(), buf.as_mut_ptr().cast(), buf.len());
            if len < 0 {
                let error = io::Error::last_os_error();
                match error.raw_os_error().unwrap_or(0) {
                    libc::EWOULDBLOCK => return Ok(Poll::Pending),
                    _ => return Err(error),
                }
            }

            Ok(Poll::Ready(len as usize))
        }
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<Poll<()>> {
        unsafe {
            let result = libc::write(self.fd.as_raw_fd(), buf.as_ptr().cast(), buf.len());

            if result < 0 {
                return Err(io::Error::last_os_error());
            }

            Ok(Poll::Ready(()))
        }
    }
}

impl<const BLOCKING: bool> AsRawFd for Queue<'_, BLOCKING> {
    fn as_raw_fd(&self) -> std::os::fd::RawFd {
        self.fd.as_raw_fd()
    }
}