//! Main TUN device implementation.
//! Does not include the code for writing to and reading from the TUN device.
//! For that go to [`queue`](mod@crate::queue).

use std::{
    ffi::{CStr, CString},
    io, mem,
    net::IpAddr,
    os::fd::{AsFd, AsRawFd, FromRawFd, OwnedFd},
};

use cidr::IpCidr;

use super::queue::Queue;
use crate::builder::{Config, Device as D};

/// A TUN Device.
/// Create with [`DeviceBuilder`]
#[derive(Debug)]
pub struct Device {
    name: CString,
    queues: Vec<OwnedFd>,
    ctl: OwnedFd,
    no_pi: bool,
}
impl D for Device {
    /// This function is private.
    /// Use [`DeviceBuilder`] to create a new device.
    fn new(config: Config) -> io::Result<Self> {
        let name = match config.name.clone() {
            Some(name) => {
                if name.as_bytes_with_nul().len() > libc::IFNAMSIZ {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "interface name too long",
                    ));
                }
                Some(name)
            }
            None => None,
        };

        let num_queues = config.num_queues.unwrap_or(1);
        if num_queues < 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "number of queues must be at least 1",
            ));
        }
        let mut queues = Vec::new();

        // Create a new empty interface request
        let mut ifr: libc::ifreq = unsafe { mem::zeroed() };

        // Copy the name and leave the rest of the array as 0 (nul)
        if let Some(name) = name {
            let count = name.as_bytes().len();
            unsafe {
                std::ptr::copy_nonoverlapping(name.into_raw(), ifr.ifr_name.as_mut_ptr(), count)
            };
        }

        let mut flags = 0;
        flags |= libc::IFF_TUN as i16;
        if config.no_pi {
            flags |= libc::IFF_NO_PI as i16;
        }
        if config.multi_queue.unwrap_or(false) {
            flags |= libc::IFF_MULTI_QUEUE as i16;
        }

        ifr.ifr_ifru.ifru_flags = flags;

        unsafe {
            for _ in 0..num_queues {
                // Open the TUN device
                #[cfg(target_os = "linux")]
                let result = libc::open(b"/dev/net/tun\0".as_ptr().cast(), libc::O_RDWR);
                // Open the given TUN device file descriptor
                #[cfg(target_os = "android")]
                let result = libc::dup(config.raw_fd.unwrap_or(-1));

                if result < 0 {
                    return Err(io::Error::last_os_error());
                }
                let fd = OwnedFd::from_raw_fd(result);

                if libc::ioctl(fd.as_raw_fd(), TUNSETIFF, &mut ifr as *mut _) < 0 {
                    libc::close(fd.as_raw_fd());

                    return Err(io::Error::last_os_error());
                }

                queues.push(fd);
            }
        }

        let name = unsafe { CStr::from_ptr(ifr.ifr_name.as_ptr()) };

        let ctl = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
        if ctl < 0 {
            return Err(io::Error::last_os_error());
        }
        let ctl = unsafe { OwnedFd::from_raw_fd(ctl) };

        let device = Self {
            name: name.into(),
            queues,
            ctl,
            no_pi: config.no_pi,
        };
        device.configure(&config)?;

        Ok(device)
    }
}

impl Device {
    fn configure(&self, config: &Config) -> io::Result<()> {
        if let Some(address) = config.address {
            self.set_address(address)?;
        }

        if let Some(netmask) = config.netmask {
            self.set_netmask(netmask)?;
        }

        if config.up {
            self.bring_up()?;
        }

        Ok(())
    }

    unsafe fn request(&self) -> libc::ifreq {
        let mut req: libc::ifreq = mem::zeroed();

        req.ifr_name[..self.name.as_bytes().len()]
            .copy_from_slice(bytes_to_signed(self.name.as_bytes()));

        req
    }

    /// Enables (brings up) the device.
    pub fn bring_up(&self) -> io::Result<()> {
        unsafe {
            let mut ifr = self.request();

            if libc::ioctl(self.ctl.as_raw_fd(), libc::SIOCGIFFLAGS, &mut ifr) < 0 {
                return Err(io::Error::last_os_error());
            }

            ifr.ifr_ifru.ifru_flags |= (libc::IFF_UP | libc::IFF_RUNNING) as i16;

            if libc::ioctl(self.ctl.as_raw_fd(), libc::SIOCSIFFLAGS, &ifr) < 0 {
                return Err(io::Error::last_os_error());
            }

            Ok(())
        }
    }

