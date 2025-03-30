struct Stack {
    sp: usize,
    stack: [i32; 256]
}

impl Stack {
    fn push(&mut self, value: i32) {
        self.stack[self.sp] = value;
        self.sp += 1;
    }

    fn pop(&mut self) -> i32 {
        self.sp -= 1;
        self.stack[self.sp]
    }
}

struct Instruction {
    opcode: i32,
    argument: Option<i32>
}

struct VM {
    running: bool,
    stack: Stack,
    ip: usize,
    program: [i32; 256]
}

impl VM {
    fn mainloop(&mut self) {
        while self.running {
            let fetched = self.fetch();
            self.execute(fetched);
            self.ip += 1;
        }
    }

    fn fetch(&mut self) -> Instruction {
        let opcode = self.program[self.ip];
        let mut argument = None;
        if opcode == PSH {
            self.ip += 1;
            argument = Some(self.program[self.ip]);
        }
        Instruction { opcode, argument }
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction.opcode {
            PSH => self.stack.push(instruction.argument.expect("PSH must fetch an argument")),
            ADD => {
                let res = self.stack.pop() + self.stack.pop();
                self.stack.push(res);
            },
            POP => println!("{}", self.stack.pop()),
            HLT => self.running = false,
            other => panic!("Unknown instruction: {:#X}; IP: {:#X}", other, self.ip)
        }
    }
}

const PSH: i32 = 0;
const ADD: i32 = 1;
const POP: i32 = 2;
const HLT: i32 = 3;

fn main() {
    let mut program = [0; 256];
    program[..7].copy_from_slice(&[
        PSH, 5,
        PSH, 6,
        ADD,
        POP,
        HLT
    ]);

    let mut vm = VM {
        running: true,
        stack: Stack {
            sp: 0,
            stack: [0; 256]
        },
        ip: 0,
        program
    };

    vm.mainloop();
}
