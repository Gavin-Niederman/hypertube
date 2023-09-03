fn main() {
    let config = hypertube::config::ConfigBuilder::new()
        .with_name(std::ffi::CString::new("tun0").unwrap())
        .with_num_queues(1)
        .build();

    let device = hypertube::device::Device::new(config).unwrap();
    device.bring_up().unwrap();
    device.set_address(std::net::IpAddr::V4(std::net::Ipv4Addr::new(10, 0, 0, 1))).unwrap();

    let queue = device.queue(0).unwrap();

    let mut buf = [0; 4096];
    let amount = queue.read(&mut buf).unwrap();
    println!("{:?}", &buf[0..amount]);
}