# u2u-keyboard

Firmware for the Raspberry Pi Pico (RP2040) that bridges a UART line protocol
to a USB HID keyboard.  Text messages are received over UART0, routed according
to a destination field in the message header, and — when addressed to this node
— typed on the host computer as if entered on a physical keyboard.

## Use case

![Use Case](doc/usage_ai.avif)

This AI generated image gives you an idea about the use case.
Imagine you have some "headless" server or desktop PCs you want to control remotely.

You connect a Raspberry Pi Pico to each PC via USB, operating as a keyboard.
The Raspberry Pi Pico will be connected with each other over UART, in a "daisy-chain" manner.
That means, for each Raspberry Pi Pico, connect the UART0 TX output will be UART0 RX input of the next one.

You use a laptop to send the key-strokes to the Raspberry Pi Pico over UART.
Therefore you have to have a USB to UART cable (FTDI 3.3V), and connect its TX output 
with the RX input of the very first Raspberry Pi Pico (in the daisy chain you created above).
Optional, you can connect the USB cables RX input, with the TX output of the very last Raspberry Pi Pico.

Now you can use the following UART protocol, to send key strokes to the respective PCs.


## Hardware

| Signal     | GPIO   | Notes                                        |
|------------|--------|----------------------------------------------|
| UART0 TX   | GPIO 0 | transmit to upstream/downstream node         |
| UART0 RX   | GPIO 1 | receive from upstream/downstream node        |
| Status LED | GPIO 25 | on-board LED of the Pico                    |
| USB        | —      | USB full-speed, exposes HID keyboard to host |

UART settings: **115200 baud, 8N1**.

## Line format

Every message sent over UART must follow this format:

```
#<DST>:<TEXT>\n
```

| Field    | Description |
|----------|-------------|
| `#`      | Start-of-frame marker; also resets any partially received line |
| `<DST>`  | Decimal destination address (0 – 255) |
| `:`      | Separator |
| `<TEXT>` | Payload — printable characters and escape sequences (see below) |
| `\n`     | Line terminator (`\r` is also accepted) |

### DST routing rules

| DST value   | Action |
|-------------|--------|
| `0`         | Forward unchanged (pass-through; no LED, no local processing) |
| `1`         | **Deliver to this node** — triggers LED pulse and USB keystroke output |
| `2` – `254` | Forward with DST decremented by 1 (daisy-chain routing) |
| `255`       | Broadcast — forward unchanged **and** deliver to this node |
| others      | Message to other destinations are dropped |

Lines received before the first `#` are silently discarded.
Max line length is 256 bytes. Lines longer than 256 bytes are dropped!

### Examples

```
#1:Hello\n          →  typed on the local host
#2:ping\n           →  forwarded as  #1:ping\n  to the next node
#255:hello\n        →  typed locally and forwarded unchanged
#0:ignored\n        →  forwarded, no LED, no local processing
```

## Escape sequences

Inside `<TEXT>`, a backslash `\` introduces an escape sequence.  The character
immediately following the `\` selects the action; neither character is typed
literally.  An unrecognised or trailing `\` is silently dropped.

| Sequence       | Key / Action         |
|----------------|----------------------|
| `\\`           | Backslash            |
| `\H`           | Hash #               |
| `\n`           | Enter                |
| `\N`           | Shift + Enter        |
| `\r`           | Ctrl + Enter         |
| `\t`           | Tab                  |
| `\T`           | Shift + Tab          |
| `\e`           | Escape               |
| `\b`           | Backspace            |
| `\w`           | Windows key (left)   |
| `\W`           | Windows key (right)  |
| `\a`           | Alt (left)           |
| `\A`           | Alt (right)          |
| `\s`           | Shift (left)         |
| `\S`           | Shift (right)        |
| `\g`           | Ctrl (left)          |
| `\G`           | Ctrl (right)         |
| `\C`           | Caps Lock            |
| `\i`           | Cursor up            |
| `\k`           | Cursor down          |
| `\j`           | Cursor left          |
| `\l`           | Cursor right         |
| `\I`           | Insert               |
| `\D`           | Delete               |
| `\^`           | Home (Pos1)          |
| `\$`           | End                  |
| `\P`           | Page Up              |
| `\p`           | Page Down            |
| `\1` – `\9`    | F1 – F9              |
| `\0`           | F10                  |
| `\c`           | Ctrl + C             |
| `\d`           | Ctrl + D             |
| `\z`           | Ctrl + Z             |

**Example** — open the Run dialog on Windows and launch Notepad:

```
#1:\wnotepad\n
```


### More Examples

Send greetings to the PCs, connected at the 1st and 3rd Raspberry Pi Pico:

```
#1:echo 'Hey number one'
#3:echo 'Hello \H3'
```

Send greetings to everybody:

```
#255:echo 'Hi'
```


Send a message though the chain without processing it:

```
#0:This message should be seen by all\neventually also by the sender!
```


## Keyboard layout selection

HID keycodes represent physical key positions.  The firmware must be built for
the same layout that is active on the host OS so that the correct characters
are produced.

| Cargo feature   | Host layout             | Default |
|-----------------|-------------------------|---------|
| `layout-qwertz` | German / Austrian QWERTZ | yes    |
| `layout-qwerty` | US / UK QWERTY           | no     |

Exactly one layout feature may be active at a time; the compiler enforces this.

## German umlauts and special characters (QWERTZ only)

When built with `layout-qwertz`, the following characters are supported in
addition to the full printable ASCII range.  They are expected to arrive
**UTF-8 encoded** over UART; the firmware converts them to Latin-9 internally
before the HID keycode lookup.

| Character | Unicode | UTF-8 bytes | Key on QWERTZ  |
|-----------|---------|-------------|----------------|
| ä         | U+00E4  | `C3 A4`     | ä              |
| Ä         | U+00C4  | `C3 84`     | Shift + ä      |
| ö         | U+00F6  | `C3 B6`     | ö              |
| Ö         | U+00D6  | `C3 96`     | Shift + ö      |
| ü         | U+00FC  | `C3 BC`     | ü              |
| Ü         | U+00DC  | `C3 9C`     | Shift + ü      |
| ß         | U+00DF  | `C3 9F`     | ß              |
| §         | U+00A7  | `C2 A7`     | Shift + 3      |
| °         | U+00B0  | `C2 B0`     | Shift + ^      |
| €         | U+20AC  | `E2 82 AC`  | AltGr + E      |

These characters are not available in the `layout-qwerty` build.

## Building and flashing

Install prerequisites:

```bash
rustup target add thumbv6m-none-eabi
cargo install elf2uf2-rs
```

Build (QWERTZ default):

```bash
cargo build --release
```

Build for QWERTY:

```bash
cargo build --release --no-default-features --features layout-qwerty
```

Flash — put the Pico into UF2 bootloader mode (hold BOOTSEL while plugging
in USB), then run:

```bash
./upload.sh
```

The script builds the firmware, converts the ELF to UF2, and offers to copy
it to the mounted `RPI-RP2` volume.

## LED behaviour

| Event                                      | Pulse duration |
|--------------------------------------------|----------------|
| Character received over UART               | 20 ms          |
| Message addressed to this node (DST 1/255) | 100 ms         |
