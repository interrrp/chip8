use std::{thread::sleep, time::Duration};

use raylib::{ffi::KeyboardKey, RaylibHandle};

pub fn is_key_pressed(rl: &RaylibHandle, code: u8) -> bool {
    rl.is_key_down(code_to_key(code))
}

pub fn wait_for_key(rl: &RaylibHandle) -> u8 {
    loop {
        for code in 0x1..=0xF {
            if is_key_pressed(rl, code) {
                return code;
            }
        }
        sleep(Duration::from_millis(16));
    }
}

fn code_to_key(code: u8) -> KeyboardKey {
    match code {
        0x1 => KeyboardKey::KEY_ONE,
        0x2 => KeyboardKey::KEY_TWO,
        0x3 => KeyboardKey::KEY_THREE,
        0x4 => KeyboardKey::KEY_FOUR,
        0x5 => KeyboardKey::KEY_FIVE,
        0x6 => KeyboardKey::KEY_SIX,
        0x7 => KeyboardKey::KEY_SEVEN,
        0x8 => KeyboardKey::KEY_EIGHT,
        0x9 => KeyboardKey::KEY_NINE,
        0xA => KeyboardKey::KEY_A,
        0xB => KeyboardKey::KEY_B,
        0xC => KeyboardKey::KEY_C,
        0xD => KeyboardKey::KEY_D,
        0xE => KeyboardKey::KEY_E,
        0xF => KeyboardKey::KEY_F,
        _ => KeyboardKey::KEY_ZERO,
    }
}
