//! Audio waveform buffer

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

pub struct WaveformBuffer {
    samples: Vec<f32>,
    write_pos: usize,
    has_received_data: AtomicBool,
    sample_count: AtomicU64,
}

impl WaveformBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: vec![0.0; capacity],
            write_pos: 0,
            has_received_data: AtomicBool::new(false),
            sample_count: AtomicU64::new(0),
        }
    }

    pub fn push(&mut self, data: &[f32]) {
        if !data.is_empty() {
            self.has_received_data.store(true, Ordering::Relaxed);
            self.sample_count.fetch_add(data.len() as u64, Ordering::Relaxed);
        }
        for &s in data {
            self.samples[self.write_pos] = s;
            self.write_pos = (self.write_pos + 1) % self.samples.len();
        }
    }

    #[allow(dead_code)]
    pub fn has_data(&self) -> bool {
        self.has_received_data.load(Ordering::Relaxed)
    }

    pub fn sample_count(&self) -> u64 {
        self.sample_count.load(Ordering::Relaxed)
    }

    pub fn display_samples(&self, count: usize) -> Vec<f32> {
        let count = count.min(self.samples.len());
        let start = (self.write_pos + self.samples.len() - count) % self.samples.len();
        (0..count)
            .map(|i| self.samples[(start + i) % self.samples.len()])
            .collect()
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

    pub fn peak(&self, count: usize) -> f32 {
        let count = count.min(self.samples.len());
        if count == 0 {
            return 0.0;
        }
        let start = (self.write_pos + self.samples.len() - count) % self.samples.len();
        (0..count)
            .map(|i| self.samples[(start + i) % self.samples.len()].abs())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }
}
