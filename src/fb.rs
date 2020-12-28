use std::fs::File;
use std::rc::Rc;

use framebuffer::{Framebuffer, VarScreeninfo};
use rusttype::Font;

use crate::{Result,Screen,Rgb};

pub struct FramebufferScreen {
    vinf: VarScreeninfo,
    max: [u8; 3],
    fb: Framebuffer,
    buffer: Vec<u8>,
    font: Rc<Font<'static>>,
}

impl FramebufferScreen {
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
        let buffer = (0..(2*vinf.yres*vinf.xres))
            .map(|_| 0)
            .collect::<Vec<u8>>();

        let font = Rc::new(Font::try_from_bytes(ttf_noto_sans::REGULAR)
            .expect("Error constructing Font"));

        Ok(FramebufferScreen {
            vinf,
            max,
            fb,
            buffer,
            font,
        })
    }
}

impl Screen for FramebufferScreen {
    fn xres(&self) -> u32 {
        self.vinf.xres
    }

    fn yres(&self) -> u32 {
        self.vinf.yres
    }

    fn max(&self) -> [u8; 3] {
        self.max
    }

    fn font(&self) -> Rc<Font<'static>> {
        self.font.clone()
    }

    fn set_pixel(&mut self, x: u32, y: u32, rgb: Rgb) {
        let c = (((rgb[0] & self.max[0]) as u32) << self.vinf.red.offset) |
                (((rgb[1] & self.max[1]) as u32) << self.vinf.green.offset) |
                (((rgb[2] & self.max[2]) as u32) << self.vinf.blue.offset);

        let (fst, snd) = ((c & 0xFF00) >> 8, c & 0x00FF);

        let pos = 2*(y*self.vinf.xres + x) as usize;

        self.buffer[pos] = fst as u8;
        self.buffer[pos+1] = snd as u8;
    }

    fn flip(&mut self) {
        self.fb.write_frame(&self.buffer);
        for b in self.buffer.iter_mut() {
            *b = 0;
        }
    }
}
