//! Audio waveform buffer

pub struct WaveformBuffer {
    samples: Vec<f32>,
    write_pos: usize,
    total_samples: usize,
}

impl WaveformBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: vec![0.0; capacity],
            write_pos: 0,
            total_samples: 0,
        }
    }

    pub fn push(&mut self, data: &[f32]) {
        for &s in data {
            self.samples[self.write_pos] = s;
            self.write_pos = (self.write_pos + 1) % self.samples.len();
            self.total_samples += 1;
        }
    }

    pub fn sample_count(&self) -> usize {
        self.total_samples
    }

    pub fn display_samples(&self, count: usize) -> Vec<f32> {
        let count = count.min(self.samples.len());
        let start = (self.write_pos + self.samples.len() - count) % self.samples.len();
        (0..count)
            .map(|i| self.samples[(start + i) % self.samples.len()])
            .collect()
    }

    pub fn peak(&self, count: usize) -> f32 {
        let count = count.min(self.samples.len());
        if count == 0 {
            return 0.0;
        }
        let start = (self.write_pos + self.samples.len() - count) % self.samples.len();
        (0..count)
            .map(|i| self.samples[(start + i) % self.samples.len()].abs())
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    pub fn rms(&self, count: usize) -> f32 {
        let count = count.min(self.samples.len());
        if count == 0 {
            return 0.0;
        }
        let start = (self.write_pos + self.samples.len() - count) % self.samples.len();
        let sum: f32 = (0..count)
            .map(|i| {
                let s = self.samples[(start + i) % self.samples.len()];
                s * s
            })
            .sum();
        (sum / count as f32).sqrt()
    }
}
