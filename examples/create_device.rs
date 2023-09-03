fn main() {
    let config = hypertube::config::ConfigBuilder::new()
        .with_name(std::ffi::CString::new("tun0").unwrap())
        .with_num_queues(1)
        .with_address(std::net::IpAddr::V4(std::net::Ipv4Addr::new(10, 0, 0, 1)))
        .with_blocking(true)
        .build();

    let device = hypertube::device::Device::new(config).unwrap();
    device.bring_up().unwrap();

    let queue = device.queue(0).unwrap();

    loop {
        let mut buf = [0; 4096];
        let amount = queue.read(&mut buf).unwrap();
        println!("{:?}", &buf[0..amount]);
    }
}