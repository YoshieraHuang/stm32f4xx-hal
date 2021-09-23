#![deny(unsafe_code)]
#![no_main]
#![no_std]

use panic_halt as _;
use stm32f4xx_hal as hal;

use cortex_m_rt::entry;
use hal::{gpio::NoPin, pac, prelude::*, spi::Spi};
use smart_leds::{brightness, hsv::RGB8, SmartLedsWrite};
use ws2812_spi as ws2812;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().expect("cannot take peripherals");
    let cp = cortex_m::Peripherals::take().expect("cannot take core peripherals");

    // Configure APB bus clock to 56MHz, cause ws2812b requires 3.5Mbps SPI
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(56.mhz()).freeze();

    let mut delay = hal::delay::Delay::new(cp.SYST, &clocks);
    let gpioa = dp.GPIOA.split();

    let spi = Spi::new(
        dp.SPI1,
        (gpioa.pa5, NoPin, gpioa.pa7),
        ws2812::MODE,
        3500.khz(),
        clocks,
    );

    let mut ws = ws2812::Ws2812::new(spi);

    const NUM_LEDS: usize = 8;
    let mut data = [RGB8::default(); NUM_LEDS];

    loop {
        for j in 0..(256 * 5) {
            for i in 0..NUM_LEDS {
                data[i] = wheel((((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as u8);
            }
            ws.write(brightness(data.iter().cloned(), 32)).unwrap();
            delay.delay_ms(5u8);
        }
    }
}

/// Input a value 0 to 255 to get a color value
/// The colours are a transition r - g - b - back to r.
fn wheel(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return (255 - wheel_pos * 3, 0, wheel_pos * 3).into();
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return (0, wheel_pos * 3, 255 - wheel_pos * 3).into();
    }
    wheel_pos -= 170;
    (wheel_pos * 3, 255 - wheel_pos * 3, 0).into()
}
