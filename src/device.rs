use std::{
    ffi::CString,
    io::{self, Read, Write},
    mem,
    net::IpAddr,
    os::fd::{AsFd, AsRawFd, FromRawFd, OwnedFd},
};

use cidr::IpCidr;

use crate::{config::Config, queue::Queue};

#[derive(Debug)]
pub struct Device {
    name: CString,
    queues: Vec<OwnedFd>,
    ctl: OwnedFd,
}

impl Device {
    /// Creates a new Device.
    /// Errors if name is too long.
    pub fn new(config: Config) -> io::Result<Self> {
        let name = config.name.unwrap_or_default();
        if name.as_bytes_with_nul().len() > libc::IFNAMSIZ {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "interface name too long",
            ));
        }

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
        interface_request.ifr_name[..name.as_bytes().len()]
            .copy_from_slice(bytes_to_signed(name.as_bytes()));

        interface_request.ifr_ifru.ifru_flags = libc::IFF_TUN as i16
            | (if num_queues > 1 {
                libc::IFF_MULTI_QUEUE
            } else {
                0
            } | if config.no_pi { libc::IFF_NO_PI } else { 0 }) as i16; // 0x1001

        unsafe {
            for _ in 0..num_queues {
                let result = libc::open(
                    b"/dev/net/tun\0".as_ptr().cast(),
                    libc::O_RDWR,
                );
                if result < 0 {
                    return Err(io::Error::last_os_error());
                }
                let fd = OwnedFd::from_raw_fd(result);

                if libc::ioctl(fd.as_raw_fd(), TUNSETIFF, &mut interface_request) < 0 {
                    libc::close(fd.as_raw_fd());

                    return Err(io::Error::last_os_error());
                }

                queues.push(fd);
            }
        }

        let ctl = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
        if ctl < 0 {
            return Err(io::Error::last_os_error());
        }
        let ctl = unsafe { OwnedFd::from_raw_fd(ctl) };

        Ok(Self { name, queues, ctl })
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

    pub fn queue(&self, index: usize) -> Option<Queue> {
        self.queues.get(index).map(|fd| Queue::new(fd.as_fd()))
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
            IpAddr::V4(addr) => mem::transmute(libc::sockaddr_in {
                sin_family: libc::AF_INET as u16,
                sin_port: 0,
                sin_addr: libc::in_addr {
                    s_addr: u32::from_ne_bytes(addr.octets()),
                },
                sin_zero: [0; 8],
            }),
            IpAddr::V6(addr) => todo!(),
        }
    }
}

const TUNSETIFF: libc::c_ulong = ioctl_sys::iow!(b'T', 202, mem::size_of::<libc::c_int>()) as _;
