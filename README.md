# Ardaku
Ardaku is a general-purpose unikernel operating system running Wasmer to execute
WebAssembly applications.

 - [Syscalls](SYSCALLS.md)

## Ideas
 - The operating system should be able to run as an application within another
   operating system (avoiding the need for VMs)
 - Operating systems should be designed with a security-first mindset, making
   all programs sandboxed by default (making WebAssembly a good target)
 - Programs compiled for the operating system should be able to be
   re-distributed without having to match CPU architecture, while also running
   at native speeds (which Wasmer can handle pretty well)
 - Syscalls should be simple, high level mathematical functions that make it
   difficult to make programming errors, while also being powerful and fast
 - Adding support for new platforms should be as easy as implementing a trait of
   all the syscalls.
