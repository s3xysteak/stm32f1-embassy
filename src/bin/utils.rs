pub fn define_config() -> embassy_stm32::Config {
    use embassy_stm32::rcc::*;
    use embassy_stm32::time::Hertz;

    let mut config = embassy_stm32::Config::default();

    config.rcc.hsi = false;
    config.rcc.sys = Sysclk::PLL1_P;
    config.rcc.hse = Some(Hse {
        freq: Hertz::mhz(8),
        mode: HseMode::Oscillator,
    });
    config.rcc.pll = Some(Pll {
        src: PllSource::HSE,
        mul: PllMul::MUL9,
        prediv: PllPreDiv::DIV1,
    });
    config.rcc.apb1_pre = APBPrescaler::DIV2;

    config
}

pub fn time_checker() -> impl Fn() -> u64 {
    use embassy_time::Instant;

    let t = Instant::now();

    move || t.elapsed().as_micros()
}
