use std::{net::Ipv4Addr, thread::scope};

use cidr::Ipv4Cidr;

fn main() {
    let config = hypertube::config::ConfigBuilder::new()
        .with_num_queues(2)
        .with_address(std::net::IpAddr::V4(std::net::Ipv4Addr::new(10, 0, 0, 1)))
        .with_netmask(cidr::IpCidr::V4(
            Ipv4Cidr::new(Ipv4Addr::new(10, 0, 0, 0), 24).unwrap(),
        ))
        .build();

    let device = hypertube::device::Device::new(config).unwrap();
    println!("{:?}", device);
    device.bring_up().unwrap();

    let queue = device.queue(0).unwrap();

    let thread_queue = device.queue_nonblocking(1).unwrap();
    scope(|s| {
        s.spawn(move || {
            for _ in 0..5 {
                let mut buf = [
                    0x45, 0x00, 0x00, 0x90, 0x7D, 0x99, 0x40, 0x00, 0x40, 0x11, 0xA6, 0xC1, 0x0A, 0x00, 0x01, 0x02, 0x0A, 0x00, 0x01, 0x01,
                ];
                let poll = thread_queue.write(&mut buf).unwrap();
                println!("{:?}", poll);
            }
        });

        for _ in 0..5 {
            let mut buf = [0; 4096];
            let amount = queue.read(&mut buf).unwrap();
            println!("{:?}", &buf[0..amount]);
        }
    });
}
