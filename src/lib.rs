//! To create a TUN device, use a [`DeviceBuilder`](builder::DeviceBuilder).
//! There are several ways to create a [`DeviceBuilder`](builder::DeviceBuilder):
//! * [`hypertube::builder()`](crate::builder())
//! * [`DeviceBuilder::new()`](builder::DeviceBuilder::new) or [`DeviceBuilder::default()`](builder::DeviceBuilder::default)
//! * [`Device::builder()`](Device::builder)
//!
//! Eg.
//!
//! ```rust
//! let device = builder()
//!     .build()
//!     .unwrap();
//! ```
//!
//! Now to write to a [`Device`].
//! In order to write to or read from a [`Device`] you need to create a [`Queue`].
//! There are two types of queues, blocking and nonblocking.
//! ```rust
//! let device = builder()
//!     .with_num_queues(2)
//!     .build()
//!     .unwrap();
//!
//! let queue1 = device.queue(0);
//! let queue2 = device.queue_nonblocking(1);
//!
//! queue2.write(&[some bytes here]);
//!
//! let mut buf = [0; some size];
//!
//! queue1.read(&mut buf);
//!
//! ```
//! Blocking and nonblocking queues are accessed from the same pool of queues.
//! The pool of queues is created when the device is built.
//! In the future, the ability to add or remove queues after the [`Device`] is created should be added.

pub mod builder;
pub mod device;
pub mod queue;

pub use device::Device;
pub use queue::Queue;

/// Creates a new [`DeviceBuilder`](builder::DeviceBuilder).
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
