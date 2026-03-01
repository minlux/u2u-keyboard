//! German/Austrian QWERTZ keyboard layout.
//!
//! Covers ASCII 0x20–0x7E plus the Latin-9 (ISO-8859-15) German characters
//! listed in latin-9_map.txt.  Input bytes are expected to be Latin-9 encoded.
//!
//! HID keycodes are physical key positions (QWERTY reference).  To produce a
//! given character on a host with QWERTZ active we must send the keycode of
//! the physical key that QWERTZ maps to that character.
//!
//! Notable differences from QWERTY:
//!   y ↔ z swapped          (letters table)
//!   many symbols shifted    e.g. ( → Shift+8, ) → Shift+9
//!   AltGr needed for        @ [ ] { } \ | ~
//!   dead keys               ^ (0x35) and ` (Shift+0x2e) may require a
//!                           follow-up space on some hosts to produce a
//!                           standalone character.

use super::KeyCode;

const N:  u8 = 0x00; // no modifier
const C:  u8 = 0x01; // Left Ctrl
const S:  u8 = 0x02; // Left Shift
const A:  u8 = 0x04; // Left Alt
const W:  u8 = 0x08; // Left GUI  (Win left)
// const RC: u8 = 0x10; // Right Ctrl
// const RS: u8 = 0x20; // Right Shift
const G:  u8 = 0x40; // Right Alt (AltGr)
// const RW: u8 = 0x80; // Right GUI (Win right)

/// Keycodes for 'a'–'z', index 0 = 'a'.
/// y (index 24) and z (index 25) are swapped vs QWERTY.
#[rustfmt::skip]
const LETTERS: [u8; 26] = [
    0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
    0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
    0x18, 0x19, 0x1a, 0x1b, 0x1d, 0x1c, // y=0x1d  z=0x1c  (swapped)
];

/// Convert a Latin-9 byte to a [`KeyCode`].
/// Covers printable ASCII (0x20–0x7E) plus German-specific Latin-9 characters.
/// Returns [`KeyCode::None`] for bytes outside the supported set.
pub fn ascii_to_hid(ch: u8, esc: bool) -> KeyCode {
    if esc {
        return esc_sequence(ch);
    }
    match ch {
        b' '        => KeyCode::Code(N, 0x2c), // Space bar
        b'!'        => KeyCode::Code(S, 0x1e), // Shift+1
        b'"'        => KeyCode::Code(S, 0x1f), // Shift+2        (QWERTY: Shift+')
        b'#'        => KeyCode::Code(N, 0x32), // # key          (ISO key right of Ä)
        b'$'        => KeyCode::Code(S, 0x21), // Shift+4
        b'%'        => KeyCode::Code(S, 0x22), // Shift+5
        b'&'        => KeyCode::Code(S, 0x23), // Shift+6        (QWERTY: Shift+7)
        b'\''       => KeyCode::Code(S, 0x32), // Shift+#        (QWERTY: plain ' key)
        b'('        => KeyCode::Code(S, 0x25), // Shift+8        (QWERTY: Shift+9)
        b')'        => KeyCode::Code(S, 0x26), // Shift+9        (QWERTY: Shift+0)
        b'*'        => KeyCode::Code(S, 0x30), // Shift++ key    (QWERTY: Shift+8)
        b'+'        => KeyCode::Code(N, 0x30), // + key          (QWERTY: Shift+=)
        b','        => KeyCode::Code(N, 0x36), // ,
        b'-'        => KeyCode::Code(N, 0x38), // - key          (QWERTY: plain - key is at 0x2d)
        b'.'        => KeyCode::Code(N, 0x37), // .
        b'/'        => KeyCode::Code(S, 0x24), // Shift+7        (QWERTY: plain / key)
        b'0'        => KeyCode::Code(N, 0x27),
        b'1'..=b'9' => KeyCode::Code(N, 0x1e + (ch - b'1')),
        b':'        => KeyCode::Code(S, 0x37), // Shift+.        (QWERTY: Shift+;)
        b';'        => KeyCode::Code(S, 0x36), // Shift+,        (QWERTY: plain ; key)
        b'<'        => KeyCode::Code(N, 0x64), // ISO extra key  (between left-Shift and Y)
        b'='        => KeyCode::Code(S, 0x27), // Shift+0        (QWERTY: plain = key)
        b'>'        => KeyCode::Code(S, 0x64), // Shift+ISO key
        b'?'        => KeyCode::Code(S, 0x2d), // Shift+ß        (QWERTY: Shift+/)
        b'@'        => KeyCode::Code(G, 0x14), // AltGr+q        (QWERTY: Shift+2)
        b'A'..=b'Z' => KeyCode::Code(S, LETTERS[(ch - b'A') as usize]),
        b'['        => KeyCode::Code(G, 0x25), // AltGr+8        (QWERTY: plain [)
        b'\\'       => KeyCode::Code(G, 0x2d), // AltGr+ß        (QWERTY: plain \)
        b']'        => KeyCode::Code(G, 0x26), // AltGr+9        (QWERTY: plain ])
        b'^'        => KeyCode::Code(N, 0x35), // ^ key  ⚠ dead key on most QWERTZ hosts
        b'_'        => KeyCode::Code(S, 0x38), // Shift+-        (QWERTY: Shift+-)
        b'`'        => KeyCode::Code(S, 0x2e), // Shift+´ key  ⚠ dead key on most QWERTZ hosts
        b'a'..=b'z' => KeyCode::Code(N, LETTERS[(ch - b'a') as usize]),
        b'{'        => KeyCode::Code(G, 0x24), // AltGr+7        (QWERTY: Shift+[)
        b'|'        => KeyCode::Code(G, 0x64), // AltGr+ISO key  (QWERTY: Shift+\)
        b'}'        => KeyCode::Code(G, 0x27), // AltGr+0        (QWERTY: Shift+])
        b'~'        => KeyCode::Code(G, 0x30), // AltGr++ key    (QWERTY: Shift+`)

        // ---------------------------------------------------------------
        // Latin-9 (ISO-8859-15) German characters  (source: latin-9_map.txt)
        // Physical key positions on a QWERTZ keyboard:
        //   0x2f = ü/Ü key   0x33 = ö/Ö key   0x34 = ä/Ä key
        //   0x2d = ß key     0x20 = 3 key      0x35 = ^ key
        // ---------------------------------------------------------------
        0xe4        => KeyCode::Code(N, 0x34), // ä
        0xc4        => KeyCode::Code(S, 0x34), // Ä
        0xf6        => KeyCode::Code(N, 0x33), // ö
        0xd6        => KeyCode::Code(S, 0x33), // Ö
        0xfc        => KeyCode::Code(N, 0x2f), // ü
        0xdc        => KeyCode::Code(S, 0x2f), // Ü
        0xdf        => KeyCode::Code(N, 0x2d), // ß
        0xa7        => KeyCode::Code(S, 0x20), // § (Shift+3)
        0xb0        => KeyCode::Code(S, 0x35), // ° (Shift+^)
        0x80        => KeyCode::Code(G, 0x08), // € (AltGr+e)

        _           => KeyCode::None,
    }
}

