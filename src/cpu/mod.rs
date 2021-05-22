use rand::Rng;

pub struct CPU {
    v: [u8; 16],
    pc: u16,
    i: u16
}

const F: usize = 0x0F;

impl CPU {

    pub fn new() -> CPU {
        CPU {
            v: [0; 16],
            pc: 0x0100,
            i: 0x0000
        }
    }

    pub fn table(&mut self, instruction: u16) {


        let x = ((instruction & 0x0F00) >> 8) as u8 as usize;
        let y = ((instruction & 0x00F0) >> 4) as u8 as usize;


        let nnn = instruction & 0x0FFF;
        let nn = (instruction & 0x00FF) as u8;

        // Match the left-most digit for further matching
        match instruction & 0xF000 {

            0x0000 => {

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

            0x7000 => self.v[x] += nn,

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

            0xD000 => (), // TODO

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

            0xF000 => {},

            _ => panic!("unknown instruction: {}", instruction)
        };
    }

    /// JUMP - Jump to given instruction
    fn jump(&mut self, instruction: u16) {
        self.pc = instruction;
    }

    /// CALL - Call subroutine at given instruction
    fn call(&mut self, _instruction: u16) {
        // TODO
    }

    /// Get next instruction (and advance PC by 2 in doing so)
    fn next_instruction(&mut self) -> u16 {
        0 // TODO
    }

    fn is_pressed(&self, _key: u8) -> bool {
        false
    }

}