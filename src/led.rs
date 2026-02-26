use embassy_rp::gpio::Output;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Timer;

/// Send a pulse duration (in milliseconds) to this channel to trigger an LED blink.
/// The channel holds up to 4 pending pulses; excess requests are silently dropped.
pub static LED_CHANNEL: Channel<CriticalSectionRawMutex, u32, 4> = Channel::new();

/// LED task: waits for pulse durations on [`LED_CHANNEL`] and drives GPIO25 accordingly.
#[embassy_executor::task]
pub async fn led_task(mut led: Output<'static>) {
    loop {
        let ms = LED_CHANNEL.receive().await;
        led.set_high();
        Timer::after_millis(ms as u64).await;
        led.set_low();
    }
}