/// Map an escape-sequence character (the byte after `\`) to a [`KeyCode`].
fn esc_sequence(ch: u8) -> KeyCode {
    match ch {
        b'\\' => KeyCode::Code(G,  0x2d), // \\  AltGr+ß (QWERTY: plain \)
        b'H'  => KeyCode::Code(N,  0x32), // \H  # key (ISO key right of Ä)
        b'n'  => KeyCode::Code(N,  0x28), // \n  Enter
        b't'  => KeyCode::Code(N,  0x2b), // \t  Tab
        b'e'  => KeyCode::Code(N,  0x29), // \e  Escape
        b'b'  => KeyCode::Code(N,  0x2a), // \b  Backspace
        b'B'  => KeyCode::Code(N,  0x48), // \B  Pause/Break
        b'w'  => KeyCode::Code(W,  0x00), // \w  Win left
        b'W'  => KeyCode::Modifier(W),    // \W  Win left modifier
        b'a'  => KeyCode::Code(A, 0x00),  // \a  Alt left
        b'A'  => KeyCode::Modifier(A),    // \A  Alt left modifier
        b'g'  => KeyCode::Code(G, 0x00),  // \g  AltGr
        b'G'  => KeyCode::Modifier(G),    // \G  AltGr modifier
        b's'  => KeyCode::Code(S, 0x00),  // \s  Shift left
        b'S'  => KeyCode::Modifier(S),    // \S  Shift left modifier
        b'c'  => KeyCode::Code(C, 0x00),  // \c  Ctrl left
        b'C'  => KeyCode::Modifier(C),    // \C  Ctrl left modifier
        b'U'  => KeyCode::Code(N,  0x39), // \U  Caps Lock
        b'i'  => KeyCode::Code(N,  0x52), // \i  Cursor up
        b'j'  => KeyCode::Code(N,  0x50), // \j  Cursor left
        b'k'  => KeyCode::Code(N,  0x51), // \k  Cursor down
        b'l'  => KeyCode::Code(N,  0x4f), // \l  Cursor right
        b'I'  => KeyCode::Code(N,  0x49), // \I  Insert
        b'D'  => KeyCode::Code(N,  0x4c), // \D  Delete
        b'^'  => KeyCode::Code(N,  0x4a), // \^  Home (Pos1)
        b'$'  => KeyCode::Code(N,  0x4d), // \$  End
        b'P'  => KeyCode::Code(N,  0x4b), // \P  Page Up
        b'p'  => KeyCode::Code(N,  0x4e), // \p  Page Down
        b'1'  => KeyCode::Code(N,  0x3a), // \1  F1
        b'2'  => KeyCode::Code(N,  0x3b), // \2  F2
        b'3'  => KeyCode::Code(N,  0x3c), // \3  F3
        b'4'  => KeyCode::Code(N,  0x3d), // \4  F4
        b'5'  => KeyCode::Code(N,  0x3e), // \5  F5
        b'6'  => KeyCode::Code(N,  0x3f), // \6  F6
        b'7'  => KeyCode::Code(N,  0x40), // \7  F7
        b'8'  => KeyCode::Code(N,  0x41), // \8  F8
        b'9'  => KeyCode::Code(N,  0x42), // \9  F9
        b'0'  => KeyCode::Code(N,  0x43), // \0  F10
        b'-'  => KeyCode::Code(N,  0x44), // \-  F11
        b'+'  => KeyCode::Code(N,  0x45), // \+  F12
        _     => KeyCode::None,
    }
}
