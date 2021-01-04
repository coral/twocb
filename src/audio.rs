use cpal;
use cpal::traits::{DeviceTrait, HostTrait};
use log::info;

pub struct AudioInput {
    sample_rate: u32,
    buffer_size: u32,
}

impl AudioInput {
    pub fn new(sample_rate: u32, buffer_size: u32) -> AudioInput {
        AudioInput {
            sample_rate,
            buffer_size,
        }
    }
    //Start does x Y Z and you need to blah
    pub fn start(&mut self) {
        self.start_default();
    }

    fn start_default(&mut self) {
        let host = cpal::default_host();

        let device = host
            .default_input_device()
            .expect("Failed to get default input device");

        info!("Audio input device selected: {}", device.name().unwrap());

        let config = &cpal::StreamConfig {
            channels: 2,
            buffer_size: cpal::BufferSize::Fixed(self.buffer_size),
            sample_rate: cpal::SampleRate(self.sample_rate),
        };

        // let err_fn = move |err| {
        //     eprintln!("an error occurred on stream: {}", err);
        // };

        // let stream = device.build_input_stream(
        //     config,
        //     move |data, inp: &cpal::InputCallbackInfo| prc.process(data, inp),
        //     err_fn,
        // );
    }
}
