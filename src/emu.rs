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

            // DXYN - Draw sprite
            // Draw a sprite starting horizontally at VI to VI + n. Wrap sprites around
            // if necessary. If any pixel is unset, set VF to 1 (or 0 if the opposite
            // occurs.
            [0xD, _, _, _] => {
                let x = nibbles[1];
                let y = nibbles[2];
                let n = nibbles[3]; // Sprites can have a height of 1 - 15.

                let x_coord = self.v_reg[x as usize];
                let y_coord = self.v_reg[y as usize];

                let mut pixel_unset = false;

                for row in 0..n {
                    let address = self.i_reg + (row as u16);
                    let sprite_pixel_row = self.ram[address as usize];

                    for col in 0..8 {
                        let sprite_pixel = (sprite_pixel_row >> col) & 1;

                        if sprite_pixel == 1 {
                            let screen_x = (x_coord + row) as usize % SCREEN_WIDTH;
                            let screen_y = (y_coord + col) as usize % SCREEN_HEIGHT;

                            let screen_idx = SCREEN_WIDTH * screen_y + screen_x;

                            debug_assert!(
                                screen_idx < SCREEN_HEIGHT * SCREEN_HEIGHT,
                                "incorrectly calculated screen index when drawing!"
                            );

                            // Each sprite pixel is going to be XOR'd with the existing
                            // display pixel:
                            // SP  DP
                            // ON  OFF -> ON
                            // ON  ON  -> OFF
                            // OFF ON  -> ON
                            // OFF OFF -> OFF
                            pixel_unset |= self.screen[screen_idx];
                            self.screen[screen_idx] ^= true;
                        }
                    }
                }

                self.v_reg[0xF] = pixel_unset as u8;
            }

            // EX9E - Skip if key pressed
            [0xE, _, 9, 0xE] => {
                let x = nibbles[1];
                let vx = self.v_reg[x as usize];

                let key = self.keys[vx as usize];

                if key {
                    self.pc += 2;
                }
            }

            // EXA1 - Skip if key not pressed
            [0xE, _, 0xA, 1] => {
                let x = nibbles[1];
                let vx = self.v_reg[x as usize];

                let key = self.keys[vx as usize];

                if !key {
                    self.pc += 2;
                }
            }

            // FX07 - VX = DT
            [0xF, _, 0, 7] => {
                let x = nibbles[1];
                self.v_reg[x as usize] = self.dt;
            }

            // FX0A - Wait until key pressed
            // If a key isn't pressed, this moves the program counter back to rerun the
            // instruction. Better than a loop so we can still take input.
            [0xF, _, 0, 0xA] => {
                let x = nibbles[1];

                match self.keys.iter().position(|&key| key) {
                    Some(position) => {
                        self.v_reg[x as usize] = position as u8;
                    }
                    None => {
                        self.pc -= 2;
                    }
                }
            }

            // FX15 - DT = VX
            [0xF, _, 1, 5] => {
                let x = nibbles[1];
                self.dt = self.v_reg[x as usize];
            }

            // FX18 - ST = VX
            [0xF, _, 1, 8] => {
                let x = nibbles[1];
                self.st = self.v_reg[x as usize];
            }

            // FX1E - I += VX
            [0xF, _, 1, 0xE] => {
                let x = nibbles[1];

                let vx = self.v_reg[x as usize];

                self.i_reg = self.i_reg.wrapping_add(vx as u16);
            }

            // FX29 - Set I to font address
            [0xF, _, 2, 9] => {
                let x = nibbles[1];

                let vx = self.v_reg[x as usize];

                self.i_reg = vx as u16 * 5; // Each character is 5 bytes, stored
                                            // starting at address 0.
            }

            // FX33 - I = BCD of VX
            // Take VX, which is at most a 3 digit number and store each individual
            // digit in the I register.
            [0xF, _, 3, 3] => {
                let x = nibbles[1];

                let vx = self.v_reg[x as usize];

                let ones = vx % 10;
                let tens = (vx / 10) % 10;
                let hundreds = vx / 100;

                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            }

            // FX55 - Store V0 through VX into I
            [0xF, _, 5, 5] => {
                let x = nibbles[1] as usize;

                let i = self.i_reg as usize;

                for idx in 0..=x {
                    self.ram[i + idx] = self.v_reg[idx];
                }
            }

            // FX65 - Store I into V0 through VX
            [0xF, _, 6, 5] => {
                let x = nibbles[1] as usize;

                let i = self.i_reg as usize;

                for idx in 0..=x {
                    self.v_reg[idx] = self.ram[i + idx];
                }
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
