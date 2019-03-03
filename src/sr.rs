// use crossbeam::atomic::AtomicCell;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

/// A 32 kb stream (up to 1/4 second of audio at Uint16 48000, or Uint8 Grayscale 128x128).
///
/// A Stream is meant to be shared between two threads.  The master thread spawns the slave thead,
/// and gives it this struct.  The master thread cannot read any of buffer until the slave thread
/// increments the `slave` index.  The indices go back around in a circle.
#[repr(C)]
struct Sr {
    // 4-8 bytes (multiple threads may access this data)
    master: AtomicUsize,
    // 4-8 bytes (multiple threads may access this data)
    slave: AtomicUsize,
    // 32,000 bytes (protected by master and slave indices)
    buffer: [u8; 32_000],
}

/// Master thread can pull from input.
pub struct Recv {
    ptr: *mut Sr,
}

impl Recv {
    #[inline(always)]
    pub fn recv(&mut self) -> Option<u8> {
        unsafe {
            (*self.ptr).recv()
        }
    }

    #[inline(always)]
    pub(crate) fn send(&mut self, d: u8) {
        unsafe {
            (*self.ptr).send(d)
        }
    }
}

/// Slave thread can push to output.
pub struct Send {
    ptr: *mut Sr,
}

impl Send {
    #[inline(always)]
    pub fn send(&mut self, d: u8) {
        unsafe {
            (*self.ptr).send(d)
        }
    }

    #[inline(always)]
    pub(crate) fn recv(&mut self) -> Option<u8> {
        unsafe {
            (*self.ptr).recv()
        }
    }
}

/// Create new Send/Recv sockets.
#[inline(always)]
pub fn sr() -> (Send, Recv) {
    let ptr = Box::into_raw(Box::new(Sr {
        master: AtomicUsize::new(0),
        slave: AtomicUsize::new(0),
        buffer: [0; 32_000],
    }));

    (Send { ptr }, Recv { ptr })
}

impl Sr {
    /// Pull a byte from the stream, if it's available.  Only call on Master.
    #[inline(always)]
    fn recv(&mut self) -> Option<u8> {
        let master = self.master.load(Ordering::Relaxed);
        let next_master = (master + 1) % 32_000;
        let slave = self.slave.load(Ordering::Relaxed);

        // Master has caught up to the slave.
        if master == slave {
            return None;
        }

        // Read 1 byte.
        let rtn = self.buffer[master as usize];
        self.master.store(next_master, Ordering::Relaxed);
        Some(rtn)
    }

    /// Push a byte onto the stream, if there's enough space.  Only call on Slave.
    #[inline(always)]
    fn send(&mut self, byte: u8) {
        let slave = self.slave.load(Ordering::Relaxed);
        let next_slave = (slave + 1) % 32_000;
        let mut master;

        // Wait for master to catch up if slave is sending too much data.
        'lock: loop {
            master = self.master.load(Ordering::Relaxed);

            if master != next_slave {
                break 'lock;
            }
        }

        // Can write 1 byte now.
        self.buffer[slave as usize] = byte;
        self.slave.store(next_slave, Ordering::Relaxed);
    }
}
