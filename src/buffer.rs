pub type Rgb = [f32; 3];

pub trait PixelType: Copy {
    fn black() -> Self;
    fn set(&mut self, rgb: Rgb);
    fn get(&self) -> Rgb;
}

impl PixelType for [u8; 3] {
    fn black() -> Self {
        [0, 0, 0]
    }

    fn set(&mut self, rgb: Rgb) {
        self[0] = (rgb[0]*0xFF as f32) as u8;
        self[1] = (rgb[1]*0xFF as f32) as u8;
        self[2] = (rgb[2]*0xFF as f32) as u8;
    }

    fn get(&self) -> Rgb {
        [
            self[0] as f32 / 0xFF as f32,
            self[1] as f32 / 0xFF as f32,
            self[2] as f32 / 0xFF as f32,
        ]
    }
}

impl PixelType for u16 {
    fn black() -> Self {
        0u16
    }

    #[inline]
    fn set(&mut self, rgb: Rgb) {
        *self = (((rgb[0] * 31.0) as u16) << 11)
        | (((rgb[1] * 63.0) as u16) << 5)
        | ((rgb[2] * 31.0) as u16);
    }

    #[inline]
    fn get(&self) -> Rgb {
        [
            ((self & 0xf800) >> 11) as f32 / 31.0,
            ((self & 0x07e0) >> 5) as f32 / 63.0,
            (self & 0x001f) as f32 / 31.0,
        ]
    }
}

pub struct Buffer<P: PixelType> {
    width: u32,
    height: u32,
    pub data: Vec<P>,
}

impl<P: PixelType> Buffer<P> {
    pub fn new(width: u32, height: u32) -> Buffer<P> {
        Buffer {
            width,
            height,
            data: (0..(width * height)).map(|_| P::black()).collect(),
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
            *item = P::black();
        }
    }

    pub fn clear_pixel(&mut self, x: u32, y: u32) {
        if x >= self.width {
            return;
        }
        if y >= self.height {
            return;
        }

        self.data[(self.width * y + x) as usize] = P::black();
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, rgb: Rgb) {
        if x >= self.width {
            return;
        }
        if y >= self.height {
            return;
        }

        self.data[(self.width * y + x) as usize].set(rgb);
    }

    pub fn get_pixel(&mut self, x: u32, y: u32) -> Option<Rgb> {
        if x >= self.width {
            return None;
        }
        if y >= self.height {
            return None;
        }

        Some(self.data[(self.width * y + x) as usize].get())
    }

    pub fn pixels<'a>(&'a self) -> impl Iterator<Item = (u32, u32, P)> + 'a {
        let width = self.width();
        self.data.iter().enumerate().map(move |(i, rgb)| {
            let x = i % width as usize;
            let y = (i - x) / width as usize;

            (x as u32, y as u32, *rgb)
        })
    }
}
