use std::rc::Rc;

use sdl2::{
    EventPump,
    event::Event,
    keyboard::Keycode,
    pixels::Color,
    render::Canvas,
    video::Window,
};
use rusttype::Font;

use crate::{Result,Screen,Rgb};

pub struct SdlScreen {
    canvas: Canvas<Window>,
    event_pump: EventPump,
    font: Rc<Font<'static>>,
}

impl SdlScreen {
    pub fn open() -> Result<Self> {
        let context = sdl2::init()?;
        let video = context.video()?;

        let window = video.window("rust-sdl2 demo", 480, 320)
            .position_centered()
            .build()?;

        let mut canvas = window.into_canvas().build()?;

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();
        let event_pump = context.event_pump()?;

        let font = Rc::new(Font::try_from_bytes(ttf_noto_sans::REGULAR)
            .expect("Error constructing Font"));

        Ok(SdlScreen {
            canvas,
            event_pump,
            font,
        })
    }
}

impl Screen for SdlScreen {
    fn xres(&self) -> u32 {
        self.canvas.output_size().unwrap().0
    }

    fn yres(&self) -> u32 {
        self.canvas.output_size().unwrap().1
    }

    fn max(&self) -> [u8; 3] {
        [0xFF, 0xFF, 0xFF]
    }

    fn font(&self) -> Rc<Font<'static>> {
        self.font.clone()
    }

    fn set_pixel(&mut self, x: u32, y: u32, rgb: Rgb) {
        self.canvas.set_draw_color(Color::RGB(rgb[0], rgb[1], rgb[2]));
        self.canvas.draw_point((x as i32, y as i32)).unwrap();
    }

    fn flip(&mut self) {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    std::process::exit(0);
                },
                _ => {}
            }
        }

        self.canvas.present();
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
    }
}
