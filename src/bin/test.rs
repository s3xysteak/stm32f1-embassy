#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, OutputOpenDrain, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let sign = OutputOpenDrain::new(p.PA0, Level::Low, Speed::Low);

    spawner.spawn(run(sign)).unwrap();

    loop {
        Timer::after_micros(1).await;
    }
}

#[embassy_executor::task]
async fn run(sign: OutputOpenDrain<'static>) {
    let mut temp = OneWire::new(sign);
    temp.reset().await;

    loop {
        temp.write_byte(0xCC).await;
        temp.write_byte(0x44).await;
        Timer::after_millis(750).await;
        temp.reset().await;
        temp.write_byte(0xCC).await;
        temp.write_byte(0xBE).await;
        info!("{}", temp.read_byte().await);

        Timer::after_secs(1).await
    }
}

enum OneWireCheck {
    ResetSuccess,
    ResetFailed,
}

struct OneWire {
    pin: OutputOpenDrain<'static>,
}

impl OneWire {
    fn new(pin: OutputOpenDrain<'static>) -> Self {
        Self { pin }
    }
    async fn reset(&mut self) -> OneWireCheck {
        self.pin.set_low();
        Timer::after_micros(480).await;

        self.pin.set_high();
        Timer::after_micros(60).await;
        for _ in 0..60 {
            Timer::after_micros(1).await;
            if self.pin.is_low() {
            } else {
                return OneWireCheck::ResetFailed;
            }
        }
        OneWireCheck::ResetSuccess
    }
    async fn write_bit(&mut self, bit: u8) {
        self.pin.set_low();
        Timer::after_micros(1).await;

        if bit == 0 {
            self.pin.set_low();
            Timer::after_micros(60).await;
        }
        self.pin.set_high();
    }
    async fn write_byte(&mut self, data: u8) {
        for i in 0..8 {
            self.write_bit((data >> i) & 0x01).await;
        }
    }
    async fn read_bit(&mut self) -> bool {
        self.pin.set_low();
        Timer::after_micros(1).await;
        self.pin.set_high();

        Timer::after_micros(10).await;
        if self.pin.is_low() { true } else { false }
    }
    async fn read_byte(&mut self) -> u8 {
        let mut data = 0;
        for i in 0..8 {
            data |= if self.read_bit().await { 0x01 } else { 0 } << i;
        }
        data
    }
}
