#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::OutputType,
    time::khz,
    timer::simple_pwm::{PwmPin, SimplePwm},
};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

mod utils;
use utils::*;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(define_config());

    let ch1_pin = PwmPin::new_ch1(p.PA0, OutputType::PushPull);
    let mut pwm = SimplePwm::new(
        p.TIM2,
        Some(ch1_pin),
        None,
        None,
        None,
        khz(10),
        Default::default(),
    );
    let mut ch1 = pwm.ch1();
    ch1.enable();

    let mut is_up = true;
    let mut current_duty: i8 = 0;
    loop {
        if is_up {
            current_duty += 1;
        } else {
            current_duty -= 1;
        }
        if current_duty < 0 {
            is_up = true;
        } else if current_duty > 50 {
            is_up = false;
        }

        ch1.set_duty_cycle_percent(current_duty.clamp(0, 100) as u8);

        Timer::after_millis(15).await;
    }
}
