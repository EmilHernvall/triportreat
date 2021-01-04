use sdl2::{
    event::Event, keyboard::Keycode, pixels::Color, render::Canvas, video::Window, EventPump,
};

use crate::{
    buffer::Buffer,
    hardware::{Hardware, HwEvent},
    Result,
};

pub struct SdlHardware {
    canvas: Canvas<Window>,
    event_pump: EventPump,
}

impl SdlHardware {
    pub fn open() -> Result<Self> {
        let context = sdl2::init()?;
        let video = context.video()?;

        let window = video
            .window("rust-sdl2 demo", 320, 480)
            .position_centered()
            .build()?;

        let mut canvas = window.into_canvas().build()?;

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();
        let event_pump = context.event_pump()?;

        Ok(SdlHardware { canvas, event_pump })
    }
}

impl Hardware for SdlHardware {
    fn xres(&self) -> u32 {
        self.canvas.output_size().unwrap().1
    }

    fn yres(&self) -> u32 {
        self.canvas.output_size().unwrap().0
    }

    fn max(&self) -> [u8; 3] {
        [0xFF, 0xFF, 0xFF]
    }

    fn poll_events(&mut self) -> Result<Vec<HwEvent>> {
        let mut events = vec![];
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    std::process::exit(0);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    events.push(HwEvent::Scroll(1));
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    events.push(HwEvent::Scroll(-1));
                }
                _ => {}
            }
        }

        Ok(events)
    }

    fn flip(&mut self, buffer: &Buffer) -> Result<()> {
        for (x, y, rgb) in buffer.pixels() {
            self.canvas
                .set_draw_color(Color::RGB(rgb[0], rgb[1], rgb[2]));
            self.canvas
                .draw_point((self.yres() as i32 - y as i32, x as i32))
                .unwrap();
        }

        self.canvas.present();
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();

        Ok(())
    }
}
