use std::{
    io,
    os::fd::{AsRawFd, BorrowedFd, RawFd},
    task::Poll,
};

/// A queue for a TUN device.
/// Used to read and write packets to said TUN device.
/// Const generic `BLOCKING` determines whether or not the queue blocks on reads and writes.
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
    /// Read a packet from the queue.
    /// Blocks until a packet is available.
    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        unsafe { read(self.fd.as_raw_fd(), buf) }
    }

    /// Write a packet to the queue.
    /// Blocks until the packet is written.
    pub fn write(&self, buf: &[u8]) -> io::Result<()> {
        unsafe { write(self.fd.as_raw_fd(), buf) }
    }
}

impl<'a> Queue<'a, false> {
    /// Read a packet from the queue.
    /// Returns [`Poll::Pending`](core::task::Poll::Pending) if no packet is available yet.
    pub fn read(&self, buf: &mut [u8]) -> io::Result<Poll<usize>> {
        unsafe {
            match read(self.fd.as_raw_fd(), buf) {
                Ok(len) => Ok(Poll::Ready(len as usize)),
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => Ok(Poll::Pending),
                Err(err) => Err(err),
            }
        }
    }

    /// Write a packet to the queue.
    /// Returns [`Poll::Pending`](core::task::Poll::Pending) if the has not been written yet.
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
