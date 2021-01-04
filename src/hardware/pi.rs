use std::fs::File;
use std::sync::{Arc, Mutex};

use framebuffer::{Framebuffer, VarScreeninfo};
use rppal::gpio::{Gpio, Level, Trigger};

use crate::{
    buffer::{Buffer, Rgb},
    Error, Hardware, HwEvent, Result,
};

#[derive(Debug)]
struct GPIOState {
    pin1: bool,
    pin2: bool,
    counter: i32,
}

fn gpio_handler(events: Arc<Mutex<Vec<HwEvent>>>) -> Result<()> {
    let state = Arc::new(Mutex::new(GPIOState {
        pin1: false,
        pin2: false,
        counter: 0,
    }));

    let gpio = Gpio::new()?;

    let pin1 = Arc::new(Mutex::new(gpio.get(17)?.into_input_pulldown()));
    let pin2 = Arc::new(Mutex::new(gpio.get(18)?.into_input_pulldown()));
    let mut button = gpio.get(27)?.into_input_pullup();
    let mut led1 = gpio.get(22)?.into_output();
    let mut led2 = gpio.get(23)?.into_output();

    let handler = {
        let pin1 = pin1.clone();
        let pin2 = pin2.clone();
        let state = state.clone();
        Arc::new(move |pin_no| {
            let val1 = pin1.lock().unwrap().read() == Level::High;
            let val2 = pin2.lock().unwrap().read() == Level::High;

            let mut state = state.lock().unwrap();
            if val1 == state.pin1 && val2 == state.pin2 {
                return;
            }

            state.pin1 = val1;
            state.pin2 = val2;

            if val1 && val2 {
                if pin_no == 2 {
                    state.counter += 1;
                } else {
                    state.counter -= 1;
                }
            }
        })
    };

    pin1.lock()
        .unwrap()
        .set_async_interrupt(Trigger::RisingEdge, {
            let handler = handler.clone();
            move |_| handler(1)
        })?;

    pin2.lock()
        .unwrap()
        .set_async_interrupt(Trigger::RisingEdge, {
            let handler = handler.clone();
            move |_| handler(2)
        })?;

    button.set_async_interrupt(Trigger::FallingEdge, |_| {
        eprintln!("Button pressed");
    })?;

    let mut lights_on = false;
    let mut update_lights = 0u64;
    loop {
        let update = {
            let mut state = state.lock().unwrap();
            let update = state.counter;
            state.counter = 0;
            update
        };

        if update != 0 {
            let mut events = events.lock().unwrap();
            let delta = update * update.abs();
            events.push(HwEvent::Scroll(if delta > 0 { 1 } else { -1 }));
        }

        if update_lights % 50 == 0 {
            if lights_on {
                led1.set_low();
                led2.set_low();
                lights_on = false;
            } else {
                led1.set_high();
                led2.set_high();
                lights_on = true;
            }
        }

        update_lights += 1;
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

pub struct PiHardware {
    vinf: VarScreeninfo,
    max: [u8; 3],
    fb: Framebuffer,
    buffer: Vec<u8>,
    events: Arc<Mutex<Vec<HwEvent>>>,
}

impl PiHardware {
    pub fn open(device: &str) -> Result<Self> {
        let (_finf, vinf) = {
            let file = File::open(device)?;
            let finf = dbg!(Framebuffer::get_fix_screeninfo(&file)?);
            let vinf = dbg!(Framebuffer::get_var_screeninfo(&file)?);

            (finf, vinf)
        };

        let max = [
            (1 << vinf.red.length) - 1,
            (1 << vinf.green.length) - 1,
            (1 << vinf.blue.length) - 1,
        ];

        let fb = Framebuffer::new(device)?;
        let buffer = (0..(2 * vinf.yres * vinf.xres))
            .map(|_| 0)
            .collect::<Vec<u8>>();

        let events = Arc::new(Mutex::new(vec![]));

        std::thread::spawn({
            let events = events.clone();
            move || gpio_handler(events)
        });

        Ok(PiHardware {
            vinf,
            max,
            fb,
            buffer,
            events,
        })
    }

    #[inline]
    fn set_pixel(&mut self, x: u32, y: u32, rgb: Rgb) {
        let c = (((rgb[0] as u32 * self.max[0] as u32) / 0xFF) << self.vinf.red.offset)
            | (((rgb[1] as u32 * self.max[1] as u32) / 0xFF) << self.vinf.green.offset)
            | (((rgb[2] as u32 * self.max[2] as u32) / 0xFF) << self.vinf.blue.offset);

        let (fst, snd) = ((c & 0xFF00) >> 8, c & 0x00FF);

        let pos = 2 * (y * self.vinf.xres + x) as usize;

        self.buffer[pos] = fst as u8;
        self.buffer[pos + 1] = snd as u8;
    }
}

impl Hardware for PiHardware {
    fn xres(&self) -> u32 {
        self.vinf.xres
    }

    fn yres(&self) -> u32 {
        self.vinf.yres
    }

    fn max(&self) -> [u8; 3] {
        self.max
    }

    fn poll_events(&mut self) -> Result<Vec<HwEvent>> {
        let mut events = self
            .events
            .lock()
            .map_err(|_| -> Error { "Poisoned Lock".into() })?;
        let mut fresh_events = vec![];
        std::mem::swap(&mut *events, &mut fresh_events);
        Ok(fresh_events)
    }

    fn flip(&mut self, buffer: &Buffer) -> Result<()> {
        for (x, y, rgb) in buffer.pixels() {
            self.set_pixel(x, y, rgb);
        }

        self.fb.write_frame(&self.buffer);

        Ok(())
    }
}
