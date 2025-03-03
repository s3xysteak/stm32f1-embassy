#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, OutputOpenDrain, Speed};
use embassy_time::{Duration, Timer, block_for};
use {defmt_rtt as _, panic_probe as _};

mod utils;
use utils::*;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(define_config());

    let sign = OutputOpenDrain::new(p.PA0, Level::High, Speed::Low);

    spawner.spawn(run(sign)).unwrap();

    loop {
        Timer::after_micros(1).await;
    }
}

#[embassy_executor::task]
async fn run(sign: OutputOpenDrain<'static>) {
    let mut temp = OneWire::new(sign);

    loop {
        if let OneWireCheck::ResetSuccess = temp.reset().await {
            temp.write_byte(0xCC).await;
            temp.write_byte(0x44).await;
            Timer::after_millis(750).await;
            if let OneWireCheck::ResetSuccess = temp.reset().await {
                temp.write_byte(0xCC).await;
                temp.write_byte(0xBE).await;

                let lsb = temp.read_byte().await;
                let msb = temp.read_byte().await;
                info!("{0},{1}", lsb, msb);

                let temperature = transform_temp(lsb, msb);

                info!("Temperature: {}°C", temperature);
            };
        };

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
        delay_us(480);
        self.pin.set_high();
        delay_us(15);

        let mut is_ok = false;
        for _ in 0..45 {
            delay_us(1);
            if self.pin.is_low() {
                is_ok = true
            }
        }
        delay_us(420);
        if is_ok {
            OneWireCheck::ResetSuccess
        } else {
            OneWireCheck::ResetFailed
        }
    }
    async fn write_bit(&mut self, bit: u8) {
        self.pin.set_low();
        delay_us(1);

        if bit == 0 {
            self.pin.set_low();
            delay_us(59);
            self.pin.set_high();
        } else {
            self.pin.set_high();

            delay_us(59);
        }
    }
    async fn write_byte(&mut self, data: u8) {
        for i in 0..8 {
            self.write_bit((data >> i) & 0x01).await;
        }
    }
    async fn read_bit(&mut self) -> bool {
        self.pin.set_low();
        delay_us(1);
        self.pin.set_high();

        delay_us(10);
        let state = self.pin.is_high();
        delay_us(49);
        state
    }
    async fn read_byte(&mut self) -> u8 {
        let mut data = 0;
        for i in 0..8 {
            data |= if self.read_bit().await { 0x01 } else { 0 } << i;
        }
        data
    }
}

fn delay_us(us: u64) {
    block_for(Duration::from_micros(us));
}

fn transform_temp(lsb: u8, msb: u8) -> f32 {
    let raw_temp = ((msb as u16) << 8) | lsb as u16;
    if (raw_temp & 0x8000) != 0 {
        -((!raw_temp + 1) as i16) as f32 / 16.0
    } else {
        raw_temp as f32 / 16.0
    }
}
