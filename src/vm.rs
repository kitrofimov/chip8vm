use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use sdl2::event::Event;
use sdl2::keyboard::Scancode;

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;

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
    display: [[u8; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
    canvas: Canvas<Window>,
    texture: Texture<'a>,
    sdl_context: sdl2::Sdl,
}

impl<'a> VM<'a> {
    pub fn new(sdl_context: sdl2::Sdl, canvas: Canvas<Window>, texture: Texture) -> VM {
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
            display: [[0; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
            canvas,
            texture,
            sdl_context,
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

    pub fn load_program(&mut self, program: &[u8]) {
        self.ram[0x200..0x200 + program.len()].copy_from_slice(program);
    }

    pub fn mainloop(&mut self) {
        while self.running {
            let fetched = self.fetch();
            self.execute(fetched);
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
        for byte in 0..n {
            let y_coord = (y as usize + byte as usize) % DISPLAY_HEIGHT;
            let sprite_byte = self.ram[self.reg_i as usize + byte as usize];
            for bit in 0..8 {
                let x_coord = (x as usize + bit) % DISPLAY_WIDTH;
                let sprite_pixel = (sprite_byte >> (7 - bit)) & 1;
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
        let event_pump = self.sdl_context.event_pump().unwrap();
        let keyboard_state = event_pump.keyboard_state();
        keyboard_state.is_scancode_pressed(VM::chip8_key_to_scancode(chip8_key))
    }

    fn wait_for_key_press(&mut self) -> u8 {
        let mut event_pump = self.sdl_context.event_pump().unwrap();
        loop {
            for event in event_pump.poll_iter() {
                if let Event::KeyDown { scancode: Some(scancode), .. } = event {
                    if let Some(chip8_key) = VM::scancode_to_chip8_key(scancode) {
                        return chip8_key;
                    }
                }
            }
        }
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
                0x1 => self.reg[x] |= self.reg[y], // 8xy1
                0x2 => self.reg[x] &= self.reg[y], // 8xy2
                0x3 => self.reg[x] ^= self.reg[y], // 8xy3
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
                    self.reg[0xF] = self.reg[x] & 0x1;
                    self.reg[x] >>= 1;
                }
                0x7 => { // 8xy7
                    let (result, borrow) = self.reg[y].overflowing_sub(self.reg[x]);
                    self.reg[x] = result;
                    self.reg[0xF] = if borrow { 0 } else { 1 };
                }
                0xE => { // 8xyE
                    self.reg[x] = self.reg[y];
                    self.reg[0xF] = (self.reg[x] & 0x80) >> 7;
                    self.reg[x] <<= 1;
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
                0x0A => self.reg[x] = self.wait_for_key_press(), // Fx0A
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
