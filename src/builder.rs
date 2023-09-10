//! Builder for [`Device`]

use std::ffi::CString;

use crate::Device;

pub(crate) struct Config {
    pub name: Option<CString>,
    pub num_queues: Option<usize>,
    pub(crate) multi_queue: Option<bool>,
    pub no_pi: bool,
    pub address: Option<std::net::IpAddr>,
    pub netmask: Option<cidr::IpCidr>,
    pub up: bool,
}

/// A builder for [`Device`]
pub struct DeviceBuilder {
    config: Config,
}

impl DeviceBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self {
            config: Config {
                name: None,
                num_queues: None,
                multi_queue: None,
                no_pi: true,
                address: None,
                netmask: None,
                up: true,
            },
        }
    }

    pub fn with_name(mut self, name: CString) -> Self {
        self.config.name = Some(name);
        self
    }

    /// Sets whether or not the device should have a packet information header.
    pub fn with_pi(mut self, pi: bool) -> Self {
        self.config.no_pi = !pi;
        self
    }

    pub fn with_address(mut self, address: std::net::IpAddr) -> Self {
        self.config.address = Some(address);
        self
    }

    pub fn with_netmask(mut self, netmask: cidr::IpCidr) -> Self {
        self.config.netmask = Some(netmask);
        self
    }

    /// Set the number of queues the device should have.
    /// If this is not set, the device will have 1 queue.
    /// ## Panics
    /// This function will panic if `num_queues` is less than 1.
    /// ### Note
    /// There is no way to increase the number of queues a [`Device`] has after it has been created yet.
    /// This should be fixed in the future.
    pub fn with_num_queues(mut self, num_queues: usize) -> Self {
        if num_queues < 1 {
            panic!("number of queues must be at least 1")
        }
        self.config.num_queues = Some(num_queues);
        self.config.multi_queue = if num_queues > 1 {
            Some(true)
        } else {
            Some(false)
        };
        self
    }

    /// Sets whether the [`Device`] should be up on creation or not.
    pub fn with_up(mut self, up: bool) -> Self {
        self.config.up = up;
        self
    }

    /// Builds the [`Device`].
    pub fn build(self) -> std::io::Result<Device> {
        Device::new(self.config)
    }
}

impl Default for DeviceBuilder {
    fn default() -> Self {
        Self::new()
    }
}
