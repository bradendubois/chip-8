pub struct CPU {
    v_reg: [u8; 16]
}

impl CPU {

    pub fn new() -> CPU {
        CPU {
            v_reg: [0; 16]
        }
    }

    pub fn table(&mut self, instruction: u16) {

        // Match the left-most digit for further matching
        match instruction & 0xF000 {

            0x0000 => {

            },

            0x1000 => self.jump(instruction & 0x0FFF),

            0x2000 => self.call(instruction & 0x0FFF),

            0x3000 => {
                if self.v_reg[CPU::x(instruction)] == instruction as u8 {
                    self.next_instruction();    // fetch (and skip) next instruction
                }
            },

            0x4000 => {
                if self.v_reg[CPU::x(instruction)] != instruction as u8 {
                    self.next_instruction();    // fetch (and skip) next instruction
                }
            },

            0x5000 => {
                if self.v_reg[CPU::x(instruction)] == self.v_reg[instruction as u8 as usize] {
                    self.next_instruction();    // fetch (and skip) next instruction
                }
            },

            0x6000 => self.v_reg[CPU::x(instruction)] = instruction as u8,

            0x7000 => self.v_reg[CPU::x(instruction)] += instruction as u8,

            0x8000 => {

                let x = CPU::x(instruction);
                let y = CPU::y(instruction);

                match instruction & 0x000F {
                    0 => self.v_reg[x] = self.v_reg[y],
                    1 => self.v_reg[x] |= self.v_reg[y],
                    2 => self.v_reg[x] &= self.v_reg[y],
                    3 => self.v_reg[x] ^= self.v_reg[y],
                    4 => {

                    },
                    5 => {

                    },
                    6 => {

                    }
                    7 => {

                    },
                    8 => {

                    },
                    _ => panic!("unmapped instruction: {}", instruction)
                };
            },

            0x9000 => {

            },

            0xA000 => {},

            0xB000 => {},

            0xC000 => {},

            0xD000 => {},

            0xE000 => {},

            0xF000 => {},

            _ => panic!("unknown instruction: {}", instruction)
        };
    }

    fn x(instruction: u16) -> usize {
        ((instruction & 0x0F00) >> 8) as u8 as usize
    }

    fn y(instruction: u16) -> usize {
        ((instruction & 0x00F0) >> 4) as u8 as usize
    }

}