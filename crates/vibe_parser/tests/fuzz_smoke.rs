// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use vibe_parser::parse_source;

#[test]
fn lexer_parser_fuzz_smoke_does_not_panic() {
    let mut seed: u64 = 0xDEADBEEFCAFEBABE;
    for _ in 0..400 {
        let mut s = String::new();
        let len = (next_u32(&mut seed) % 220) as usize;
        for _ in 0..len {
            let c = match next_u32(&mut seed) % 16 {
                0 => '{',
                1 => '}',
                2 => '(',
                3 => ')',
                4 => '[',
                5 => ']',
                6 => '@',
                7 => ':',
                8 => '=',
                9 => '.',
                10 => ',',
                11 => '\n',
                12 => '"',
                13 => (b'a' + (next_u32(&mut seed) % 26) as u8) as char,
                14 => (b'0' + (next_u32(&mut seed) % 10) as u8) as char,
                _ => ' ',
            };
            s.push(c);
        }
        let _ = parse_source(&s);
    }
}

fn next_u32(seed: &mut u64) -> u32 {
    let mut x = *seed;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *seed = x;
    (x >> 16) as u32
}
