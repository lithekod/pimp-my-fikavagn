/*!
 * Blink the builtin LED - the "Hello World" of embedded programming.
 */
#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::usart::Usart;

use tinyvec::ArrayVec;
use ws2812_spi::prerendered::Ws2812;

use smart_leds::{
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite, RGB,
};

use nb::block;
use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());

    let a0 = pins.a0.into_analog_input(&mut adc);

    // Digital pin 13 is also connected to an onboard LED marked "L"
    // let mut led = pins.d13.into_output();
    // led.set_high();

    let mut buffer = [0; 512];

    let sclk = pins.d13.into_output();
    let mosi = pins.d11.into_output();
    let miso = pins.d12.into_pull_up_input();
    let cs = pins.d10.into_output();

    let (spi, _) = arduino_hal::spi::Spi::new(
        dp.SPI,
        sclk,
        mosi,
        miso,
        cs,
        arduino_hal::spi::Settings::default(),
    );

    let mut ws2812 = Ws2812::new(spi, &mut buffer);

    arduino_hal::delay_ms(100);

    ws2812.write([(0u8, 255u8, 0u8)].iter().cloned()).unwrap();
    ws2812.write([(255u8, 0u8, 0u8)].iter().cloned()).unwrap();
    ws2812.write([(0u8, 0u8, 255u8)].iter().cloned()).unwrap();

    let mut hues = ArrayVec::<[u8; 9]>::new();

    loop {
        let hue = (a0.analog_read(&mut adc) / 4) as u8;

        hues.insert(0, hue);
        hues.truncate(8);

        ws2812
            .write(
                hues.iter()
                    .cloned()
                    .map(|hue| {
                        hsv2rgb(Hsv {
                            hue,
                            sat: 255,
                            val: 16,
                        })
                    })
            )
            .unwrap();
        // let val = adc.read_blocking(&a0);
        arduino_hal::delay_ms(50);

        // ufmt::uwriteln!(&mut serial, "{}!\r", val).void_unwrap();
    }
}
