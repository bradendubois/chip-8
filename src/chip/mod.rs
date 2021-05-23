use std::collections::HashMap;

use device_query::{DeviceState, DeviceQuery, Keycode};
use rand::Rng;
use std::thread::sleep;
use std::time::Duration;
use std::io;
use std::io::Write;

const F: usize = 0x0F;
const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

pub struct Chip {
    v: [u8; 16],
    pc: u16,
    i: u16,
    delay_timer: u8,
    sound_timer: u8,
    stack: Vec<u16>,
    memory: [u8; 4096],
    display: io::Stdout,
    display_ram: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
    keymap: HashMap<u8, Keycode>,
    device_state: DeviceState,
}


impl Chip {

    pub fn new(data: Vec<u8>, keymap: HashMap<u8, Keycode>) -> Chip {

        let mut memory = [0 as u8; 4096];

        Chip::load_fonts(&mut memory);

        for (i, x) in data.iter().enumerate() {
            memory[i + 0x0200] = *x;
        }

        Chip {
            v: [0; 16],
            pc: 0x0200,
            i: 0x0000,
            delay_timer: 0,
            sound_timer: 0,
            stack: Vec::new(),
            memory,
            display: io::stdout(),
            display_ram: [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
            keymap,
            device_state: DeviceState::new(),
        }
    }

    pub fn run(&mut self) {

        loop {

            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }

            if self.sound_timer > 0 {
                self.sound_timer -= 1;
            }

            let instruction = self.next_instruction();

            self.execute(instruction);

            sleep(Duration::from_millis(2));
        }
    }

    pub fn execute(&mut self, instruction: u16) {

        let l = ((instruction & 0xF000) >> 12) as u8;

        let x = ((instruction & 0x0F00) >> 8) as u8 as usize;
        let y = ((instruction & 0x00F0) >> 4) as u8 as usize;

        let nnn = instruction & 0x0FFF;
        let nn = (instruction & 0x00FF) as u8;
        let n = (instruction & 0x000F) as u8;

        // Match the left-most digit for further matching
        match (l, x, y, n) {

            (0x00, 0x00, 0x0E, 0x00) => self.clear(),
            (0x00, 0x00, 0x0E, 0x0E) => self.ret(),

            (0x01,    _,    _,    _) => self.jump(nnn),
            (0x02,    _,    _,    _) => self.call(nnn),
            (0x03,    _,    _,    _) => if self.v[x] == nn { self.skip() },
            (0x04,    _,    _,    _) => if self.v[x] != nn { self.skip() },
            (0x05,    _,    _,    _) => if self.v[x] == self.v[y] { self.skip() },
            (0x06,    _,    _,    _) => self.v[x] = nn,
            (0x07,    _,    _,    _) => self.v[x] = self.v[x].wrapping_add(nn),

            (0x08,    _,    _, 0x00) => self.v[x]  = self.v[y],
            (0x08,    _,    _, 0x01) => self.v[x] |= self.v[y],
            (0x08,    _,    _, 0x02) => self.v[x] &= self.v[y],
            (0x08,    _,    _, 0x03) => self.v[x] ^= self.v[y],
            (0x08,    _,    _, 0x04) => {
                let (result, overflow) = self.v[x].overflowing_add(self.v[y]);
                self.v[x] = result;
                self.v[F] = match overflow {
                    true  => 1,
                    false => 0,
                };
            },

            (0x08,    _,    _, 0x05) => {
                self.v[x] = self.v[x].wrapping_sub(self.v[y]);
                match self.v[x].checked_sub(self.v[y]) {
                    Some(_) => self.v[F] = 00,
                    None => self.v[F] = 01
                }
            },

            (0x08,    _,    _, 0x06) => {
                self.v[F] = self.v[x] & 0x01;
                self.v[x] >>= 1;
            },

            (0x08,    _,    _, 0x07) => {
                let (result, underflow) = self.v[y].overflowing_sub(self.v[x]);
                self.v[x] = result;
                self.v[F] = match underflow {
                    true  => 00,
                    false => 01
                }
            },

            (0x08,    _,    _, 0x0E) => {
                self.v[F] = (self.v[x] & 0x80) >> 7;
                self.v[x] <<= 1;
            },


            (0x09, _, _, 0x00) => {
                if self.v[x] != self.v[y] {
                    self.next_instruction();
                }
            },

            (0x0A, _, _, _) => self.i = nnn,

            (0x0B, _, _, _) => self.jump(nnn + self.v[0] as u16),

            (0x0C, _, _, _) => self.v[x] = rand::thread_rng().gen_range(0x00 ..= 0xFF) & nn,

            (0x0D, _, _, _) => {

                self.v[F] = 0;

                for row in 0..n {

                    let py = ((self.v[y] + row) % DISPLAY_HEIGHT as u8) as usize;
                    let sprite_row = self.memory[(self.i + (row as u16)) as usize];

                    for column in 0..8 {

                        let px = ((self.v[x] + column) % DISPLAY_WIDTH as u8) as usize;
                        let on = (sprite_row & (0x80 >> column)) > 0;

                        if !(self.display_ram[py][px] ^ on)  {
                            self.v[F] = 1;
                        }

                        self.display_ram[py][px] ^= on;
                    }
                }

                self.draw_screen();
            },

            (0x0E, _, 0x09, 0x0E) => if  self.is_pressed(self.v[x]) { self.skip() },
            (0x0E, _, 0x0A, 0x01) => if !self.is_pressed(self.v[x]) { self.skip() },

            (0x0F, _, 0x00, 0x07) => self.v[x] = self.delay_timer,
            (0x0F, _, 0x00, 0x0A) => {
                loop {
                    let keys = self.device_state.get_keys();
                    for (code, keycode) in self.keymap.iter() {
                        if keys.contains(keycode) {
                            self.v[x] = *code;
                            break;
                        }
                    }
                }
            },
            (0x0F, _, 0x01, 0x05) => self.delay_timer = self.v[x],
            (0x0F, _, 0x01, 0x08) => self.sound_timer = self.v[x],
            (0x0F, _, 0x01, 0x0E) => self.i += self.v[x] as u16,
            (0x0F, _, 0x02, 0x09) => self.i = (self.v[x] * 5) as u16,
            (0x0F, _, 0x03, 0x03) => {
                self.memory[self.i as usize] = self.v[x] / 100;
                self.memory[(self.i + (1 as u16)) as usize] = (self.v[x] / 10) % 10;
                self.memory[(self.i + (2 as u16)) as usize] = self.v[x] % 10;

            },
            (0x0F, _, 0x05, 0x05) => {
                for i in 0..=x {
                    self.memory[(self.i + i as u16) as usize] = self.v[i];
                }
            },
            (0x0F, _, 0x06, 0x05) => {
                for i in 0..=x {
                    self.v[i] = self.memory[(self.i + i as u16) as usize];
                }
            },

            _ => panic!("unknown instruction: {:#06X}, pc = {:#06X}", instruction, self.pc)
        };

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
    }


