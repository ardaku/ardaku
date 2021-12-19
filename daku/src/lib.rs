//! Low-level interface for the Ardaku system.  This is similar to how the libc
//! and windows crates are used for unix-derivatives and Windows (or web-sys for
//! targetting the web).
//!
//! Examples will use the pasts async runtime, because the system is
//! asynchronous in many of it's APIs.  Other runtimes may or may not work on
//! Ardaku.

pub use rdaku::Device;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
