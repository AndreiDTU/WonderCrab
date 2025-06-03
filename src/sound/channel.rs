#[derive(Clone, Copy)]
pub struct Channel {
    pub waveform: [u8; 16],
    pub frequency: u16,

    sample_clock: u16,
    sample_idx: usize,

    sample: u8,
}

impl Channel {
    pub fn new() -> Self {
        Self {
            waveform: [0; 16],
            frequency: 0,

            sample_clock: 0,
            sample_idx: 0,

            sample: 0,
        }
    }

    pub fn tick(&mut self) -> u8 {
        self.sample_clock = self.sample_clock.saturating_sub(1);
        if self.sample_clock == 0 {
            self.sample_clock = self.frequency;
            self.sample_idx = (self.sample_idx + 1) & 0x1F;
            let byte = self.waveform[self.sample_idx / 2];
            self.sample = byte >> ((self.sample_idx % 2) * 4);
        }

        return self.sample;
    }
}