use rand::random;

use crate::constants::{
    FONTSET, FONTSET_SIZE, NUM_KEYS, NUM_REGS, RAM_SIZE, SCREEN_HEIGHT, SCREEN_WIDTH, STACK_SIZE,
    START_ADDR,
};

pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],

    // Registers
    v_reg: [u8; NUM_REGS],
    i_reg: u16,

    // Stack
    sp: u16,
    stack: [u16; STACK_SIZE],

    // Timers
    keys: [bool; NUM_KEYS],
    dt: u8,
    st: u8,
}

impl Default for Emu {
    fn default() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            // 0-initialize all registers by default
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        // Copy built in characters
        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emu
    }
}

impl Emu {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn tick(&mut self) {
        // Fetch
        let op = self.fetch();

        // Decode

        // Execute
        self.execute(op);
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st == 1 {
            // BEEP
            todo!();
        }

        if self.st > 0 {
            self.st -= 1;
        }
    }

    fn fetch(&mut self) -> u16 {
        debug_assert!(
            (self.pc as usize) < RAM_SIZE - 1,
            "program counter out of bounds!"
        );

        // Assume big endian order, so lower address = high byte, higher address = low byte
        // Each byte is a u8 but the higher byte needs to be left shifted into place in a u16
        let higher_byte = self.ram[self.pc as usize];
        let lower_byte = self.ram[(self.pc + 1) as usize];

        let op = u16::from_be_bytes([higher_byte, lower_byte]);
        self.pc += 2;

        op
    }

