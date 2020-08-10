# Dive
A simple unikernel written in Rust.  Though it targets RISC-V, it may also run
on other systems (and within other operating systems!) through machine-code
translation emulation.  No need for a virtual machine!

## Compile And Run App In Kernel
 - Install dive `cargo install dive`.
 - Install the `yeet` compiler `cargo install yeet`, or the `yote` text editor
   `cargo install yote` (yote includes the yeet compiler).
 - Create file called `test.code`:
```aratar
#!aratar 0.0.20
export start
import kernel

def start() {
    kernel.write("Hello, world!")
}
```
 - Start up the emulation, yote "run" button or:
```bash
yeet test.code
```

# Syscalls For The Dive Kernel
The emulator will translate syscalls into other code to run on whatever cpu.

Registers Used:
 - a0: (in: ecallcode Code, out: ev EventType | buf Addr)
 - a1: (in: size|curs Offs, out: len Int32U | res (Int16U, Int16U))
 - a2: (in: data|text Addr, out: dur Nanos32U)

## Core syscalls
```aratar
a0: (
    syscall_id: Hex8,
    uses_a0: Bin1
    uses_a1: Bin1
    uses_a2: Bin1
    uses_a3: Bin1
    _reserved: Bin4
    value: Bin16 # limited to 12 bits for address offsets
)
```

### Blocking System Input
 - x00: Wait for events.
```aratar
def wait() -> (a0: EventType)
```

### Non-Blocking (Queuing) System Output
 - x01: Queue a line of text output
```aratar
def say(a0~size: Int16U, a1~text: [Utf8]) -> ()
```
 - x02: Queue saving to currently open file
```aratar
def save(a0~size: Int16U, a1~seek: Opt[Int32U-1], a2~data: [Data8]) -> ()
```
 - x03: Queue truncating currently open file
```aratar
def trunc(a1~size: Opt[Int32U-1]) -> ()
```
 - x04: Queue change which file is open
```aratar
def open(a0~namesize: Int16U, a1~nametext: [Utf8]) -> ()
```

### Async System Request Input Event
 - x05: Request Prompt For Text Input
```aratar
def ask(a0~size: Int16U, a1~text: [Utf8]) -> ()
```
 - x06: Request to load from currently open file into a buffer
```aratar
def load(a0~size: Int16U, a1~seek: Opt[Int32U-1], a2~data: @[Data8]) -> ()
```
 - x07: Request random number
```aratar
def rand(a0~type: Int16U) -> ()
```
 - x08: Request separate (or emulated separate) processor for task
```aratar
def proc(a0~fn_ofs: Addr16S) -> ()
```
 - x09: Request analog to digital conversion
```aratar
def adc(a0~pin: Int16U) -> ()
```
 - x0A: Request timer
```aratar
def timer(a0~seconds: Int16U, a1~nanos: Int32U) -> (a0~id: Int16U)
```
 - x0B: Request Network Connect
   - 0x0000: TCP-IPv4
   - 0x0001: UDP-IPv4
   - 0x0080: TCP-IPv6
   - 0x0081: UDP-IPv6
   - 0xFF00: Bluetooth:4
   - 0xFF01: BLE
```aratar
def connect(a0~addrtype: Int16U, a1~port: Int32U, a2~addr: [Int8U]) -> ()
```
 - x0C: Request Network Send
```aratar
def send(a0~socket: Int16U, a1~size: Int32U, a2~data: [Data8]) -> ()
```
 - x0D: Request Network Receive
```aratar
def recv(a0~socket: Int16U, a1~size: Int32U, a2~data: @[Data8]) -> ()
```
 - x0E: Request socket connect (local system network emulation)
   - 0x0000: Localhost
```aratar
def socket(a0~addrtype: Int16U, a1~port: Int32U, a2~addr: [Int8U]) -> ()
```
 - x0F: Close connection
```aratar
def disconnect(a0~socket: Int16U) -> ()
```
 - x10: Request GPU computation
```aratar
def gpu_compute(a0~op: Int16U, a1~size: Int32U, a2~data: @[Data_]) -> ()
```
 - x11: Request GPU rendering onto a gpu raster
```aratar
def gpu_raster(a0~op: Int16U, a1~size: Int32U, a2~data: @[Float64]) -> ()
```

