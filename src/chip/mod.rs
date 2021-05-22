mod cpu;

pub struct Chip {
    cpu: cpu::CPU,
    memory: [u8; 4096]
}

impl Chip {

    pub fn new() -> Chip {
        Chip {
            cpu: cpu::CPU::new(),
            memory: [0; 4096]
        }
    }
}