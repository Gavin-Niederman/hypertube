pub mod device;
pub mod config;
pub mod queue;

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