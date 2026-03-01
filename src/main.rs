#![no_std]
#![no_main]

mod layout;
mod led;
mod usb;

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
// use embassy_time::Timer;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::UART0;
use embassy_rp::uart::{BufferedInterruptHandler, BufferedUart, Config};
use embedded_io_async::{Read, Write};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
});

static TX_BUF: StaticCell<[u8; 256]> = StaticCell::new();
static RX_BUF: StaticCell<[u8; 256]> = StaticCell::new();

/// Maximum length of a single received line (including '#', DST, ':', TEXT, '\n').
const MAX_LINE_LEN: usize = 256;

/// LED pulse duration (ms) for an addressed line (DST 1 or 255).
const LED_PULSE_ADDRESSED_MS: u32 = 200;
/// LED pulse duration (ms) for a single received character.
const LED_PULSE_CHAR_MS: u32 = 20;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    spawner
        .spawn(led::led_task(Output::new(p.PIN_25, Level::Low)))
        .unwrap();
    spawner.spawn(usb::usb_task(p.USB)).unwrap();

    let mut uart_config = Config::default();
    uart_config.baudrate = 115200;

    let uart = BufferedUart::new(
        p.UART0,
        p.PIN_0, // TX: GPIO0
        p.PIN_1, // RX: GPIO1
        Irqs,
        TX_BUF.init([0u8; 256]),
        RX_BUF.init([0u8; 256]),
        uart_config,
    );

    let (mut tx, mut rx) = uart.split();

    let mut line_buf = [0u8; MAX_LINE_LEN];
    let mut line_len: usize = 0;
    let mut in_line = false;

    info!("u2u-keyboard: UART line processor ready");

    loop {
        let mut byte = [0u8; 1];
        match rx.read(&mut byte).await {
            Err(_) => {
                warn!("UART RX error, resetting line state");
                line_len = 0;
                in_line = false;
                continue;
            }
            Ok(_) => {}
        }

        // Short blink on every received character.
        let _ = led::LED_CHANNEL.try_send(LED_PULSE_CHAR_MS);

        let b = byte[0];

        if b == b'#' {
            // Start of a new line; '#' mid-stream also resets any partial line.
            line_buf[0] = b'#';
            line_len = 1;
            in_line = true;
        } else if in_line {
            if line_len >= MAX_LINE_LEN {
                warn!("Line too long, discarding");
                line_len = 0;
                in_line = false;
                continue;
            }
            line_buf[line_len] = b;
            line_len += 1;

            if b == b'\n' || b == b'\r' {
                handle_line(&line_buf[..line_len], &mut tx).await;
                line_len = 0;
                in_line = false;
            }
        }
        // Bytes received before the first '#' are silently discarded.
    }
}

/// Parse and dispatch a complete line of the form `#<DST>:<TEXT>\n`.
///
/// Routing rules:
/// - DST == 0   → drop (ignore)
/// - DST == 1   → deliver to `on_local_text()` for local processing
/// - DST 2..254 → decrement DST by 1 and retransmit on UART0
/// - DST == 255 → retransmit on UART0 unchanged (broadcast / all-nodes)
/// - DST > 255  → drop (out of range)
async fn handle_line(line: &[u8], tx: &mut impl Write) {
    // Minimum valid line: "#0:\n" → 4 bytes
    if line.len() < 4 || line[0] != b'#' {
        return;
    }

    // Locate the ':' that separates the DST field from the TEXT field.
    let colon = match line[1..].iter().position(|&b| b == b':') {
        Some(i) => 1 + i,
        None => {
            warn!("Malformed line: missing ':'");
            return;
        }
    };

    let dst = match parse_u32(&line[1..colon]) {
        Some(v) => v,
        None => {
            warn!("Malformed line: invalid DST field");
            return;
        }
    };

    // TEXT includes the trailing '\n'.
    let text = &line[colon + 1..];

    match dst {
        0 => {
            // Forward with DST 0, to relay a message ignored by all nodes
            // Used to test message forwarding without triggering application logic or LED pulses.
            transmit_line(tx, 0, text).await;
        }
        1 => {
            // Addressed to this node: longer LED pulse + application logic.
            let _ = led::LED_CHANNEL.try_send(LED_PULSE_ADDRESSED_MS);
            on_local_text(text).await;
        }
        2..=254 => {
            // Forward with DST decremented by 1.
            transmit_line(tx, dst - 1, text).await;
        }
        255 => {
            // Broadcast / all-nodes: longer LED pulse + forward unchanged.
            let _ = led::LED_CHANNEL.try_send(LED_PULSE_ADDRESSED_MS);
            transmit_line(tx, 255, text).await;
            on_local_text(text).await;
        }
        _ => {
            // DST > 255: out of protocol range, drop.
            warn!("Dropping line: DST={} out of range", dst);
        }
    }
}