### Async Hardware Mapping
 - x12: RESERVED for another GPU syscall
 - x13: Create a buffer on the GPU
```aratar
def gpu_buf()
```
 - x14: Try to connect to an additional screen (automatic).
```aratar
def screen() -> (
    a0~pixels: Opt[Addr], a1~res: (w: Int16U, h: Int16U), a2~dur: Nanos32U
)
```
 - x15: Try to connect to an additional speaker (automatic).
```aratar
def speaker() -> (
    a0~samples: Opt[Addr], a1~res: Int32U, a2~dur: Nanos32U
)
```
 - x16: Try to connect to an additional camera (user selection prompt).
```aratar
def camera() -> (
    a0~pixels: Opt[Addr], a1~res: (w: Int16U, h: Int16U), a2~dur: Nanos32U
)
```
 - x17: Try to connect to an additional microphone (user selection prompt).
```aratar
def microphone() -> (a0: Opt[Addr], a1~len: Int32U, a2~dur: Nanos32U)
```
 - x18: Digital GPIO Voltage
```aratar
def digital_gpio() -> (a0~buf: Addr[Bit], a1~len: Int32U)
```
 - x19: Analog GPIO Voltage
```aratar
def analog_gpio() -> (a0~buf: Addr[Fix1_31], a1~len: Int32U)
```
 - x18: Digital GPIO Direction
```aratar
def digital_is_out() -> (a0~buf: Addr[Bit], a1~len: Int32U)
```
 - x19: Analog GPIO Direction
```aratar
def analog_is_out() -> (a0~buf: Addr[Bit], a1~len: Int32U)
```

### Non-blocking System Input
 - x1F: Query hardware support
   - x0000: Number of CPUs (must be non-zero)
   - x0001: Number of GPUs (must be either 1 or 0)
   - x0010: Number of Digital GPIO pins (no number restrictions)
   - x0011: Number of Analog GPIO pins (no number restrictions)
```aratar
def query() -> IntU32
```

