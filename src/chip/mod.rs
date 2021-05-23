use std::cmp::min;
use std::collections::HashMap;

use device_query::{DeviceState, DeviceQuery, Keycode};
use rand::Rng;
use std::thread::sleep;
use std::time::Duration;

pub struct Chip {
    v: [u8; 16],
    pc: u16,
    i: u16,
    delay_timer: u8,
    sound_timer: u8,
    stack: Vec<u16>,
    memory: [u8; 4096],
    display: [[bool; 64]; 32],
    keymap: HashMap<u8, Keycode>,
    device_state: DeviceState
}

const F: usize = 0x0F;

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
            display: [[false; 64]; 32],
            keymap,
            device_state: DeviceState::new()
        }
    }

    pub fn run(&mut self) {

        println!("cpu beginning");

        self.draw_screen();

        loop {

            let instruction = self.next_instruction();
            // println!("fetched: {:#04X}", instruction);

            self.execute(instruction);
        }
    }

    pub fn execute(&mut self, instruction: u16) {

        let x = ((instruction & 0x0F00) >> 8) as u8 as usize;
        let y = ((instruction & 0x00F0) >> 4) as u8 as usize;

        let nnn = instruction & 0x0FFF;
        let nn = (instruction & 0x00FF) as u8;
        let n = (instruction & 0x000F) as u8;

        // Match the left-most digit for further matching
        match instruction & 0xF000 {

            0x0000 => {

                match nn {


                    0xEE => self.ret(),

                    _ => panic!("unmapped instruction: {}", instruction)

                }
            },

            0x1000 => self.jump(nnn),

            0x2000 => self.call(nnn),

            0x3000 => {
                if self.v[x] == nn {
                    self.next_instruction();    // fetch (and skip) next instruction
                }
            },

            0x4000 => {
                if self.v[x] != nn {
                    self.next_instruction();    // fetch (and skip) next instruction
                }
            },

            0x5000 => {
                if self.v[x] == self.v[y] {
                    self.next_instruction();    // fetch (and skip) next instruction
                }
            },

            0x6000 => self.v[x] = nn,

            0x7000 => self.v[x] = self.v[x].wrapping_add(nn),

            0x8000 => {

                match instruction & 0x000F {
                    0 => self.v[x] = self.v[y],
                    1 => self.v[x] |= self.v[y],
                    2 => self.v[x] &= self.v[y],
                    3 => self.v[x] ^= self.v[y],
                    4 => {
                        match self.v[x].checked_add(self.v[y]) {
                            Some(i) => {
                                self.v[x] = i;
                                self.v[F] = 00;
                            },
                            None => {
                                self.v[x] = self.v[x].wrapping_add(self.v[y]);
                                self.v[F] = 01;
                            }
                        }
                    },
                    5 => {
                        match self.v[x].checked_sub(self.v[y]) {
                            Some(i) => {
                                self.v[x] = i;
                                self.v[F] = 00;
                            },
                            None => {
                                self.v[x] = self.v[x].wrapping_sub(self.v[y]);
                                self.v[F] = 01;
                            }
                        }
                    },
                    6 => {
                        self.v[F] = self.v[y] & 0x01;
                        self.v[x] = self.v[y] >> 1;
                    }
                    7 => {
                        match self.v[y].checked_sub(self.v[x]) {
                            Some(i) => {
                                self.v[x] = i;
                                self.v[F] = 00;
                            },
                            None => {
                                self.v[x] = self.v[y].wrapping_sub(self.v[x]);
                                self.v[F] = 01;
                            }
                        }
                    },
                    8 => {
                        self.v[F] = self.v[y] & 0x80;
                        self.v[x] = self.v[y] << 1;
                    },
                    _ => panic!("unmapped instruction: {}", instruction)
                };
            },

            0x9000 => {
                if self.v[x] != self.v[y] {
                    self.next_instruction();
                }
            },

            0xA000 => self.i = nnn,

            0xB000 => self.jump(nnn + self.v[0] as u16),

            0xC000 => self.v[x] = rand::thread_rng().gen_range(0x00 ..= 0xFF) & nn,

            0xD000 => {

                let py = (self.v[x] % 32) as usize;
                let px = (self.v[y] % 64) as usize;

                let mut change = false;

                for row in 0..(min(py + ((n / 8) as usize), 32)) {

                    let sprite_row = self.memory[(self.i + (n as u16)) as usize];

                    for column in (px..min(px + 8, 64)).rev() {

                        if px + column >= 64 {
                            break
                        }

                        let on = (sprite_row & ((column as u8) << 1)) > 0;
                        let previous = self.display[row][px];
                        self.display[row][px] ^= on;
                        if self.display[row][px] != previous {
                            change = true;
                        }
                    }
                }

                match change {
                    true => self.v[F] = 01,
                    false => self.v[F] = 00
                };

                self.draw_screen();
            },

            0xE000 => {
                match nn {
                    0x9E => {
                        if self.is_pressed(self.v[x]) {
                            self.next_instruction();
                        }
                    },

                    0xA1 => {
                        if !self.is_pressed(self.v[x]) {
                            self.next_instruction();
                        }
                    },

                    _ => panic!("unmapped instruction: {}", instruction)
                }
            },

            0xF000 => {

                match nn {

                    0x07 => self.v[x] = self.delay_timer,
                    0x0A => {
                        loop {

                            let keys = self.device_state.get_keys();

                            for (code, keycode) in self.keymap.iter() {
                                if keys.contains(keycode) {
                                    self.v[x] = *code;
                                    break;
                                }
                            }

                            sleep(Duration::new(0, 1000000000));

                            if self.delay_timer > 0 {
                                self.delay_timer -= 1;
                            }
                        }
                    },

                    0x15 => self.delay_timer = self.v[x],
                    0x18 => self.sound_timer = self.v[x],
                    0x1E => self.i += self.v[x] as u16,
                    0x29 => self.i = self.v[x] as u16,
                    0x33 => {
                        self.memory[self.i as usize] = self.v[x] / 100;
                        self.memory[self.i as usize] = (self.v[x] % 100) / 10;
                        self.memory[self.i as usize] = self.v[x] % 10;

                    },
                    0x55 => {
                        for i in 0..x {
                            self.memory[self.i as usize] = self.v[i];
                            self.i += 1;
                        }
                    },

                    0x65 => {
                        for i in 0..x {
                            self.v[i] = self.memory[self.i as usize];
                            self.i += 1;
                        }
                    }

                    _ => panic!("unmapped instruction: {}", instruction)

                }
            },

            _ => panic!("unknown instruction: {}", instruction)
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
        self.pc = instruction
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

    fn draw_screen(&self) {

        print!("{}[2J", 27 as char);

        for row in self.display.iter() {
            for column in row.iter() {
                if *column {
                    print!("X");
                }
            } println!();
        }
    }
}