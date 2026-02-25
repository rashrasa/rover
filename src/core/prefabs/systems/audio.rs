use rodio::{Decoder, OutputStream, Sink};
use std::{fs::File, time::Duration};
use winit::keyboard::KeyCode;

use crate::core;

pub struct AudioSystem {
    sink: Sink,
    _stream_handle: OutputStream,
}

impl AudioSystem {
    pub fn new() -> Self {
        let stream_handle = rodio::OutputStreamBuilder::open_default_stream().unwrap();
        let sink = rodio::Sink::connect_new(&stream_handle.mixer());
        sink.pause();
        if crate::core::MUTE {
            sink.set_volume(0.0);
        } else {
            sink.set_volume(0.2);
        }
        sink.append(
            // TODO: Currently hardcoded to example audio.
            Decoder::try_from(File::open("examples/rover/assets/engine.wav").unwrap()).unwrap(),
        );

        Self {
            sink,
            _stream_handle: stream_handle,
        }
    }
}

impl core::System for AudioSystem {
    fn before_tick(&mut self, args: &mut core::BeforeTickArgs) {
        if *args.input.is_pressed(&KeyCode::KeyW)
            | *args.input.is_pressed(&KeyCode::KeyA)
            | *args.input.is_pressed(&KeyCode::KeyS)
            | *args.input.is_pressed(&KeyCode::KeyD)
        {
            if *args.input.is_pressed(&KeyCode::ControlLeft) {
                self.sink.set_speed(2.0);
            } else {
                self.sink.set_speed(1.0);
            }
            self.sink.play();
            if self.sink.get_pos() > Duration::new(5, 0) {
                self.sink.try_seek(Duration::ZERO).unwrap();
            }
        } else {
            self.sink.set_speed(1.0);
            self.sink.pause();
        }
    }
}
