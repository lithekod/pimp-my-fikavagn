/*!
 * Blink the builtin LED - the "Hello World" of embedded programming.
 */
#![no_std]
#![no_main]

use ws2812_spi::prerendered::Ws2812;

use embedded_hal::spi::FullDuplex;
use smart_leds::{RGB, RGB8, SmartLedsWrite, hsv::{hsv2rgb, Hsv}};

use panic_halt as _;

struct UnicornHAT<'b, SPI> {
    pixels: [RGB8; 64],
    inner: Ws2812<'b, SPI>,
}

impl<'b, SPI> UnicornHAT<'b, SPI>
where SPI: FullDuplex<u8> {
    fn new(ws2812: Ws2812<'b, SPI>) -> Self {
        Self {
            pixels: [RGB {r: 0, g: 0, b: 0}; 64],
            inner: ws2812,
        }
    }

    fn send(&mut self) -> Result<(), SPI::Error> {
        self.inner.write(self.pixels.iter().cloned())
    }

    fn set_at(&mut self, x: usize, y: usize, c: RGB8) {
        assert!(x < 8);
        assert!(y < 8);
        self.pixels[y * 8 + x] = c;
    }

    fn set_all(&mut self, c: RGB8) {
        self.pixels = [c; 64];
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());

    let mut buffer = [0; 1024];

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

    let ws2812 = Ws2812::new(spi, &mut buffer);
    let mut unicorn = UnicornHAT::new(ws2812);

    let mut i: u8 = 0;

    loop {
        for y in 0..8 {
            for x in 0..8 {
                let offset = y * 32 + x * 4;
                unicorn.set_at(x, y, hsv2rgb(Hsv { hue: i.overflowing_add(offset as u8).0, sat: 255, val: 16 }));
            }
        }
        unicorn.send().unwrap();
        i = i.overflowing_add(1).0;
        arduino_hal::delay_ms(1);
    }
}
