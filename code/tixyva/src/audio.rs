use anyhow::Context;
use cpal::{
    traits::{DeviceTrait, EventLoopTrait, HostTrait},
    StreamData, UnknownTypeInputBuffer,
};

use std::{
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
};

/// Join handle and processor for audio data.
pub struct AudioHandler {
    /// Is None if something went wrong when initializing
    handle: Option<JoinHandle<()>>,
    recent_value: Arc<RwLock<f32>>,
}

impl AudioHandler {
    /// Create threads and open microphone.
    /// If a problem occurs it will only ever send 0.
    pub fn new() -> Self {
        // i wrote pretty much this same code for Mega
        // https://github.com/gamma-delta/mega/blob/master/src/audio/mod.rs#L92
        // however this time i am not completely delirious.

        let recent_value = Arc::new(RwLock::new(0.0));

        let handle: anyhow::Result<_> = try {
            let sender = recent_value.clone();

            let audio_host = cpal::default_host();
            let audio_in = audio_host
                .default_input_device()
                .context("No input device")?;
            let mic_format = audio_in
                .supported_input_formats()
                .unwrap()
                .next()
                .expect("microphone supports no inputs?")
                .with_max_sample_rate();
            // Start the input stream
            let mic_event_loop = audio_host.event_loop();
            let mic_stream_id = mic_event_loop
                .build_input_stream(&audio_in, &mic_format)
                .expect("The mic's format wasn't supported?");
            mic_event_loop
                .play_stream(mic_stream_id)
                .expect("failed to play mic stream");

            // cpal hijacks the thread
            thread::spawn(move || {
                mic_event_loop.run(move |_id, result| {
                    let data = match result {
                        Ok(data) => data,
                        Err(oh_no) => {
                            eprintln!("Error in audio! {}", oh_no);
                            return;
                        }
                    };
                    let raw_buffer = match data {
                        StreamData::Input {
                            buffer: UnknownTypeInputBuffer::F32(buffer),
                        } => buffer,
                        _ => panic!("Unknown mic stream data type ;("),
                    };

                    let average = raw_buffer.iter().fold(0.0, |acc, x| acc + x.abs())
                        / raw_buffer.len() as f32;
                    *sender.write().unwrap() = average;
                });
            })
        };
        let handle = match handle {
            Ok(it) => Some(it),
            Err(ono) => {
                eprintln!("Error when initializing audio! {:?}", ono);
                None
            }
        };

        Self {
            handle,
            recent_value,
        }
    }

    pub fn get_volume(&self) -> f64 {
        if self.handle.is_some() {
            *self.recent_value.read().unwrap() as f64
        } else {
            0.0
        }
    }
}
