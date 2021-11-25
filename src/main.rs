/*!
 * Blink the builtin LED - the "Hello World" of embedded programming.
 */
#![no_std]
#![no_main]

use ws2812_spi::prerendered::Ws2812;

use embedded_hal::spi::FullDuplex;
use smart_leds::{RGB, RGB8, SmartLedsWrite};

use panic_halt as _;

struct UnicornHAT<'b, SPI> {
    pixels: [RGB8; 64],
    inner: Ws2812<'b, SPI>,
}

impl<'b, SPI> UnicornHAT<'b, SPI>
where SPI: FullDuplex<u8> {
    fn new(ws2812: Ws2812<'b, SPI>) -> Self {
        Self {
            pixels: [RGB::default(); 64],
            inner: ws2812,
        }
    }

    fn send(&mut self) -> Result<(), SPI::Error> {
        self.inner.write(self.pixels.iter().cloned())
    }

    fn set_at(&mut self, x: usize, y: usize, c: RGB8) {
        assert!(x < 8);
        assert!(y < 8);
        if y % 2 == 0 {
            self.pixels[y * 8 + x] = c;
        } else {
            self.pixels[y * 8 + (7 - x)] = c;
        }
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

    let _a0 = pins.a0.into_analog_input(&mut adc);
    let a1 = pins.a1.into_analog_input(&mut adc);
    let a2 = pins.a2.into_analog_input(&mut adc);

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

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

    let mut x: u8 = 4;
    let mut y: u8 = 4;
    let mut moving = false;

    loop {
        let dx = a1.analog_read(&mut adc);
        let dy = a2.analog_read(&mut adc);

        if dx > 1023-100 && !moving {
            x  = x.wrapping_add(1);
            moving = true;
        } else if dx < 100 && !moving {
            x = x.wrapping_sub(1);
            moving = true;
        } else if dy > 1023-100 && !moving {
            y = y.wrapping_add(1);
            moving = true;
        } else if dy < 100 && !moving {
            y  = y.wrapping_sub(1);
            moving = true;
        }

        ufmt::uwriteln!(serial, "x: {}, y: {}, dx: {}, dy: {}\r", x, y, dx, dy).unwrap();

        if dx != 0 && dx != 1023 && dy != 0 && dy != 1023 {
            moving = false;
        }

        if x < 8 && y < 8 {
            unicorn.set_all(RGB::default());
            unicorn.set_at(x as usize, y as usize, RGB { r: 32, g: 0, b: 0});
            unicorn.send().unwrap();
        }

        arduino_hal::delay_ms(100);
    }
}
