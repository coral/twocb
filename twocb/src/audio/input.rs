use crate::audio;
use cpal;
use cpal::traits::{DeviceTrait, HostTrait};
use crossbeam_channel;
use log::{error, info};

pub struct Input {
    stream_settings: audio::StreamSetting,
    stream: Option<cpal::Stream>,
}

impl Input {
    pub fn new(s: audio::StreamSetting) -> Input {
        Input {
            stream_settings: s,
            stream: None,
        }
    }

    pub fn start(&mut self) -> crossbeam_channel::Receiver<Vec<f32>> {
        let host = cpal::default_host();

        let device = host
            .default_input_device()
            .expect("Failed to get default input device");

        let device_name = match device.name() {
            Ok(device_name) => device_name,
            Err(e) => {
                error!("Could not get device name: {}", e);
                "no-device".to_string()
            }
        };

        info!("Audio input device selected: {}", device_name);

        let config = &cpal::StreamConfig {
            channels: self.stream_settings.channels,
            buffer_size: cpal::BufferSize::Fixed(self.stream_settings.buffer_size),
            sample_rate: self.stream_settings.sample_rate,
        };

        let err_fn = move |err| {
            error!("Error on audio input stream: {}", err);
        };

        let (ds, dr) = crossbeam_channel::unbounded();

        let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let output_fell_behind = false;
            match ds.send(data.to_vec()) {
                Err(err) => {
                    error!("Input audio cannot be fed to channel: {}", err)
                }
                _ => {}
            };
            if output_fell_behind {
                eprintln!("output stream fell behind: try increasing latency");
            }
        };

        let stream = device
            .build_input_stream(config, input_data_fn, err_fn, None)
            .unwrap();
        self.stream = Some(stream);

        return dr;
    }
}
