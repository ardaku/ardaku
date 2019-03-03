// Do Nothing
const NULL: u8 = b'\x00';
// Print Heading (Program -> OS)
const SOH: u8 = b'\x01';
// Switch To Text Mode (Program -> OS)
const STX: u8 = b'\x02';
// Switch To Graphical Mode (Program -> OS)
const ETX: u8 = b'\x03';
// Exit Program (Program -> OS)
const EOT: u8 = b'\x04';
// Query Audio Buffer Open Samples (Program <-> OS), switches direction
const ENQ: u8 = b'\x05';
// Network Send Packet (Program -> OS)
const ACK: u8 = b'\x06';
// OS Sound / Screen Flash
const BEL: u8 = b'\x07';
// Backspace for Terminal Output '\b' (Program -> OS)
const BS: u8 = b'\x08';
// Tab for Terminal Output '\t' (Program -> OS)
const TAB: u8 = b'\x09';
// New Line for Terminal Output '\n' (Program -> OS)
const LF: u8 = b'\x0A';
// Vertical Tab for Terminal Output '\n\n\n\n' (Program -> OS)
const VT: u8 = b'\x0B';
// Clear Terminal Ouput (Program -> OS)
const FF: u8 = b'\x0C';
// Carriage Return for Terminal Output '\r' (Program -> OS)
const CR: u8 = b'\x0D';
// UNIMPLEMENTED: Shift Out
const SO: u8 = b'\x0E';
// UNIMPLEMENTED: Shift In
const SI: u8 = b'\x0F';
// Switch Input Device [Touchscreen, Keyboard, Game] (Program -> OS)
const DLE: u8 = b'\x10';
// Poll Device Input (Program <-> OS), switches direction
const DC1: u8 = b'\x11';
// Poll Device Microphone (Program <-> OS), switches direction
const DC2: u8 = b'\x12';
// Poll Device USB (Program <-> OS), switches direction
const DC3: u8 = b'\x13';
// Poll Device Camera, Webcam (Program <-> OS), switches direction
const DC4: u8 = b'\x14';
// Network Receive Packet (Program -> OS)
const NAK: u8 = b'\x15';
// Save State To Drive (Program -> OS)
const SYN: u8 = b'\x16';
// UNIMPLEMENTED: End of trans. block
const ETB: u8 = b'\x17';
// UNIMPLEMENTED: Cancel
const CAN: u8 = b'\x18';
// UNIMPLEMENTED: End of medium
const EM: u8 = b'\x19';
// Write to Graphic (Program -> OS)
const SUB: u8 = b'\x1A';
// Vector Graphic Operation (Program -> OS)
const ESC: u8 = b'\x1B';
// Load State From Drive (Program -> OS)
const FS: u8 = b'\x1C';
// Load From External Drive (Program <-> OS), switches
const GS: u8 = b'\x1D';
// Speaker Output @Uint16 48,000 hz (Program -> OS)
const RS: u8 = b'\x1E';
// UNIMPLEMENTED: Unit Separator
const US: u8 = b'\x1F';

/// Dive API Message
pub enum Msg {
    
}
