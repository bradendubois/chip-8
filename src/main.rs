mod chip;

use std::fs;
use std::env;

use device_query::Keycode;
use std::process::exit;

fn main() {

    let mut keymap = std::collections::HashMap::new();

    // Row 1
    keymap.insert(0x1 as u8, Keycode::Key1);
    keymap.insert(0x2 as u8, Keycode::Key2);
    keymap.insert(0x3 as u8, Keycode::Key3);
    keymap.insert(0xC as u8, Keycode::Key4);

    // Row 2
    keymap.insert(0x4 as u8, Keycode::Q);
    keymap.insert(0x5 as u8, Keycode::W);
    keymap.insert(0x6 as u8, Keycode::E);
    keymap.insert(0xD as u8, Keycode::R);

    // Row 3
    keymap.insert(0x7 as u8, Keycode::A);
    keymap.insert(0x8 as u8, Keycode::S);
    keymap.insert(0x9 as u8, Keycode::D);
    keymap.insert(0xE as u8, Keycode::F);

    // Row 4
    keymap.insert(0xA as u8, Keycode::Z);
    keymap.insert(0x0 as u8, Keycode::X);
    keymap.insert(0xB as u8, Keycode::C);
    keymap.insert(0xF as u8, Keycode::V);

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("provide the path to a ROM as a command-line argument!");
        exit(0);
    }

    let file = &args[1];

    match fs::read(file) {
        Ok(bytes) => {
            let mut chip = chip::Chip::new(bytes, keymap);
            chip.run();
        },
        Err(i) => panic!("{}", i)
    }
}
