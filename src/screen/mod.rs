use std::rc::Rc;

use rusttype::{point, Font, Scale};

use crate::Result;

#[cfg(feature = "with-framebuffer")]
pub mod fb;
#[cfg(feature = "with-sdl")]
pub mod sdl;

type Rgb = [u8; 3];

#[cfg(feature = "with-framebuffer")]
pub fn create_screen() -> Result<fb::FramebufferScreen> {
    fb::FramebufferScreen::open("/dev/fb1")
}

#[cfg(feature = "with-sdl")]
pub fn create_screen() -> Result<sdl::SdlScreen> {
    sdl::SdlScreen::open()
}

pub trait Screen {
    fn xres(&self) -> u32;
    fn yres(&self) -> u32;
    fn max(&self) -> [u8; 3];
    fn font(&self) -> Rc<Font<'static>>;
    fn set_pixel(&mut self, x: u32, y: u32, rgb: Rgb);

    fn draw_text(&mut self, x: u32, y: u32, size: f32, text: &str) {
        let scale = Scale::uniform(size);

        let font = self.font();
        let v_metrics = font.v_metrics(scale);
        let glyphs: Vec<_> = font
            .layout(text, scale, point(x as f32, y as f32 + v_metrics.ascent))
            .collect();

        let [r_max, g_max, b_max] = self.max();
        for glyph in glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                glyph.draw(|x, y, v| {
                    if x > self.xres() {
                        return;
                    }
                    if y > self.yres() {
                        return;
                    }

                    let r = (v*r_max as f32) as u8;
                    let g = (v*g_max as f32) as u8;
                    let b = (v*b_max as f32) as u8;

                    let x = x + bounding_box.min.x as u32;
                    let y = y + bounding_box.min.y as u32;

                    self.set_pixel(x, y, [r, g, b]);
                });
            }
        }
    }

    fn flip(&mut self);
}


