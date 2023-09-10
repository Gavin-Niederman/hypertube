use std::{
    io,
    os::fd::{AsRawFd, BorrowedFd, RawFd},
    task::Poll,
};

#[derive(Debug)]
pub struct Queue<'a, const BLOCKING: bool> {
    fd: BorrowedFd<'a>,
}

impl<'a, const BLOCKING: bool> Queue<'a, BLOCKING> {
    pub(crate) fn new(fd: BorrowedFd<'a>) -> io::Result<Self> {
        let mut flags = unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_GETFL) };
        if BLOCKING {
            flags &= !libc::O_NONBLOCK;
        } else {
            flags |= libc::O_NONBLOCK;
        }

        if unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_SETFL, flags) } < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(Self { fd })
    }
}

unsafe fn read(fd: RawFd, buf: &mut [u8]) -> io::Result<usize> {
    let len = libc::read(fd, buf.as_mut_ptr().cast(), buf.len());
    if len < 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(len as usize)
}

unsafe fn write(fd: RawFd, buf: &[u8]) -> io::Result<()> {
    let result = libc::write(fd, buf.as_ptr().cast(), buf.len());
    if result < 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}

impl<'a> Queue<'a, true> {
    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        unsafe { read(self.fd.as_raw_fd(), buf) }
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<()> {
        unsafe { write(self.fd.as_raw_fd(), buf) }
    }
}

impl<'a> Queue<'a, false> {
    pub fn read(&self, buf: &mut [u8]) -> io::Result<Poll<usize>> {
        unsafe {
            match read(self.fd.as_raw_fd(), buf) {
                Ok(len) => Ok(Poll::Ready(len as usize)),
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => Ok(Poll::Pending),
                Err(err) => Err(err),
            }
        }
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<Poll<()>> {
        unsafe {
            match write(self.fd.as_raw_fd(), buf) {
                Ok(()) => Ok(Poll::Ready(())),
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => Ok(Poll::Pending),
                Err(err) => Err(err),
            }
        }
    }
}

impl<const BLOCKING: bool> AsRawFd for Queue<'_, BLOCKING> {
    fn as_raw_fd(&self) -> std::os::fd::RawFd {
        self.fd.as_raw_fd()
    }
}
