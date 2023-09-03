use std::ffi::CString;

pub struct Config<const MQ: bool> {
    pub name: Option<CString>,
    pub num_queues: Option<usize>,
    pub no_pi: bool,
    pub address: Option<std::net::IpAddr>,
    pub netmask: Option<cidr::IpCidr>,
}

impl<const MQ: bool> Default for Config<MQ> {
    fn default() -> Self {
        Self {
            name: None,
            num_queues: if MQ {None} else {Some(1)},
            no_pi: true,
            address: None,
            netmask: None,
        }
    }
}

pub struct ConfigBuilder<const MQ: bool> {
    config: Config<MQ>,
}

impl<const MQ: bool> ConfigBuilder<MQ> {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    pub fn with_name(mut self, name: CString) -> Self {
        self.config.name = Some(name);
        self
    }

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

    pub fn build(self) -> Config<MQ> {
        self.config
    }
}

impl ConfigBuilder<true> {
    pub fn with_num_queues(mut self, num_queues: usize) -> Self {
        if num_queues < 2 {
            panic!(
                "number of queues must be at least 2 when using multi-queue mode
                Use ConfigBuilder<false> for single-queue mode"
            )
        }
        self.config.num_queues = Some(num_queues);
        self
    }
}
