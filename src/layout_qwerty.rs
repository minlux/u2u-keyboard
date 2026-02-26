//! US/UK QWERTY keyboard layout — full ASCII 0x20–0x7E mapping.
//!
//! HID keycodes are physical key positions on the QWERTY reference layout,
//! so on a host with QWERTY active the keycode equals the character directly.

const N:  u8 = 0x00; // no modifier
const C:  u8 = 0x01; // Left Ctrl
const S:  u8 = 0x02; // Left Shift
const A:  u8 = 0x04; // Left Alt
const W:  u8 = 0x08; // Left GUI  (Win left)
const RC: u8 = 0x10; // Right Ctrl
const RS: u8 = 0x20; // Right Shift
const G:  u8 = 0x40; // Right Alt (AltGr)
const RW: u8 = 0x80; // Right GUI (Win right)

/// Keycodes for 'a'–'z', index 0 = 'a'.  Straight alphabetical order.
#[rustfmt::skip]
const LETTERS: [u8; 26] = [
    0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
    0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
    0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, // y=0x1c  z=0x1d
];

/// Convert a byte to a `(modifier, keycode)` pair.
///
/// When `esc` is `false` the byte is treated as a printable ASCII character
/// (0x20–0x7E).  When `esc` is `true` the byte is the character that followed
/// a `\` and is looked up in the escape-sequence table instead.
pub fn ascii_to_hid(ch: u8, esc: bool) -> Option<(u8, u8)> {
    if esc {
        return esc_sequence(ch);
    }
    Some(match ch {
        b' '        => (N, 0x2c), // Space bar
        b'!'        => (S, 0x1e), // Shift+1
        b'"'        => (S, 0x34), // Shift+'
        b'#'        => (S, 0x20), // Shift+3
        b'$'        => (S, 0x21), // Shift+4
        b'%'        => (S, 0x22), // Shift+5
        b'&'        => (S, 0x24), // Shift+7
        b'\''       => (N, 0x34), // '
        b'('        => (S, 0x26), // Shift+9
        b')'        => (S, 0x27), // Shift+0
        b'*'        => (S, 0x25), // Shift+8
        b'+'        => (S, 0x2e), // Shift+=
        b','        => (N, 0x36), // ,
        b'-'        => (N, 0x2d), // -
        b'.'        => (N, 0x37), // .
        b'/'        => (N, 0x38), // /
        b'0'        => (N, 0x27),
        b'1'..=b'9' => (N, 0x1e + (ch - b'1')),
        b':'        => (S, 0x33), // Shift+;
        b';'        => (N, 0x33), // ;
        b'<'        => (S, 0x36), // Shift+,
        b'='        => (N, 0x2e), // =
        b'>'        => (S, 0x37), // Shift+.
        b'?'        => (S, 0x38), // Shift+/
        b'@'        => (S, 0x1f), // Shift+2
        b'A'..=b'Z' => (S, LETTERS[(ch - b'A') as usize]),
        b'['        => (N, 0x2f), // [
        b'\\'       => (N, 0x31), // \
        b']'        => (N, 0x30), // ]
        b'^'        => (S, 0x23), // Shift+6
        b'_'        => (S, 0x2d), // Shift+-
        b'`'        => (N, 0x35), // `
        b'a'..=b'z' => (N, LETTERS[(ch - b'a') as usize]),
        b'{'        => (S, 0x2f), // Shift+[
        b'|'        => (S, 0x31), // Shift+\
        b'}'        => (S, 0x30), // Shift+]
        b'~'        => (S, 0x35), // Shift+`
        _           => return None,
    })
}

/// Map an escape-sequence character (the byte after `\`) to a `(modifier, keycode)` pair.
fn esc_sequence(ch: u8) -> Option<(u8, u8)> {
    Some(match ch {
        b'\\' => (N, 0x31), // \\
        b'H'  => (S, 0x20), // \H  # key (Shift+3)
        b'n'  => (N, 0x28), // \n  Enter
        b'N'  => (S, 0x28), // \N  Shift+Enter
        b'r'  => (C, 0x28), // \r  Ctrl+Enter
        b't'  => (N, 0x2b), // \t  Tab
        b'T'  => (S, 0x2b), // \T  Shift+Tab
        b'e'  => (N, 0x29), // \e  Escape
        b'b'  => (N, 0x2a), // \b  Backspace
        b'w'  => ( W, 0x00), // \w  Win left
        b'W'  => (RW, 0x00), // \W  Win right
        b'a'  => ( A, 0x00), // \a  Alt left
        b'A'  => ( G, 0x00), // \A  Alt right
        b's'  => ( S, 0x00), // \s  Shift left
        b'S'  => (RS, 0x00), // \S  Shift right
        b'g'  => ( C, 0x00), // \g  Ctrl left
        b'G'  => (RC, 0x00), // \G  Ctrl right
        b'C'  => (N, 0x39), // \C  Caps Lock
        b'i'  => (N, 0x52), // \i  Cursor up
        b'j'  => (N, 0x50), // \j  Cursor left
        b'k'  => (N, 0x51), // \k  Cursor down
        b'l'  => (N, 0x4f), // \l  Cursor right
        b'I'  => (N, 0x49), // \I  Insert
        b'D'  => (N, 0x4c), // \D  Delete
        b'^'  => (N, 0x4a), // \^  Home (Pos1)
        b'$'  => (N, 0x4d), // \$  End
        b'P'  => (N, 0x4b), // \P  Page Up
        b'p'  => (N, 0x4e), // \p  Page Down
        b'1'  => (N, 0x3a), // \1  F1
        b'2'  => (N, 0x3b), // \2  F2
        b'3'  => (N, 0x3c), // \3  F3
        b'4'  => (N, 0x3d), // \4  F4
        b'5'  => (N, 0x3e), // \5  F5
        b'6'  => (N, 0x3f), // \6  F6
        b'7'  => (N, 0x40), // \7  F7
        b'8'  => (N, 0x41), // \8  F8
        b'9'  => (N, 0x42), // \9  F9
        b'0'  => (N, 0x43), // \0  F10
        b'c'  => (C, 0x06), // \c  Ctrl+C  (keycode for 'c')
        b'd'  => (C, 0x07), // \d  Ctrl+D  (keycode for 'd')
        b'z'  => (C, 0x1d), // \z  Ctrl+Z  (QWERTY: z=0x1d)
        _     => return None,
    })
}
