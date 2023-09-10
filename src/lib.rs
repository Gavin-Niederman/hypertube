pub mod config;
pub mod device;
pub mod queue;

pub use device::Device;
pub use queue::Queue;

pub fn config() -> config::ConfigBuilder {
    config::ConfigBuilder::new()
}

#[cfg(test)]
mod tests {
    #[test]
    fn create_tun_device() {
        let config = crate::config::ConfigBuilder::new()
            .with_num_queues(1)
            .build();

        crate::device::Device::new(config).unwrap();
    }
}
