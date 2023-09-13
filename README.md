# Hypertube

Hypertube is a library for the creation and usage of TUN devices.
Hypertube is meant to be a replacement for [tun](https://crates.io/crates/tun) that aims to improve the cli and allow for writing to and reading from multiple queues at the same time.

### Hypertube is not a perfect replacement for tun
The Hypertube API is similar to tun but not completely the same.
Also Hypertube is not currently cross-platform like tun, but I plan for it to be in the future.

## Usage
To create a TUN device, use a `DeviceBuilder`.
There are several ways to create a `DeviceBuilder`:
* `hypertube::builder()`
* `DeviceBuilder::new()` or `DeviceBuilder::default()`
* `Device::builder()`

Eg.

```rust
let device = builder()
    .build()
    .unwrap();
```

Now to write to a `Device`.
In order to write to or read from a `Device` you need to create a queue.
There are two types of queues, blocking and nonblocking.
```rust
let device = builder()
    .with_num_queues(2) 
    .build()
    .unwrap();

let queue1 = device.queue(0);
let queue2 = device.queue_nonblocking(1);

queue2.write(&[some bytes here]);

let mut buf = [0; some size];

queue1.read(&mut buf);

```
Blocking and nonblocking queues are accessed from the same pool of queues.
The pool of queues is created when the device is built.
You can add to the pool of queues by calling `Device.create_queue()` which will return the index of the new queue for easy accesess.


## Todo
* [X] Documentation
* [X] Non blocking queues
* [ ] IPV6 support
* [ ] Cross-platform support
  * [X] Linux
    * [X] Linux
    * [X] Android
  * [ ] BSD
    * [ ] Darwin
      * [ ] Macos
      * [ ] IOS
    * [ ] Maybe FreeBSD
  * [ ] Maybe Windows