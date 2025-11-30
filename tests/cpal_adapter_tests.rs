//! Tests for cpal audio adapter

#[cfg(feature = "cpal")]
mod cpal_tests {
    use screencapturekit::cpal_adapter::{AudioFormat, AudioRingBuffer};

    #[test]
    fn test_audio_format_to_stream_config() {
        let format = AudioFormat {
            sample_rate: 48000,
            channels: 2,
            bits_per_sample: 32,
            is_float: true,
        };

        let config = format.to_stream_config();
        assert_eq!(config.sample_rate.0, 48000);
        assert_eq!(config.channels, 2);
    }

    #[test]
    fn test_audio_format_sample_format_float() {
        let format = AudioFormat {
            sample_rate: 44100,
            channels: 2,
            bits_per_sample: 32,
            is_float: true,
        };

        assert_eq!(format.sample_format(), cpal::SampleFormat::F32);
    }

    #[test]
    fn test_audio_format_sample_format_i16() {
        let format = AudioFormat {
            sample_rate: 44100,
            channels: 2,
            bits_per_sample: 16,
            is_float: false,
        };

        assert_eq!(format.sample_format(), cpal::SampleFormat::I16);
    }

    #[test]
    fn test_audio_format_sample_format_i8() {
        let format = AudioFormat {
            sample_rate: 22050,
            channels: 1,
            bits_per_sample: 8,
            is_float: false,
        };

        assert_eq!(format.sample_format(), cpal::SampleFormat::I8);
    }

    #[test]
    fn test_audio_format_sample_format_i32() {
        let format = AudioFormat {
            sample_rate: 96000,
            channels: 2,
            bits_per_sample: 32,
            is_float: false,
        };

        assert_eq!(format.sample_format(), cpal::SampleFormat::I32);
    }

    #[test]
    fn test_ring_buffer_write_read() {
        let mut buffer = AudioRingBuffer::new(100);

        // Write some samples
        let input = [1.0f32, 2.0, 3.0, 4.0, 5.0];
        let written = buffer.write(&input);
        assert_eq!(written, 5);
        assert_eq!(buffer.available(), 5);

        // Read them back
        let mut output = [0.0f32; 5];
        let read = buffer.read(&mut output);
        assert_eq!(read, 5);
        assert_eq!(output, input);
        assert_eq!(buffer.available(), 0);
    }

    #[test]
    fn test_ring_buffer_wrap_around() {
        let mut buffer = AudioRingBuffer::new(10);

        // Fill with samples
        let input1 = [1.0f32; 8];
        buffer.write(&input1);

        // Read some
        let mut output = [0.0f32; 6];
        buffer.read(&mut output);

        // Write more (should wrap)
        let input2 = [2.0f32; 6];
        let written = buffer.write(&input2);
        assert_eq!(written, 6);

        // Read all remaining
        let mut output2 = [0.0f32; 8];
        let read = buffer.read(&mut output2);
        assert_eq!(read, 8);
    }

    #[test]
    fn test_ring_buffer_overflow_protection() {
        let mut buffer = AudioRingBuffer::new(5);

        // Try to write more than capacity
        let input = [1.0f32; 10];
        let written = buffer.write(&input);
        assert_eq!(written, 5); // Only writes up to capacity
    }

    #[test]
    fn test_ring_buffer_underflow_fills_silence() {
        let mut buffer = AudioRingBuffer::new(10);

        // Write 3 samples
        buffer.write(&[1.0, 2.0, 3.0]);

        // Read 5 samples (2 should be silence)
        let mut output = [9.9f32; 5];
        let read = buffer.read(&mut output);
        assert_eq!(read, 3);
        assert_eq!(output[0], 1.0);
        assert_eq!(output[1], 2.0);
        assert_eq!(output[2], 3.0);
        assert_eq!(output[3], 0.0); // Silence
        assert_eq!(output[4], 0.0); // Silence
    }
}
