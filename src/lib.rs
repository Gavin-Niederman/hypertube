pub mod builder;
pub mod device;
pub mod queue;

pub use device::Device;
pub use queue::Queue;

/// Creates a new DeviceBuilder
pub fn builder() -> builder::DeviceBuilder {
    builder::DeviceBuilder::new()
}

#[cfg(test)]
mod tests {
    #[test]
    fn create_tun_device() {
        let device = crate::builder().with_num_queues(1).build();
        println!("{:?}", device);
    }
}
