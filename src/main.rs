/*!
 * Blink the builtin LED - the "Hello World" of embedded programming.
 */
#![no_std]
#![no_main]

use arraydeque::ArrayDeque;
use ws2812_spi::prerendered::Ws2812;

use embedded_hal::spi::FullDuplex;
use smart_leds::{SmartLedsWrite, RGB, RGB8};

use panic_halt as _;

enum Direction {
    Up,
    Down,
    Left,
    Right
}

struct UnicornHAT<'b, SPI> {
    pixels: [RGB8; 64],
    inner: Ws2812<'b, SPI>,
}

impl<'b, SPI> UnicornHAT<'b, SPI>
where
    SPI: FullDuplex<u8>,
{
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
        //TODO (x=0, y=0) doesn't match the usual (0, 0). should be either top left or bottom left
        if y % 2 == 0 {
            self.pixels[y * 8 + x] = c;
        } else {
            self.pixels[y * 8 + (7 - x)] = c;
        }
    }

    fn set_all(&mut self, c: RGB8) {
        self.pixels = [c; 64];
    }

    fn set_snake<'i, I: IntoIterator<Item = &'i (usize, usize)>>(&mut self, snake: I) {
        for (x, y) in snake.into_iter() {
            self.set_at(*x, *y, RGB8 { r: 32, g: 0, b: 0 });
        }
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

    let mut snake: ArrayDeque<[(usize, usize); 64]> = ArrayDeque::new();
    snake.push_front((4, 4)).unwrap();
    snake.push_front((5, 4)).unwrap();
    snake.push_front((6, 4)).unwrap();

    let mut dir = Direction::Right; // +x
    let mut apple = (2, 2);

    //TODO countdown? pixels in some corner
    arduino_hal::delay_ms(2000);
    //TODO signal initial direction with flashing

    loop {
        let dx = a1.analog_read(&mut adc);
        let dy = a2.analog_read(&mut adc);

        if dx > 1023 - 100 {
            dir = Direction::Right;
        } else if dx < 100 {
            dir = Direction::Left;
        } else if dy > 1023 - 100 {
            dir = Direction::Up;
        } else if dy < 100 {
            dir = Direction::Down;
        }

        let old = snake.back().unwrap();
        let new = match dir {
            Direction::Up => (old.0, old.1 + 1),
            Direction::Down => (old.0, old.1 - 1),
            Direction::Left => (old.0 - 1, old.1),
            Direction::Right => (old.0 + 1, old.1),
        };

        if new.0 > 7 || new.1 > 7 {
            loop {} //TODO you died
        }

        for part in snake.iter() {
            if &new == part {
                loop {} //TODO you died. flash collision
            }
        }

        if new != apple {
            snake.pop_front();
        }
        snake.push_back(new).unwrap();

        unicorn.set_all(RGB::default());
        unicorn.set_at(apple.0, apple.1, RGB { r: 0, g: 32, b: 0 });
        unicorn.set_snake(&snake); //TODO gradient from head to tail
        unicorn.send().unwrap();

        arduino_hal::delay_ms(500);
    }
}
