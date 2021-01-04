use std::rc::Rc;

use rusttype::{point, Font, Scale};

pub type Rgb = [u8; 3];

pub struct Buffer {
    width: u32,
    height: u32,
    data: Vec<Rgb>,
    font: Rc<Font<'static>>,
}

impl Buffer {
    pub fn new(width: u32, height: u32) -> Buffer {
        let font =
            Rc::new(Font::try_from_bytes(ttf_noto_sans::REGULAR).expect("Error constructing Font"));

        Buffer {
            width,
            height,
            data: (0..(width * height)).map(|_| [0, 0, 0]).collect(),
            font,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn clear(&mut self) {
        for item in self.data.iter_mut() {
            *item = [0, 0, 0];
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, rgba: Rgb) {
        if x >= self.width {
            return;
        }
        if y >= self.height {
            return;
        }

        self.data[(self.width * y + x) as usize] = rgba;
    }

    pub fn get_pixel(&mut self, x: u32, y: u32) -> Option<Rgb> {
        if x >= self.width {
            return None;
        }
        if y >= self.height {
            return None;
        }

        Some(self.data[(self.width * y + x) as usize])
    }

    pub fn draw_text(&mut self, x: u32, y: u32, size: f32, text: &str, color: Rgb) {
        let scale = Scale::uniform(size);

        let font = self.font.clone();
        let v_metrics = font.v_metrics(scale);
        let glyphs: Vec<_> = font
            .layout(text, scale, point(x as f32, y as f32 + v_metrics.ascent))
            .collect();

        for glyph in glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                glyph.draw(|x, y, v| {
                    let x = self.height - (x + bounding_box.min.x as u32);
                    let y = y + bounding_box.min.y as u32;

                    let bg = match self.get_pixel(y, x) {
                        Some(p) => p,
                        None => return,
                    };

                    let output = [
                        (v * color[0] as f32 + (1.0 - v) * bg[0] as f32) as u8,
                        (v * color[1] as f32 + (1.0 - v) * bg[1] as f32) as u8,
                        (v * color[2] as f32 + (1.0 - v) * bg[2] as f32) as u8,
                    ];

                    self.set_pixel(y, x, output);
                });
            }
        }
    }

    pub fn pixels<'a>(&'a self) -> impl Iterator<Item = (u32, u32, Rgb)> + 'a {
        let width = self.width();
        self.data.iter().enumerate().map(move |(i, rgb)| {
            let x = i % width as usize;
            let y = (i - x) / width as usize;

            (x as u32, y as u32, *rgb)
        })
    }
}
