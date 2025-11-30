//! Tests for cpal audio adapter

#[cfg(feature = "cpal")]
mod cpal_tests {
    use screencapturekit::cpal_adapter::AudioFormat;

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
}