    /// Sets the destination address of the device.
    pub fn set_address(&self, address: IpAddr) -> io::Result<()> {
        let mut ifr = unsafe { self.request() };

        ifr.ifr_ifru.ifru_addr = ip_to_sockaddr(address);

        unsafe {
            if libc::ioctl(self.ctl.as_raw_fd(), libc::SIOCSIFADDR, &ifr) < 0 {
                return Err(io::Error::last_os_error());
            }
        }

        Ok(())
    }

    /// Sets the netmask of the device.
    pub fn set_netmask(&self, network: IpCidr) -> io::Result<()> {
        let mut ifr = unsafe { self.request() };

        ifr.ifr_ifru.ifru_addr = ip_to_sockaddr(network.mask());

        unsafe {
            if libc::ioctl(self.ctl.as_raw_fd(), libc::SIOCSIFNETMASK, &ifr) < 0 {
                return Err(io::Error::last_os_error());
            }
        }

        Ok(())
    }

    /// Creates a queue and returns the index it was placed at.
    /// This doesn't need to be called if the device already has the queue you need.
    pub fn create_queue(&mut self) -> io::Result<usize> {
        let mut flags = libc::IFF_MULTI_QUEUE as i16;
        flags |= libc::IFF_TUN as i16;
        if self.no_pi {
            flags |= libc::IFF_NO_PI as i16;
        }

        unsafe {
            let mut interface_request = self.request();
            interface_request.ifr_ifru.ifru_flags = flags;

            let result = libc::open(b"/dev/net/tun\0".as_ptr().cast(), libc::O_RDWR);
            if result < 0 {
                return Err(io::Error::last_os_error());
            }
            let fd = OwnedFd::from_raw_fd(result);

            if libc::ioctl(fd.as_raw_fd(), TUNSETIFF, &mut interface_request as *mut _) < 0 {
                libc::close(fd.as_raw_fd());

                return Err(io::Error::last_os_error());
            }

            self.queues.push(fd);
        }

        Ok(self.queues.len() - 1)
    }

    /// Close a queue at the given index
    /// Do not call if you are using the queue in another thread as this will invalidate the file descriptor.
    pub fn close_queue(&mut self, index: usize) -> io::Result<()> {
        if index >= self.queues.len() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "queue at index {index} not found",
            ));
        }

        let fd = self.queues.remove(index);

        unsafe {
            if libc::close(fd.as_raw_fd()) < 0 {
                return Err(io::Error::last_os_error());
            };
        }

        Ok(())
    }

    /// Gets a **blocking** queue from the device
    /// Errors if there is no queue at the given index
    pub fn queue(&self, index: usize) -> io::Result<Queue<true>> {
        Ok(
            match self.queues.get(index).map(|fd| Queue::new(fd.as_fd())) {
                Some(queue) => queue?,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "queue {index} not found",
                    ))
                }
            },
        )
    }

    /// Gets a nonblocking queue from the device
    /// Errors if there is no queue at the given index
    pub fn queue_nonblocking(&self, index: usize) -> io::Result<Queue<false>> {
        Ok(
            match self.queues.get(index).map(|fd| Queue::new(fd.as_fd())) {
                Some(queue) => queue?,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "queue {index} not found",
                    ))
                }
            },
        )
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            for queue in &self.queues {
                libc::close(queue.as_raw_fd());
            }
            libc::close(self.ctl.as_raw_fd());
        }
    }
}

fn bytes_to_signed(val: &[u8]) -> &[i8] {
    unsafe { mem::transmute(val) }
}

fn ip_to_sockaddr(addr: IpAddr) -> libc::sockaddr {
    unsafe {
        match addr {
            IpAddr::V4(addr) => {
                let addr = libc::sockaddr_in {
                    sin_family: libc::AF_INET as u16,
                    sin_port: 0,
                    sin_addr: libc::in_addr {
                        s_addr: u32::from_ne_bytes(addr.octets()),
                    },
                    sin_zero: [0; 8],
                };
                mem::transmute(addr)
            }

            IpAddr::V6(_addr) => {
                todo!()
            }
        }
    }
}

const TUNSETIFF: libc::c_ulong = ioctl_sys::iow!(b'T', 202, mem::size_of::<libc::c_int>()) as _;
