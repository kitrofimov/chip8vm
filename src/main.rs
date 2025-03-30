use std::fs::File;
use std::io::Read;

struct VM {
    running: bool,
    ram: [u8; 4096],
    pc: usize,
    reg: [u8; 16],
    reg_i: u16,
    stack: [u16; 16],
    sp: usize,
    delay_timer: u8,
    sound_timer: u8,
}

impl VM {
    fn new() -> VM {
        VM {
            running: true,
            ram: [0; 4096],
            pc: 0x200,
            reg: [0; 16],
            reg_i: 0,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    fn mainloop(&mut self) {
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

    fn execute(&mut self, opcode: u16) {
        let nnn = opcode & 0x0FFF;
        let nn = (opcode & 0x00FF) as u8;
        let n = (opcode & 0x000F) as u8;
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;

        // TODO: Implement the opcode execution logic
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_rom>", args[0]);
        std::process::exit(1);
    }
    let mut file = File::open(&args[1]).expect("Failed to open ROM file");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .expect("Failed to read ROM file");

    let mut vm = VM::new();
    vm.ram[0x200..0x200 + buffer.len()].copy_from_slice(&buffer);

    println!("Loaded {} bytes into RAM (address 0x200)", buffer.len());
    println!("Starting VM...");

    vm.mainloop();
}