    /// JUMP - Jump to given instruction
    fn jump(&mut self, instruction: u16) {
        self.pc = instruction;
    }

    /// CALL - Call subroutine at given instruction
    fn call(&mut self, instruction: u16) {
        self.stack.push(self.pc);
        self.jump(instruction);
    }

    /// RET - Return from a subroutine
    fn ret(&mut self) {
        self.pc = self.stack.pop().unwrap();
    }

    /// Get next instruction (and advance PC by 2 in doing so)
    fn next_instruction(&mut self) -> u16 {
        let big = self.memory[(self.pc % 4096) as usize];
        let little = self.memory[(self.pc.wrapping_add(1)  % 4096) as usize];
        self.pc = self.pc.wrapping_add(2);
        ((big as u16) << 8) + (little as u16)
    }

    fn is_pressed(&self, key: u8) -> bool {
        self.device_state.get_keys().contains(&self.keymap[&key])
    }

    fn load_fonts(memory: &mut [u8; 4096]) {

        let font_map: [[u8; 5]; 16] = [
            [0xF0, 0x90, 0x90, 0x90, 0xF0],     // 0
            [0x20, 0x60, 0x20, 0x20, 0x70],     // 1
            [0xF0, 0x10, 0xF0, 0x80, 0xF0],     // 2
            [0xF0, 0x10, 0xF0, 0x10, 0xF0],     // 3
            [0x90, 0x90, 0xF0, 0x10, 0x10],     // 4
            [0xF0, 0x80, 0xF0, 0x10, 0xF0],     // 5
            [0xF0, 0x80, 0xF0, 0x90, 0xF0],     // 6
            [0xF0, 0x10, 0x20, 0x40, 0x40],     // 7
            [0xF0, 0x90, 0xF0, 0x90, 0xF0],     // 8
            [0xF0, 0x90, 0xF0, 0x10, 0x90],     // 9
            [0xF0, 0x90, 0xF0, 0x90, 0x90],     // A
            [0xE0, 0x90, 0xE0, 0x90, 0xE0],     // B
            [0xF0, 0x80, 0x80, 0x80, 0xF0],     // C
            [0xE0, 0x90, 0x90, 0x90, 0xE0],     // D
            [0xF0, 0x80, 0xF0, 0x80, 0xF0],     // E
            [0xF0, 0x80, 0xF0, 0x80, 0x80]      // F
        ];

        let mut i = 0;

        for letter in font_map.iter() {
            for value in letter {
                memory[i] = *value;
                i += 1;
            }
        }

        assert!(i < 0x0200)
    }

    #[allow(unused_must_use)]
    fn draw_screen(&mut self) {

        self.display.flush();

        for row in self.display_ram.iter() {
            for column in row.iter() {
                if *column {
                    print!("█");
                } else {
                    print!(" ");
                }
            } println!();
        }
    }

    fn clear(&mut self) {
        print!("{}[2J", 27 as char);
        for y in  0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                self.display_ram[y][x] = false;
            }
        }
    }

    fn skip(&mut self) {
        self.pc = self.pc.wrapping_add(2);
    }
}