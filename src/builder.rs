use std::ffi::CString;

use crate::Device;

pub struct DeviceBuilder {
    pub name: Option<CString>,
    pub num_queues: Option<usize>,
    pub(crate) multi_queue: Option<bool>,
    pub no_pi: bool,
    pub address: Option<std::net::IpAddr>,
    pub netmask: Option<cidr::IpCidr>,
    pub up: bool,
}

impl DeviceBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            num_queues: None,
            multi_queue: None,
            no_pi: true,
            address: None,
            netmask: None,
            up: true,
        }
    }

    pub fn with_name(mut self, name: CString) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_pi(mut self, pi: bool) -> Self {
        self.no_pi = !pi;
        self
    }

    pub fn with_address(mut self, address: std::net::IpAddr) -> Self {
        self.address = Some(address);
        self
    }

    pub fn with_netmask(mut self, netmask: cidr::IpCidr) -> Self {
        self.netmask = Some(netmask);
        self
    }

    pub fn with_num_queues(mut self, num_queues: usize) -> Self {
        if num_queues < 1 {
            panic!("number of queues must be at least 1")
        }
        self.num_queues = Some(num_queues);
        self.multi_queue = if num_queues > 1 {
            Some(true)
        } else {
            Some(false)
        };
        self
    }

    pub fn with_up(mut self, up: bool) -> Self {
        self.up = up;
        self
    }

    pub fn build(self) -> std::io::Result<Device> {
        Device::new(self)
    }
}