/// Handle text addressed to this node (DST == 1).
/// Logs the content and sends each supported character as a USB HID keystroke.
async fn on_local_text(text: &[u8]) {
    // Strip the trailing line ending for display and key-sending.
    let text = match text {
        [rest @ .., b'\n'] => rest,
        [rest @ .., b'\r'] => rest,
        other => other,
    };

    info!("Local text received ({} bytes): {:a}", text.len(), text);

    /* Development only!
    // Give the user time to focus the target input field before typing starts.
    Timer::after_secs(5).await;
    */

    // Translate UTF-8 multi-byte sequences to Latin-9 single bytes so that
    // ascii_to_hid can process German characters (ä ö ü ß § ° €).
    #[cfg(feature = "layout-qwertz")]
    let mut latin9_buf = [0u8; MAX_LINE_LEN];
    #[cfg(feature = "layout-qwertz")]
    let text = utf8_to_latin9(text, &mut latin9_buf);

    let mut escape = false;
    let mut latched_modifier: u8 = 0;
    for &ch in text {
        if escape {
            escape = false;
            match layout::ascii_to_hid(ch, true) {
                layout::KeyCode::Code(modifier, keycode) => {
                    usb::KBD_CHANNEL.send((latched_modifier | modifier, keycode)).await;
                    latched_modifier = 0;
                }
                layout::KeyCode::Modifier(modifier) => {
                    latched_modifier |= modifier;
                }
                layout::KeyCode::None => {}
            }
        } else if ch == b'\\' {
            escape = true;
        } else {
            match layout::ascii_to_hid(ch, false) {
                layout::KeyCode::Code(modifier, keycode) => {
                    usb::KBD_CHANNEL.send((latched_modifier | modifier, keycode)).await;
                    latched_modifier = 0;
                }
                layout::KeyCode::None | layout::KeyCode::Modifier(_) => {}
            }
        }
    }
}

/// Serialize and transmit `#<dst>:<text>` on UART0.
/// `text` must already include the trailing `'\n'`.
async fn transmit_line(tx: &mut impl Write, dst: u32, text: &[u8]) {
    // Header: '#' + up to 3 decimal digits + ':' → 5 bytes max for dst ≤ 255.
    let mut hdr = [0u8; 6];
    let mut hlen = 0;
    hdr[hlen] = b'#';
    hlen += 1;
    hlen += fmt_u32(&mut hdr[hlen..], dst);
    hdr[hlen] = b':';
    hlen += 1;

    if tx.write_all(&hdr[..hlen]).await.is_err() {
        warn!("UART TX error (header)");
        return;
    }
    if tx.write_all(text).await.is_err() {
        warn!("UART TX error (body)");
    }
}

// ---------------------------------------------------------------------------
// Utility: ASCII decimal parsing and formatting
// ---------------------------------------------------------------------------

/// Translate UTF-8 encoded text to Latin-9 (ISO-8859-15).
///
/// - ASCII bytes (< 0x80) pass through unchanged.
/// - 2-byte sequences with codepoints 0x80–0xFF are emitted as their single Latin-9 byte
///   (covers ä Ä ö Ö ü Ü ß § °).
/// - U+20AC (€, three-byte sequence 0xE2 0x82 0xAC) is emitted as Latin-9 0x80.
/// - All other multi-byte sequences are silently dropped.
///
/// Returns the translated sub-slice of `out` (always ≤ `input` length).
#[cfg(feature = "layout-qwertz")]
fn utf8_to_latin9<'a>(input: &[u8], out: &'a mut [u8]) -> &'a [u8] {
    let mut i = 0;
    let mut j = 0;
    while i < input.len() && j < out.len() {
        let b = input[i];
        if b < 0x80 {
            // Plain ASCII — pass through.
            out[j] = b;
            j += 1;
            i += 1;
        } else if b & 0xE0 == 0xC0 && i + 1 < input.len() {
            // 2-byte sequence: codepoints U+0080–U+07FF.
            let b2 = input[i + 1];
            if b2 & 0xC0 == 0x80 {
                let cp = (((b & 0x1F) as u16) << 6) | ((b2 & 0x3F) as u16);
                if cp >= 0x80 {
                    // Codepoints 0x80–0xFF map 1-to-1 to Latin-9 bytes.
                    out[j] = cp as u8;
                    j += 1;
                }
            }
            i += 2;
        } else if b & 0xF0 == 0xE0 && i + 2 < input.len() {
            // 3-byte sequence: codepoints U+0800–U+FFFF.
            let b2 = input[i + 1];
            let b3 = input[i + 2];
            if b2 & 0xC0 == 0x80 && b3 & 0xC0 == 0x80 {
                let cp = (((b & 0x0F) as u32) << 12)
                    | (((b2 & 0x3F) as u32) << 6)
                    | ((b3 & 0x3F) as u32);
                if cp == 0x20AC {
                    out[j] = 0x80; // € → Latin-9 0x80
                    j += 1;
                }
            }
            i += 3;
        } else {
            // 4-byte sequence, stray continuation byte, or truncated sequence: skip.
            i += 1;
        }
    }
    &out[..j]
}

/// Parse a non-empty ASCII decimal byte slice into a `u32`.
/// Returns `None` if the slice is empty, contains non-digit bytes, or overflows.
fn parse_u32(s: &[u8]) -> Option<u32> {
    if s.is_empty() {
        return None;
    }
    let mut val: u32 = 0;
    for &b in s {
        if !b.is_ascii_digit() {
            return None;
        }
        val = val.checked_mul(10)?.checked_add((b - b'0') as u32)?;
    }
    Some(val)
}

/// Write `val` as ASCII decimal into `buf`. Returns the number of bytes written.
fn fmt_u32(buf: &mut [u8], mut val: u32) -> usize {
    if val == 0 {
        if buf.is_empty() {
            return 0;
        }
        buf[0] = b'0';
        return 1;
    }
    // Build digits in reverse order.
    let mut tmp = [0u8; 10];
    let mut n = 0;
    while val > 0 {
        tmp[n] = b'0' + (val % 10) as u8;
        val /= 10;
        n += 1;
    }
    let out = n.min(buf.len());
    for i in 0..out {
        buf[i] = tmp[n - 1 - i];
    }
    out
}
