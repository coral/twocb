use cpal;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::unbounded;
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

    pub fn start(&mut self, p: &'static mut Processing) {
        let host = cpal::default_host();

        //let (s, r) = unbounded();

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
                println!("ok");
                for i in data.iter() {
                    println!("{}", i);
                }
            },
            err_fn,
        );
        std::thread::sleep(std::time::Duration::from_secs(120));
    }

    pub fn process(&mut self, input: &[f32], inp: &cpal::InputCallbackInfo) {
        for n in input.iter() {
            println!("{}", n)
        }
    }
}

pub struct Processing {}

impl Processing {
    pub fn new() -> Processing {
        Processing {}
    }

    pub fn run<'a>(&mut self, input: &'a mut AudioInput) {
        input.start();
        //input.start(move |data, inp: &cpal::InputCallbackInfo| self.process(data, inp));

        std::thread::sleep(std::time::Duration::from_secs(120));
    }

    fn process(&mut self, input: &[f32], info: &cpal::InputCallbackInfo) {
        for n in input.iter() {
            println!("{}", n)
        }
    }
}
