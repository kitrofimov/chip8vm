//! A CHIP-8 interpreter
//! 
//! This module implements a CHIP-8 interpreter, which is a virtual machine
//! that can run CHIP-8 programs. It uses the SDL2 library for graphics and
//! audio handling.

use std::time::{Duration, Instant};
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::{AudioSubsystem, EventPump};

/// The width of the display in pixels
pub const DISPLAY_WIDTH: usize = 64;
/// The height of the display in pixel
pub const DISPLAY_HEIGHT: usize = 32;

const VM_FREQUENCY: u32 = 500;
const TIMER_FREQUENCY: u32 = 60;

const AUDIO_SAMPLE_RATE: f32 = 44100.0;
const AUDIO_TARGET_FREQUENCY: f32 = 440.0;
const AUDIO_VOLUME: f32 = 0.1;

struct SquareWave {
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for sample in out.iter_mut() {
            self.phase = (self.phase + AUDIO_TARGET_FREQUENCY / AUDIO_SAMPLE_RATE) % 1.0;

            *sample = if self.phase < 0.5 {
                self.volume
            } else {
                -self.volume
            };
        }
    }
}

/// Stucture representing the state of the virtual machine
pub struct VM<'a> {
    running: bool,
    ram: [u8; 4096],
    pc: usize,
    reg: [u8; 16],
    reg_i: u16,
    stack: [u16; 16],
    sp: usize,
    delay_timer: u8,
    sound_timer: u8,
    waiting_for_key: Option<usize>,
    display: [[u8; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
    event_pump: sdl2::EventPump,
    canvas: Canvas<Window>,
    texture: Texture<'a>,
    audio_device: AudioDevice<SquareWave>,
}

impl<'a> VM<'a> {
    /// Create a new virtual machine
    pub fn new(
        canvas: Canvas<Window>,
        texture: Texture,
        event_pump: EventPump,
        audio: AudioSubsystem
    ) -> VM {
        let desired_spec = AudioSpecDesired {
            freq: Some(AUDIO_SAMPLE_RATE as i32),
            channels: Some(1),
            samples: None,
        };
        let audio_device = audio
            .open_playback(None, &desired_spec, |_spec| SquareWave {
                phase: 0.0,
                volume: 0.0,
            })
            .unwrap();
        audio_device.resume();

        let mut vm = VM {
            running: true,
            ram: [0; 4096],
            pc: 0x200,
            reg: [0; 16],
            reg_i: 0,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            waiting_for_key: None,
            display: [[0; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
            event_pump,
            canvas,
            texture,
            audio_device
        };

        let font_data: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];
        vm.ram[0x000..0x050].copy_from_slice(&font_data);

        vm
    }

    /// Load a program into the virtual machine
    pub fn load_program(&mut self, program: &[u8]) {
        self.ram[0x200..0x200 + program.len()].copy_from_slice(program);
    }

    /// Start the main loop of the virtual machine
    pub fn mainloop(&mut self) {
        let mut last_timer_update = Instant::now();
        let cycle_duration = Duration::from_secs_f64(1.0 / (VM_FREQUENCY as f64));

        while self.running {
            let cycle_start = Instant::now();

            if last_timer_update.elapsed() >= Duration::from_secs_f64(1.0 / TIMER_FREQUENCY as f64) {
                if self.delay_timer > 0 {
                    self.delay_timer -= 1;
                }
                if self.sound_timer >= 1 {
                    self.sound_timer -= 1;
                    self.audio_device.lock().volume = AUDIO_VOLUME;
                } else {
                    self.audio_device.lock().volume = 0.0;
                }
                last_timer_update = Instant::now();
            }

            for event in self.event_pump.poll_iter() {
                if let Event::KeyDown { scancode: Some(Scancode::Escape), .. } = event {
                    self.running = false;
                }
                if let Event::KeyUp { scancode: Some(scancode), .. } = event {
                    if let Some(register) = self.waiting_for_key {
                        if let Some(chip8_key) = VM::scancode_to_chip8_key(scancode) {
                            self.reg[register] = chip8_key;
                            self.waiting_for_key = None;
                        }
                    }
                }
            }

            // Do not process opcodes while waiting for a key
            // Is not inside the upper loop because there may be no events
            if self.waiting_for_key.is_some() {
                continue;
            }

            print!("PC: 0x{:X} ", self.pc);
            let fetched = self.fetch();
            println!("OPCODE: 0x{:X}", fetched);
            self.execute(fetched);

            let elapsed = cycle_start.elapsed();
            if elapsed < cycle_duration {
                std::thread::sleep(cycle_duration - elapsed);
            }
        }
    }

    fn push(&mut self, value: u16) {
        self.stack[self.sp] = value;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp]
    }

    fn fetch(&mut self) -> u16 {
        let high = self.ram[self.pc] as u16;
        let low = self.ram[self.pc + 1] as u16;
        self.pc += 2;
        (high << 8) | low
    }

    fn clear_screen(&mut self) {
        self.display = [[0; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
        self.render_display();
    }

    fn draw_sprite(&mut self, x: u8, y: u8, n: u8) {
        self.reg[0xF] = 0;
        let x = x % DISPLAY_WIDTH as u8;
        let y = y % DISPLAY_HEIGHT as u8;
        for byte in 0..n {
            let y_coord = y as usize + byte as usize;
            if y_coord >= DISPLAY_HEIGHT {
                break;
            }
            let sprite_byte = self.ram[self.reg_i as usize + byte as usize];
            for bit in 0..8 {
                let x_coord = x as usize + bit;
                if x_coord >= DISPLAY_WIDTH {
                    break;
                }
                let sprite_pixel = match (sprite_byte >> (7 - bit)) & 1 {
                    0 => 0,
                    1 => 0xFF,
                    _ => unreachable!()
                };
                let screen_pixel = &mut self.display[y_coord][x_coord];
                if *screen_pixel == 1 && sprite_pixel == 1 {
                    self.reg[0xF] = 1;
                }
                *screen_pixel ^= sprite_pixel;
            }
        }
        self.render_display();
    }

    fn render_display(&mut self) {
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();

        let (canvas_width, canvas_height) = self.canvas.output_size().unwrap();
        let dest_rect = Rect::new(0, 0, canvas_width, canvas_height);

        self.texture
            .update(None, self.display.as_flattened(), DISPLAY_WIDTH)
            .unwrap();
        self.canvas
            .copy(&self.texture, None, Some(dest_rect))
            .unwrap();
        self.canvas.present();
    }

    // 1 2 3 C        1 2 3 4
    // 4 5 6 D  <==>  Q W E R
    // 7 8 9 E  <==>  A S D F
    // A 0 B F        Z X C V
    fn scancode_to_chip8_key(scancode: Scancode) -> Option<u8> {
        match scancode {
            Scancode::Num1 => Some(0x1),
            Scancode::Num2 => Some(0x2),
            Scancode::Num3 => Some(0x3),
            Scancode::Num4 => Some(0xC),
            Scancode::Q => Some(0x4),
            Scancode::W => Some(0x5),
            Scancode::E => Some(0x6),
            Scancode::R => Some(0xD),
            Scancode::A => Some(0x7),
            Scancode::S => Some(0x8),
            Scancode::D => Some(0x9),
            Scancode::F => Some(0xE),
            Scancode::Z => Some(0xA),
            Scancode::X => Some(0x0),
            Scancode::C => Some(0xB),
            Scancode::V => Some(0xF),
            _ => None,
        }
    }

    fn chip8_key_to_scancode(chip8_key: u8) -> Scancode {
        match chip8_key {
            0x1 => Scancode::Num1,
            0x2 => Scancode::Num2,
            0x3 => Scancode::Num3,
            0xC => Scancode::Num4,
            0x4 => Scancode::Q,
            0x5 => Scancode::W,
            0x6 => Scancode::E,
            0xD => Scancode::R,
            0x7 => Scancode::A,
            0x8 => Scancode::S,
            0x9 => Scancode::D,
            0xE => Scancode::F,
            0xA => Scancode::Z,
            0x0 => Scancode::X,
            0xB => Scancode::C,
            0xF => Scancode::V,
            _ => panic!("Invalid CHIP-8 key: {}", chip8_key),
        }
    }

    fn is_key_pressed(&self, chip8_key: u8) -> bool {
        let keyboard_state = self.event_pump.keyboard_state();
        keyboard_state.is_scancode_pressed(VM::chip8_key_to_scancode(chip8_key))
    }

    fn execute(&mut self, opcode: u16) {
        let nnn = opcode & 0x0FFF;
        let nn = (opcode & 0x00FF) as u8;
        let n = (opcode & 0x000F) as u8;
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;

        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => self.clear_screen(), // 00E0
                0x00EE => self.pc = self.pop() as usize, // 00EE
                _ => panic!("0x0NNN (execute native subroutine) called!") // 0nnn
            },
            0x1000 => self.pc = nnn as usize, // 1nnn
            0x2000 => { // 2nnn
                self.push(self.pc as u16);
                self.pc = nnn as usize;
            }
            0x3000 => { // 3xnn
                if self.reg[x] == nn {
                    self.pc += 2;
                }
            }
            0x4000 => {
                if self.reg[x] != nn { // 4xnn
                    self.pc += 2;
                }
            }
            0x5000 => {
                if self.reg[x] == self.reg[y] { // 5xy0
                    self.pc += 2;
                }
            }
            0x6000 => self.reg[x] = nn, // 6xnn
            0x7000 => self.reg[x] = self.reg[x].wrapping_add(nn), // 7xnn
            0x8000 => match opcode & 0x000F {
                0x0 => self.reg[x] = self.reg[y], // 8xy0
                0x1 => { // 8xy1
                    self.reg[x] |= self.reg[y];
                    self.reg[0xF] = 0;
                }
                0x2 => { // 8xy2
                    self.reg[x] &= self.reg[y];
                    self.reg[0xF] = 0;
                }
                0x3 => { // 8xy3
                    self.reg[x] ^= self.reg[y];
                    self.reg[0xF] = 0;
                }
                0x4 => { // 8xy4
                    let (result, carry) = self.reg[x].overflowing_add(self.reg[y]);
                    self.reg[x] = result;
                    self.reg[0xF] = if carry { 1 } else { 0 };
                }
                0x5 => { // 8xy5
                    let (result, borrow) = self.reg[x].overflowing_sub(self.reg[y]);
                    self.reg[x] = result;
                    self.reg[0xF] = if borrow { 0 } else { 1 };
                }
                0x6 => { // 8xy6
                    self.reg[x] = self.reg[y];
                    if x == 0xF {
                        self.reg[x] &= 0x1;
                    }
                    else {
                        self.reg[0xF] = self.reg[x] & 0x1;
                        self.reg[x] >>= 1;
                    }
                }
                0x7 => { // 8xy7
                    let (result, borrow) = self.reg[y].overflowing_sub(self.reg[x]);
                    self.reg[x] = result;
                    self.reg[0xF] = if borrow { 0 } else { 1 };
                }
                0xE => { // 8xyE
                    self.reg[x] = self.reg[y];
                    self.reg[0xF] = (self.reg[x] & 0x80) >> 7;
                    if x != 0xF {
                        self.reg[x] <<= 1;
                    }
                }
                _ => {}
            },
            0x9000 => { // 9xy0
                if self.reg[x] != self.reg[y] {
                    self.pc += 2;
                }
            }
            0xA000 => self.reg_i = nnn, // Annn
            0xB000 => self.pc = (nnn + self.reg[0] as u16) as usize, // Bnnn
            0xC000 => self.reg[x] = rand::random::<u8>() & nn, // Cxnn
            0xD000 => self.draw_sprite(self.reg[x], self.reg[y], n), // Dxyn
            0xE000 => match opcode & 0x00FF {
                0x9E => { // Ex9E
                    if self.is_key_pressed(self.reg[x]) {
                        self.pc += 2;
                    }
                }
                0xA1 => { // ExA1
                    if !self.is_key_pressed(self.reg[x]) {
                        self.pc += 2;
                    }
                }
                _ => {}
            },
            0xF000 => match opcode & 0x00FF {
                0x07 => self.reg[x] = self.delay_timer, // Fx07
                0x0A => self.waiting_for_key = Some(x), // Fx0A
                0x15 => self.delay_timer = self.reg[x], // Fx15
                0x18 => self.sound_timer = self.reg[x], // Fx18
                0x1E => self.reg_i = self.reg_i.wrapping_add(self.reg[x] as u16), // Fx1E
                0x29 => self.reg_i = (self.reg[x] as u16) * 5, // Fx29
                0x33 => { // Fx33
                    self.ram[self.reg_i as usize] = self.reg[x] / 100;
                    self.ram[self.reg_i as usize + 1] = (self.reg[x] / 10) % 10;
                    self.ram[self.reg_i as usize + 2] = self.reg[x] % 10;
                }
                0x55 => { // Fx55
                    for i in 0..=x {
                        self.ram[self.reg_i as usize] = self.reg[i];
                        self.reg_i = self.reg_i.wrapping_add(1);
                    }
                }
                0x65 => { // Fx65
                    for i in 0..=x {
                        self.reg[i] = self.ram[self.reg_i as usize];
                        self.reg_i = self.reg_i.wrapping_add(1);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}
