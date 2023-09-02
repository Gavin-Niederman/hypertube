fn main() {
    let config = hypertube::config::ConfigBuilder::new()
        .with_num_queues(1)
        .build();

    let device = hypertube::device::Device::new(config).unwrap();

    let queue = device.queue(0).unwrap();

    queue.write(&[0x0, 0x0, 0x0]).unwrap()
}