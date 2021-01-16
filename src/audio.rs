use cpal;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use log::{error, info};
use ringbuf::RingBuffer;

pub struct AudioInput {
    sample_rate: u32,
    buffer_size: u32,
    channels: u16,
    buffer: RingBuffer<f32>,
}

impl AudioInput {
    pub fn new(sample_rate: u32, buffer_size: u32, channels: u16) -> AudioInput {
        AudioInput {
            sample_rate,
            buffer_size,
            channels,
            buffer: RingBuffer::<f32>::new((buffer_size * 8) as usize),
        }
    }
    //Start does x Y Z and you need to blah
    // pub fn start(&mut self) {
    //     self.start_default();
    // }

    pub fn start(&mut self) {
        let host = cpal::default_host();

        let device = host
            .default_input_device()
            .expect("Failed to get default input device");

        info!("Audio input device selected: {}", device.name().unwrap());

        let config = &cpal::StreamConfig {
            channels: self.channels,
            buffer_size: cpal::BufferSize::Fixed(self.buffer_size),
            sample_rate: cpal::SampleRate(self.sample_rate),
        };

        dbg!(config);

        let err_fn = move |err| {
            error!("Error on audio input stream: {}", err);
        };

        let stream = device.build_input_stream(
            config,
            move |data: &[f32], inp: &cpal::InputCallbackInfo| {
                //dbg!(data);
            },
            err_fn,
        );
        std::thread::sleep(std::time::Duration::from_secs(120));
        // dbg!(stream.unwrap().play());
        // std::thread::sleep(std::time::Duration::from_secs(120));
        // let host = cpal::default_host();
        // let device = host
        //     .default_input_device()
        //     .expect("Failed to get default input device");
        // println!("Default input device: {}", device.name().unwrap());
        // println!("{:?}", device.name());
        // let config = &cpal::StreamConfig {
        //     channels: self.channels,
        //     buffer_size: cpal::BufferSize::Fixed(self.buffer_size),
        //     sample_rate: cpal::SampleRate(self.sample_rate),
        // };
        // dbg!(config);
        // let err_fn = move |err| {
        //     eprintln!("an error occurred on stream: {}", err);
        // };

        // let stream = device.build_input_stream(
        //     config,
        //     move |data: &[f32], inp: &cpal::InputCallbackInfo| {
        //         dbg!(data);
        //     },
        //     err_fn,
        // );

        // std::thread::sleep(std::time::Duration::from_secs(120));
        // drop(stream);
    }
}

pub struct Processing {}

impl Processing {
    pub fn new() -> Processing {
        Processing {}
    }

    pub fn run(&mut self, input: &mut AudioInput) {
        input.start();
        // input.start(|input: &[f32], info: &cpal::InputCallbackInfo| {
        //     dbg!("HELLO");
        //     dbg!(input);
        // });

        std::thread::sleep(std::time::Duration::from_secs(120));
    }

    //fn process(&mut self, input: &[f32], info: &cpal::InputCallbackInfo) {}
}