## EventType: Bin32
 - UserInput: x0
   - Keyboard: x0 `(lang: [Ascii; 2])`
     - Quit(Escape)/Tilde/NotSign/Power: 0x00
     - One/Bang/InvertedBang/LatinMode: 0x01
     - Two/At/Squared/ChineseCangjieMode: 0x02
     - Three/Pound/Pound/JapaneseOyayubiShifutoMode: 0x03
     - Four/Dollar/CurrencySign/KoreanDubeolsikMode: 0x04
     - Five/Percent/Euro/GeorgianPhoneticMode: 0x05
     - Six/Caret/Trademark/GreekCodepointMode: 0x06
     - Seven/Ampersand/Cent/HebrewArabicCyrillicMode: 0x07
     - Eight/Asterisk/Ruble/ArmenianPhoneticMode: 0x08
     - Nine/LParens/Yen/TamazightMode: 0x09
     - Zero/RParens/LaoMode: 0x0A
     - Minus/Underscore: 0x0B
     - Plus/Equal: 0x0C
     - Backspace/Delete: 0x0D
     - Tab/UnTab: 0x10
     - q/Q: 0x11
     - w/W: 0x12
     - e/E: 0x13
     - r/R: 0x14
     - t/T: 0x15
     - y/Y: 0x16
     - u/U: 0x17
     - i/I: 0x18
     - o/O: 0x19
     - p/P: 0x1A
     - LSquare/LBrace: 0x1B
     - LSquare/LBrace: 0x1C
     - Backslash/Bar: 0x1D
     - Compose/Search: 0x20
     - a/A: 0x21
     - s/S: 0x22
     - d/D: 0x23
     - f/F: 0x24
     - g/G: 0x25
     - h/H: 0x26
     - j/J: 0x27
     - k/K: 0x28
     - l/L: 0x29
     - Semicolon/Colon: 0x2A
     - Apostrophe/Quote: 0x2B
     - Enter/UnEnter: 0x2C
     - LShift: 0x30
     - z/Z: 0x32
     - x/X: 0x33
     - c/C: 0x34
     - v/V: 0x35
     - b/B: 0x36
     - n/N: 0x37
     - m/M: 0x38
     - Comma/LessThan: 0x39
     - Period/MoreThan: 0x3A
     - Slash/Question: 0x3B
     - ArrowUp: 0x3C
     - Shift: 0x3D
     - Control: 0x40
     - Clipboard: 0x41
     - System: 0x42
     - Alt: 0x43
     - Space: 0x45
     - AltGraph: 0x48
     - VolumeDown: 0x49
     - VolumeUp: 0x4A
     - ArrowLeft: 0x4B
     - ArrowDown: 0x4C
     - ArrowRight: 0x4D
     - TextInput: 0x70-0x7F `(utf8: [Byte; 2])`
     - \[ReleaseKey\]: 0x8_-0xB_
     - TextInputLong: 0xFF `(pre_utf32: [Byte; 2])`
   - Mouse: x1
     - ButtonLeft: 0x00 `(Bool)`
     - ButtonMiddle: 0x01 `(Bool)`
     - ButtonRight: 0x02 `(Bool)`
     - ButtonSide: 0x03 `(Bool)`
     - ButtonDpi: 0x04 `(Bool)`
     - MoveX: 0x80 `(Bool)`
     - MoveY: 0x81 `(Bool)`
     - ScrollX: 0x90 `(Bool)`
     - ScrollY: 0x91 `(Bool)`
   - Touchscreen: x2
     - TouchX: 0x00 `(Int4U, Int16U)`
     - TouchY: 0x01 `(Int4U, Int16U)`
     - Release: 0x0F `()`
   - Touchpad: x3
     - ButtonLeft: 0x00 `(Bool)`
     - ButtonMiddle: 0x01 `(Bool)`
     - ButtonRight: 0x02 `(Bool)`
     - MoveX: 0x80 `(Int16U)`
     - MoveY: 0x81 `(Int16U)`
     - ScrollX: 0x90 `(Int16U)`
     - ScrollY: 0x91 `(Int16U)`
   - Gamepad: x4
     - ButtonLTrigger: 0x00 `(Bool)`
     - ButtonRTrigger 0x01 `(Bool)`
     - ButtonLBumper: 0x02 `(Bool)`
     - ButtonRBumper: 0x03 `(Bool)`
     - ButtonLStick: 0x04 `(Bool)`
     - ButtonRStick: 0x05 `(Bool)`
     - ButtonLMenu: 0x06 `(Bool)`
     - ButtonRMenu: 0x07 `(Bool)`
     - ButtonDpadUp: 0x08 `(Bool)`
     - ButtonDpadLeft: 0x09 `(Bool)`
     - ButtonDpadRight: 0x0A `(Bool)`
     - ButtonDpadDown: 0x0B `(Bool)`
     - ButtonV: 0x0C `(Bool)`
     - ButtonA: 0x0D `(Bool)`
     - ButtonB: 0x0E `(Bool)`
     - ButtonH: 0x0F `(Bool)`
     - ButtonPaddleL: 0x10 `(Bool)`
     - ButtonPaddleR: 0x11 `(Bool)`
     - AxisMainStickX: 0x80 `(Int16U)`
     - AxisMainStickY: 0x81 `(Int16U)`
     - AxisAltStickX: 0x82 `(Int16U)`
     - AxisAltStickY: 0x83 `(Int16U)`
     - AxisLTrigger: 0x84 `(Int16U)`
     - AxisRTrigger: 0x85 `(Int16U)`
 - Custom Circuit Input: x1
   - Gpio: x0
     - Digital: x00
     - Analog: x01
     - Pwm: x02
   - Bus: x1
     - I2c: x00
     - Spi: x01
     - Can: x02
     - Pci: x03
     - Sada: xFF
 - Async Ready: x2
   - RandomReady: x0
   - AdcReady: x1
   - TaskReady: x2
   - TimerReady: x3
 - Mapped (Media) Input: x3
   - Camera: x0
     - Capture: x00
   - Microphone: x1
     - Capture: x00
 - Defined Ciruit Input: x4
   - ~TimerTick: x0~
   - GPS: x1
   - Accelerometer: x2
   - Gyro: x3
   - StorageDrive: x4
   - BatterySensor: xF
 - Network Input: x7
   - WifiEthernetData: x0
   - Bluetooth: x1
   - Cell: x2
     - Text: x00
     - Call: x01
     - TextMedia: x02
   - Ble: x3
 - Media Output: xB
   - ScreenRefresh: x0
   - SpeakerRefresh: x1
