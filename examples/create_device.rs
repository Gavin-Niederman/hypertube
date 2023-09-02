fn main() {
    let config = hypertube::config::ConfigBuilder::new()
        .with_num_queues(1)
        .build();

    let device = hypertube::device::Device::new(config).unwrap();

    println!("{:?}", device);
}