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
