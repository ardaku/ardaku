# aldarons-kernel
A kernel, written in Rust (Early stages).

## Install In another OS
Have an installer for Linux, Windows, Mac OS, that downloads a program onto your computer.

## Install On X86
Have an installer image (Linux) should look for what drivers are needed, and then build Aldaron's Kernel.

## Install On Raspberry Pi / Phone / Other ARM
Have a script that builds an image that's the same for all of said platform.

## What should it do.
Use "SHMFS": Secure Hash Map File System (in folder when in anther os).  What does root of filesystem look like:
* `kernel` / `kernel.exe` the actual OS executable
* `.shmfs` the hashmap representation of the filesystem using the serde crate
* `[APP].shmfs` a hashmap representation of the files owned by a specific app
* `aldarons-os.shmfs` the os application (owns all program files)

Provide the ADI API, so the ADI implementation just simply calls this API by sending messages to the kernel thread.
