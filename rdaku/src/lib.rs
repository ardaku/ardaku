/// A device handle.
#[derive(Debug)]
pub struct Device(u32);

/// Services provided by the Ardaku engine.
#[derive(Debug)]
pub enum Service {
    /// Serial / debug (file) logging
    Logging = 0,
    /// Serial / debug command prompt
    Prompt = 1,

    /// Pixel display buffer
    Screen = 2,
    /// Camera frame capture.
    Camera = 3,

    /// Play f32 7.1 audio (32 samples / channel)
    Speakers = 4,
    /// Record f32 7.1 audio (32 samples / channel)
    Microphone = 5,

    /// Atomically save a section of a page to the currently open file.
    Save = 6,
    /// Load requested sections of currently open file.
    Load = 7,

    /// Share currently open file
    Share = 8,
    /// Open a different file (chosen by the user)
    Open = 9,

    /// Rumble or other haptic events for HID
    Haptic = 10,
    /// Get input from an HID
    Input = 11,

    /// Speech synthesis
    Speech = 12,
    /// Voice recognition
    Voice = 13,

    /// WebGPU Graphics API
    Graphics = 14,
    /// Neural-network, visual processing, compute shader, etc. API
    Compute = 15,

    /// Load a WASM module (DE => Apps, Apps => Plugins).
    Import = 16,
    /// Respond to requests from parent WASM module.
    Export = 17,

    /// Connect to a server (HTTP).
    Net = 18,
    /// Start a server (HTTP).
    Serve = 19,

    /// Set GUI headbar / other GUI OS integration.
    Header = 20,
    /// Get GUI Events.
    Gui = 21,

    /// Set power / CPU settings / Hostname.
    Hardware = 22,
    /// Get battery level / CPU & RAM usage.
    Stats = 23,

    /// Set accessibility/localization settings.
    Accessibility = 24,
    /// Get accessibility/localization settings.
    Settings = 25,

    /// Send / receive bluetooth message
    Bluetooth = 26,
    /// Do bluetooth device pairing
    Pair = 27,

    /// WebRTC transmit stream
    WebRTC = 28,
    /// WebRTC stream receiving
    Stream = 29,

    /// Set a timer.
    Timer = 30,
    /// Get Date/Time
    Clock = 31,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
