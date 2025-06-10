/// Waveform sound channel
/// 
/// This struct only describes the waveform sampler behaviour of the sound channels.
/// Its output may be overwritten by the voice or noise features.
#[derive(Clone, Copy)]
pub struct Channel {
    /// The 16 byte waveform, contains 32 4-bit samples
    pub waveform: [u8; 16],
    /// The amount of ticks the sound unit makes before this channel is ticked switches samples
    pub frequency: u16,

    /// Time since last channel tick
    sample_clock: u16,
    /// Index of the sample to be played from among the 32 contained in the waveform
    sample_idx: usize,

    /// The sample currently being put out
    sample: u8,
}

impl Channel {
    /// Generates a new channel
    pub fn new() -> Self {
        Self {
            waveform: [0; 16],
            frequency: 0,

            sample_clock: 0,
            sample_idx: 0,

            sample: 0,
        }
    }

    /// Informs the channel that a sound unit tick has happened and updates the sample clock
    /// 
    /// When the sample clock reaches 0 the sample index and output sample are updated
    /// 
    /// # Return value
    /// 
    /// The sample currently being played
    pub fn tick(&mut self) -> u8 {
        self.sample_clock = self.sample_clock.saturating_sub(1);
        if self.sample_clock == 0 {
            self.sample_clock = self.frequency;
            self.sample_idx = (self.sample_idx + 1) & 0x1F;
            let byte = self.waveform[self.sample_idx / 2];
            self.sample = (byte >> ((self.sample_idx & 1) * 4)) & 0x0F;
        }

        return self.sample;
    }
}