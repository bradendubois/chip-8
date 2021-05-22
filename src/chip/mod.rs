mod cpu;

pub struct Chip {
    cpu: cpu::CPU,
    memory: [u8; 4096],
    stack: Vec<u16>
}

impl Chip {

    pub fn new(data: Vec<u8>) -> Chip {
        let mut chip = Chip {
            cpu: cpu::CPU::new(),
            memory: [0; 4096],
            stack: Vec::new()
        };

        for (i, x) in data.iter().enumerate() {
            chip.memory[i + 200] = *x;
        }

        chip
    }
}