//! US/UK QWERTY keyboard layout — full ASCII 0x20–0x7E mapping.
//!
//! HID keycodes are physical key positions on the QWERTY reference layout,
//! so on a host with QWERTY active the keycode equals the character directly.

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

/// Keycodes for 'a'–'z', index 0 = 'a'.  Straight alphabetical order.
#[rustfmt::skip]
const LETTERS: [u8; 26] = [
    0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
    0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
    0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, // y=0x1c  z=0x1d
];

/// Convert a byte to a [`KeyCode`].
///
/// When `esc` is `false` the byte is treated as a printable ASCII character
/// (0x20–0x7E).  When `esc` is `true` the byte is the character that followed
/// a `\` and is looked up in the escape-sequence table instead.
pub fn ascii_to_hid(ch: u8, esc: bool) -> KeyCode {
    if esc {
        return esc_sequence(ch);
    }
    match ch {
        b' '        => KeyCode::Code(N, 0x2c), // Space bar
        b'!'        => KeyCode::Code(S, 0x1e), // Shift+1
        b'"'        => KeyCode::Code(S, 0x34), // Shift+'
        b'#'        => KeyCode::Code(S, 0x20), // Shift+3
        b'$'        => KeyCode::Code(S, 0x21), // Shift+4
        b'%'        => KeyCode::Code(S, 0x22), // Shift+5
        b'&'        => KeyCode::Code(S, 0x24), // Shift+7
        b'\''       => KeyCode::Code(N, 0x34), // '
        b'('        => KeyCode::Code(S, 0x26), // Shift+9
        b')'        => KeyCode::Code(S, 0x27), // Shift+0
        b'*'        => KeyCode::Code(S, 0x25), // Shift+8
        b'+'        => KeyCode::Code(S, 0x2e), // Shift+=
        b','        => KeyCode::Code(N, 0x36), // ,
        b'-'        => KeyCode::Code(N, 0x2d), // -
        b'.'        => KeyCode::Code(N, 0x37), // .
        b'/'        => KeyCode::Code(N, 0x38), // /
        b'0'        => KeyCode::Code(N, 0x27),
        b'1'..=b'9' => KeyCode::Code(N, 0x1e + (ch - b'1')),
        b':'        => KeyCode::Code(S, 0x33), // Shift+;
        b';'        => KeyCode::Code(N, 0x33), // ;
        b'<'        => KeyCode::Code(S, 0x36), // Shift+,
        b'='        => KeyCode::Code(N, 0x2e), // =
        b'>'        => KeyCode::Code(S, 0x37), // Shift+.
        b'?'        => KeyCode::Code(S, 0x38), // Shift+/
        b'@'        => KeyCode::Code(S, 0x1f), // Shift+2
        b'A'..=b'Z' => KeyCode::Code(S, LETTERS[(ch - b'A') as usize]),
        b'['        => KeyCode::Code(N, 0x2f), // [
        b'\\'       => KeyCode::Code(N, 0x31), // \
        b']'        => KeyCode::Code(N, 0x30), // ]
        b'^'        => KeyCode::Code(S, 0x23), // Shift+6
        b'_'        => KeyCode::Code(S, 0x2d), // Shift+-
        b'`'        => KeyCode::Code(N, 0x35), // `
        b'a'..=b'z' => KeyCode::Code(N, LETTERS[(ch - b'a') as usize]),
        b'{'        => KeyCode::Code(S, 0x2f), // Shift+[
        b'|'        => KeyCode::Code(S, 0x31), // Shift+\
        b'}'        => KeyCode::Code(S, 0x30), // Shift+]
        b'~'        => KeyCode::Code(S, 0x35), // Shift+`
        _           => KeyCode::None,
    }
}

/// Map an escape-sequence character (the byte after `\`) to a [`KeyCode`].
fn esc_sequence(ch: u8) -> KeyCode {
    match ch {
        b'\\' => KeyCode::Code(N,  0x31), // \\
        b'H'  => KeyCode::Code(S,  0x20), // \H  # key (Shift+3)
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
