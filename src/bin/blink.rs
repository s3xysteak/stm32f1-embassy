#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let led = Output::new(p.PC13, Level::Low, Speed::Low);

    spawner.spawn(run(led)).unwrap();

    loop {
        Timer::after_micros(1).await;
    }
}

#[embassy_executor::task]
async fn run(mut led: Output<'static>) {
    loop {
        Timer::after_millis(500).await;
        led.set_high();

        Timer::after_millis(500).await;
        led.set_low();
    }
}
