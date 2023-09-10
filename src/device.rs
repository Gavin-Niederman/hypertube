use std::{
    ffi::{CStr, CString},
    io, mem,
    net::IpAddr,
    os::fd::{AsFd, AsRawFd, FromRawFd, OwnedFd},
};

use cidr::IpCidr;

use crate::{builder::DeviceBuilder, queue::Queue};

#[derive(Debug)]
pub struct Device {
    name: CString,
    queues: Vec<OwnedFd>,
    ctl: OwnedFd,
}

type Config = DeviceBuilder;

impl Device {
    /// This function is private. Use [`DeviceBuilder`](crate::builder::DeviceBuilder) to create a new device.
    pub(crate) fn new(config: Config) -> io::Result<Self> {
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

        let mut interface_request: libc::ifreq = unsafe { mem::zeroed() };

        // Copy the name and leave the rest of the array as 0 (nul)
        if let Some(name) = name {
            let count = name.as_bytes().len();
            unsafe {
                std::ptr::copy_nonoverlapping(
                    name.into_raw(),
                    interface_request.ifr_name.as_mut_ptr(),
                    count,
                )
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

        interface_request.ifr_ifru.ifru_flags = flags;

        unsafe {
            for _ in 0..num_queues {
                let result = libc::open(b"/dev/net/tun\0".as_ptr().cast(), libc::O_RDWR);
                if result < 0 {
                    return Err(io::Error::last_os_error());
                }
                let fd = OwnedFd::from_raw_fd(result);

                if libc::ioctl(fd.as_raw_fd(), TUNSETIFF, &mut interface_request as *mut _) < 0 {
                    libc::close(fd.as_raw_fd());

                    return Err(io::Error::last_os_error());
                }

                queues.push(fd);
            }
        }

        let name = unsafe { CStr::from_ptr(interface_request.ifr_name.as_ptr()) };

        let ctl = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
        if ctl < 0 {
            return Err(io::Error::last_os_error());
        }
        let ctl = unsafe { OwnedFd::from_raw_fd(ctl) };

        let device = Self {
            name: name.into(),
            queues,
            ctl,
        };
        device.configure(&config)?;

        Ok(device)
    }

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

    pub fn set_enabled(&mut self, enabled: bool) -> io::Result<()> {
        unsafe {
            let mut ifr = self.request();

            if enabled {
                ifr.ifr_ifru.ifru_flags |= (libc::IFF_UP | libc::IFF_RUNNING) as i16;
            } else {
                ifr.ifr_ifru.ifru_flags &= !(libc::IFF_UP as i16);
            }

            if libc::ioctl(self.ctl.as_raw_fd(), libc::SIOCSIFFLAGS, &ifr) < 0 {
                return Err(io::Error::last_os_error());
            }

            Ok(())
        }
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
