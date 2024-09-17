//! A basic implementation of `Chip8IO` using `piston` for graphics and input, and `ears` for sound
//! Press `Escape` to exit the emulator

// FIXME: Cannot quit the emulator
// NOTE: Maybe not, do further testing (might just be slow in debug mode)

extern crate piston_window;
extern crate ears;

use std::path::Path;

use self::piston_window::*;
use self::ears::{Sound, AudioController};
use super::{SCREEN_WIDTH, SCREEN_HEIGHT};

/// The size of each pixel (in pixels)
const PIXEL_SIZE: usize = 10;

/// Stores state used for doing I/O
#[allow(missing_debug_implementations)]
pub struct Io {
    window: PistonWindow,
    keys: ::Keys,
    should_close: bool,
    sound: Sound,
}

impl Io {
    /// Initializes the state, creating the window and sound data
    /// Requires a path to a sound file, used for playing sounds
    /// The sound file must be in a format recognized by `ears`, for example wav or ogg
    pub fn new<P: AsRef<Path>>(sound_path: P) -> Io {
        let window: PistonWindow = WindowSettings::new("Chip-8 Emulator",
                                                       [(SCREEN_WIDTH * PIXEL_SIZE) as u32,
                                                        (SCREEN_HEIGHT * PIXEL_SIZE) as u32])
            .build()
            .unwrap();

        let path = sound_path.as_ref().to_str().unwrap_or_else(|| {
            panic!("Path to sound file was invalid");
        });

        let sound = Sound::new(path).unwrap_or_else(|| {
            panic!("Failed to create sound from file: {}", path);
        });

        Io {
            window: window,
            keys: [false; 16],
            should_close: false,
            sound: sound,
        }
    }

    /// A helper function to detect keyboard input and when to close the window
    fn handle_event(&mut self, event: &Event) {
        if let Event::Input(ref input) = *event {
            match *input {
                Input::Press(button) => self.set_key(button, true),
                Input::Release(button) => self.set_key(button, false),
                _ => {}
            }
        }
    }

    /// Handles a key press, setting the keyboard state
    fn set_key(&mut self, button: Button, state: bool) {
        if let Button::Keyboard(key) = button {
            let button = match key {
                Key::D1 => 0x0,
                Key::D2 => 0x1,
                Key::D3 => 0x2,
                Key::D4 => 0xC,
                Key::Q => 0x4,
                Key::W => 0x5,
                Key::E => 0x6,
                Key::R => 0xD,
                Key::A => 0x7,
                Key::S => 0x8,
                Key::D => 0x9,
                Key::F => 0xE,
                Key::Z => 0xA,
                Key::X => 0x0,
                Key::C => 0xB,
                Key::V => 0xF,
                Key::Escape => {
                    self.should_close = true;
                    return;
                }
                _ => return,
            };

            self.keys[button] = state;
        }
    }
}

impl ::Chip8IO for Io {
    fn draw(&mut self, pixels: &[bool]) {
        // Handle all events
        while let Some(e) = self.window.next() {
            match e {
                // If idling, don't draw anything
                Event::Idle(_) => {
                    return;
                }
                // Detect if the window has closed
                Event::Input(Input::Close) => {
                    self.should_close = true;
                    return;
                }
                _ => {}
            }

            // Update keyboard state
            self.handle_event(&e);

            // Draw the display
            self.window.draw_2d(&e, |c, g| {
                // Clear the screen with black
                clear([0.0; 4], g);

                // Iterate through each pixel, get its coordinates and draw a square at its location
                for x in 0..SCREEN_WIDTH {
                    for y in 0..SCREEN_HEIGHT {
                        let pixel = pixels[x + y * SCREEN_WIDTH];

                        // White if the pixel is on, black otherwise
                        let color = if pixel { [1.0; 4] } else { [0.0; 4] };
                        let size = PIXEL_SIZE as f64;
                        let screen_x = (x * PIXEL_SIZE) as f64;
                        let screen_y = (y * PIXEL_SIZE) as f64;

                        // Draw a square for the pixel
                        rectangle(color, [screen_x, screen_y, size, size], c.transform, g);
                    }
                }
            });
        }
    }

    fn play_sound(&mut self) {
        self.sound.play();
    }

    fn get_keys(&mut self) -> ::Keys {
        while let Some(e) = self.window.next() {
            if let Event::Idle(_) = e {
                return self.keys;
            }
            self.handle_event(&e);
        }

        self.keys
    }

    fn should_close(&self) -> bool {
        // The `handle_event` function detects when the emulator should, this just returns the
        // flag
        self.should_close
    }
}