    fn execute(&mut self, op: u16) {
        // Split 2 byte operation into 4 nibbles (4 bits each).
        // &-with 0xF to remove extraneous data
        let nibbles = [op >> 12, op >> 8, op >> 4, op].map(|nibble| (nibble & 0xF) as u8);

        match nibbles {
            // NOP - Nothing
            [0, 0, 0, 0] => return,

            // CLS - Clear screen
            [0, 0, 0xE, 0] => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            }

            // RET - Return from subroutine
            [0, 0, 0xE, 0xE] => {
                let ret_addr = self.pop();
                self.pc = ret_addr;
            }

            // 1NNN - Jump
            [1, _, _, _] => {
                let nnn = op & 0xFFF;
                self.pc = nnn;
            }

            // 2NNN - Call subroutine
            [2, _, _, _] => {
                let nnn = op & 0xFFF;

                self.push(self.pc); // So we can return later
                self.pc = nnn;
            }

            // 3XNN - Skip next if VX == NN
            [3, _, _, _] => {
                let x = nibbles[1];
                let nn = (op & 0xFF) as u8;

                if self.v_reg[x as usize] == nn {
                    self.pc += 2;
                }
            }

            // 4XNN - Skip next if VX != NN
            [4, _, _, _] => {
                let x = nibbles[1];
                let nn = (op & 0xFF) as u8;

                if self.v_reg[x as usize] != nn {
                    self.pc += 2;
                }
            }

            // 5XY0 - Skip next if VX == VY
            [5, _, _, 0] => {
                let x = nibbles[1];
                let y = nibbles[2];

                if self.v_reg[x as usize] == self.v_reg[y as usize] {
                    self.pc += 2;
                }
            }

            // 6XNN - VX = NN
            [6, _, _, _] => {
                let x = nibbles[1];
                let nn = (op & 0xFF) as u8;

                self.v_reg[x as usize] = nn;
            }

            // 7XNN - VX += NN
            [7, _, _, _] => {
                let x = nibbles[1];
                let nn = (op & 0xFF) as u8;

                self.v_reg[x as usize] = self.v_reg[x as usize].wrapping_add(nn);
            }

            // 8XY0 - VX = VY
            [8, _, _, 0] => {
                let x = nibbles[1];
                let y = nibbles[2];

                self.v_reg[x as usize] = self.v_reg[y as usize];
            }

            // 8XY1 - VX |= VY
            [8, _, _, 1] => {
                let x = nibbles[1];
                let y = nibbles[2];

                self.v_reg[x as usize] |= self.v_reg[y as usize];
            }

            // 8XY2 - VX &= VY
            [8, _, _, 2] => {
                let x = nibbles[1];
                let y = nibbles[2];

                self.v_reg[x as usize] &= self.v_reg[y as usize];
            }

            // 8XY3 - VX ^= VY
            [8, _, _, 3] => {
                let x = nibbles[1];
                let y = nibbles[2];

                self.v_reg[x as usize] ^= self.v_reg[y as usize];
            }

            // 8XY4 - VX += VY
            // Needs to handle overflow and set the carry flag (register VF)
            [8, _, _, 4] => {
                let x = nibbles[1] as usize;
                let y = nibbles[2] as usize;

                let (new_vx, overflowed) = self.v_reg[x].overflowing_add(self.v_reg[y]);

                // true - 1, false - 0
                let new_vf = overflowed as u8;

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }

            // 8XY5 - VX -= VY
            // Needs to handle underflow and set the carry flag (register VF)
            [8, _, _, 5] => {
                let x = nibbles[1] as usize;
                let y = nibbles[2] as usize;

                let (new_vx, underflowed) = self.v_reg[x].overflowing_sub(self.v_reg[y]);

                // true - 0, false - 1
                let new_vf = !underflowed as u8;

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }

            // 8XY6 - VX >>= 1
            // Assuming the Cowgod specification, VY is ignored here
            // LSB of original VX is stored in VF.
            [8, _, _, 6] => {
                let x = nibbles[1];

                let lsb = self.v_reg[x as usize] & 1;

                self.v_reg[x as usize] >>= 1;
                self.v_reg[0xF] = lsb;
            }

            // 8XY7 - VX = VY - VX
            // Needs to handle underflow and set the carry flag (register VF)
            [8, _, _, 7] => {
                let x = nibbles[1] as usize;
                let y = nibbles[2] as usize;

                let (new_vx, underflowed) = self.v_reg[y].overflowing_sub(self.v_reg[x]);

                // true - 0, false - 1
                let new_vf = !underflowed as u8;

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }

            // 8XYE - VX <<= 1
            // Assuming the Cowgod specification, VY is ignored here
            // MSB of original VX is stored in VF.
            [8, _, _, 0xE] => {
                let x = nibbles[1];

                // Mask isn't necessary but good to ensure we only get one bit
                let msb = (self.v_reg[x as usize] >> 7) & 1;

                self.v_reg[x as usize] <<= 1;
                self.v_reg[0xF] = msb;
            }

            // 9XY0 - Skip next if VX != VY
            [9, _, _, 0] => {
                let x = nibbles[1];
                let y = nibbles[2];

                if self.v_reg[x as usize] != self.v_reg[y as usize] {
                    self.pc += 2;
                }
            }

            // ANNN - I = NNN
            [0xA, _, _, _] => {
                let nnn = op & 0xFFF;
                self.i_reg = nnn;
            }

            // BNNN - Jump to V0 + NNN
            [0xB, _, _, _] => {
                let nnn = op & 0xFFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            }

            // CXNN - VX = rand() & NN
            [0xC, _, _, _] => {
                let x = nibbles[1];
                let nn = (op & 0xFF) as u8;

                let rng: u8 = random();

                self.v_reg[x as usize] = rng & nn;
            }

            [_, _, _, _] => unimplemented!("Unimplemented opcode: {op}"),
        }
    }

    fn push(&mut self, val: u16) {
        debug_assert!((self.sp as usize) < STACK_SIZE, "stack pointer overflowed!");

        self.stack[self.sp as usize] = val;

        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        debug_assert!((self.sp as usize) > 0, "stack pointer underflowed!");

        self.sp -= 1;

        self.stack[self.sp as usize]
    }
}
