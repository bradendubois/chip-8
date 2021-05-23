mod cpu;

use std::collections::HashMap;
use device_query::Keycode;

pub struct Chip {
    cpu: cpu::CPU,
    memory: [u8; 4096],
    keymap: HashMap<u8, Keycode>
}

impl Chip {

    pub fn new(data: Vec<u8>, keymap: HashMap<u8, Keycode>) -> Chip {
        let mut chip = Chip {
            cpu: cpu::CPU::new(),
            memory: [0; 4096],
            keymap
        };

        for (i, x) in data.iter().enumerate() {
            chip.memory[i + 200] = *x;
        }

        chip
    }

}