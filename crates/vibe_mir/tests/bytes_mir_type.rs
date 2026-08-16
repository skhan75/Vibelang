// Copyright 2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! `Bytes` must survive the type-name round trip through MIR. A name that does
//! not map falls to `MirType::Unknown`, which lowers to `I64` rather than a
//! pointer, and a native taking `Bytes` would then be called with the wrong ABI
//! and read a pointer as an integer.

use vibe_mir::{mir_type_name, parse_type_name, MirType};

#[test]
fn bytes_maps_to_its_own_mir_type() {
    assert_eq!(parse_type_name("Bytes"), MirType::Bytes);
    assert_eq!(mir_type_name(&MirType::Bytes), "Bytes");
}

#[test]
fn bytes_does_not_fall_through_to_unknown() {
    assert_ne!(parse_type_name("Bytes"), MirType::Unknown);
}
