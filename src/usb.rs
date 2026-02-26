use defmt::{info, warn};
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_rp::Peri;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Timer;
use embassy_usb::class::hid::{HidReaderWriter, ReportId, RequestHandler, State};
use embassy_usb::control::OutResponse;
use embassy_usb::{Builder, Config};
use static_cell::StaticCell;

bind_interrupts!(struct UsbIrqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

// ---------------------------------------------------------------------------
// HID descriptor — standard boot-protocol keyboard (8-byte input report)
// ---------------------------------------------------------------------------

const HID_KEYBOARD_DESCRIPTOR: &[u8] = &[
    0x05, 0x01,        // Usage Page (Generic Desktop)
    0x09, 0x06,        // Usage (Keyboard)
    0xa1, 0x01,        // Collection (Application)
    // --- Modifier keys (8 bits) ---
    0x05, 0x07,        //   Usage Page (Key Codes)
    0x19, 0xe0,        //   Usage Minimum (224 = Left Control)
    0x29, 0xe7,        //   Usage Maximum (231 = Right GUI)
    0x15, 0x00,        //   Logical Minimum (0)
    0x25, 0x01,        //   Logical Maximum (1)
    0x75, 0x01,        //   Report Size (1 bit)
    0x95, 0x08,        //   Report Count (8)
    0x81, 0x02,        //   Input (Data, Variable, Absolute)
    // --- Reserved byte ---
    0x95, 0x01,        //   Report Count (1)
    0x75, 0x08,        //   Report Size (8 bits)
    0x81, 0x01,        //   Input (Constant)
    // --- LED output report (5 LEDs + 3 pad bits) ---
    0x95, 0x05,        //   Report Count (5)
    0x75, 0x01,        //   Report Size (1 bit)
    0x05, 0x08,        //   Usage Page (LEDs)
    0x19, 0x01,        //   Usage Minimum (Num Lock)
    0x29, 0x05,        //   Usage Maximum (Kana)
    0x91, 0x02,        //   Output (Data, Variable, Absolute)
    0x95, 0x01,        //   Report Count (1)
    0x75, 0x03,        //   Report Size (3 bits padding)
    0x91, 0x01,        //   Output (Constant)
    // --- Key array (6 simultaneous keys) ---
    0x95, 0x06,        //   Report Count (6)
    0x75, 0x08,        //   Report Size (8 bits)
    0x15, 0x00,        //   Logical Minimum (0)
    0x25, 0x65,        //   Logical Maximum (101)
    0x05, 0x07,        //   Usage Page (Key Codes)
    0x19, 0x00,        //   Usage Minimum (0)
    0x29, 0x65,        //   Usage Maximum (101)
    0x81, 0x00,        //   Input (Data, Array)
    0xc0,              // End Collection
];

// ---------------------------------------------------------------------------
// Keystroke channel
// ---------------------------------------------------------------------------

/// Send `(modifier, keycode)` pairs here to have them typed via USB HID.
/// Capacity 256 — enough for a full text line without blocking the UART task.
pub static KBD_CHANNEL: Channel<CriticalSectionRawMutex, (u8, u8), 256> = Channel::new();

// ---------------------------------------------------------------------------
// USB HID task
// ---------------------------------------------------------------------------

#[embassy_executor::task]
pub async fn usb_task(usb: Peri<'static, USB>) {
    let driver = Driver::new(usb, UsbIrqs);

    let mut usb_config = Config::new(0x1209, 0x0001);
    usb_config.manufacturer = Some("u2u-keyboard");
    usb_config.product = Some("u2u HID Keyboard");
    usb_config.serial_number = Some("U2U0001");
    usb_config.max_power = 100;
    usb_config.max_packet_size_0 = 64;

    static CONFIG_DESC:     StaticCell<[u8; 256]>        = StaticCell::new();
    static BOS_DESC:        StaticCell<[u8; 256]>        = StaticCell::new();
    static MSOS_DESC:       StaticCell<[u8; 128]>        = StaticCell::new();
    static CONTROL_BUF:     StaticCell<[u8; 64]>         = StaticCell::new();
    static HID_STATE:       StaticCell<State>            = StaticCell::new();
    static REQUEST_HANDLER: StaticCell<KbdRequestHandler> = StaticCell::new();

    let mut builder = Builder::new(
        driver,
        usb_config,
        CONFIG_DESC.init([0u8; 256]),
        BOS_DESC.init([0u8; 256]),
        MSOS_DESC.init([0u8; 128]),
        CONTROL_BUF.init([0u8; 64]),
    );

    let hid_config = embassy_usb::class::hid::Config {
        report_descriptor: HID_KEYBOARD_DESCRIPTOR,
        request_handler: None,
        poll_ms: 10,
        max_packet_size: 8,
    };
    let hid = HidReaderWriter::<_, 1, 8>::new(
        &mut builder,
        HID_STATE.init(State::new()),
        hid_config,
    );
    let (hid_reader, mut hid_writer) = hid.split();

    let mut usb = builder.build();

    info!("u2u-keyboard: USB HID task started");

    // Keyboard sender: reads (modifier, keycode) from KBD_CHANNEL and types them.
    let keyboard_fut = async {
        loop {
            let (modifier, keycode) = KBD_CHANNEL.receive().await;

            // Key-down report
            let report = [modifier, 0x00, keycode, 0, 0, 0, 0, 0];
            if hid_writer.write(&report).await.is_err() {
                warn!("HID write error (key down)");
                continue;
            }

            // Hold long enough for the host to register the key press (~20 ms).
            Timer::after_millis(20).await;

            // Key-up report (all zeros)
            if hid_writer.write(&[0u8; 8]).await.is_err() {
                warn!("HID write error (key up)");
            }

            // Short gap between keystrokes so the host can distinguish repetitions.
            Timer::after_millis(10).await;
        }
    };

    join(
        usb.run(),
        join(hid_reader.run(false, REQUEST_HANDLER.init(KbdRequestHandler {})), keyboard_fut),
    )
    .await;
}

// ---------------------------------------------------------------------------
// HID request handler — responds to class requests from the host
// ---------------------------------------------------------------------------

struct KbdRequestHandler {}

impl RequestHandler for KbdRequestHandler {
    fn get_report(&mut self, id: ReportId, _buf: &mut [u8]) -> Option<usize> {
        info!("HID get_report id={:?}", id);
        None
    }

    fn set_report(&mut self, id: ReportId, data: &[u8]) -> OutResponse {
        // data[0] bits: 0=NumLock 1=CapsLock 2=ScrollLock 3=Compose 4=Kana
        info!("HID LED report id={:?} data={:x}", id, data);
        OutResponse::Accepted
    }

    fn set_idle_ms(&mut self, id: Option<ReportId>, dur: u32) {
        info!("HID set_idle id={:?} dur={}ms", id, dur);
    }

    fn get_idle_ms(&mut self, id: Option<ReportId>) -> Option<u32> {
        info!("HID get_idle id={:?}", id);
        None
    }
}
