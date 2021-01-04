use crate::{buffer::Buffer, Result};

#[cfg(feature = "hw-pi")]
pub mod pi;
#[cfg(feature = "hw-sdl")]
pub mod sdl;

#[cfg(feature = "hw-pi")]
pub fn create_hardware() -> Result<pi::PiHardware> {
    pi::PiHardware::open("/dev/fb1")
}

#[cfg(feature = "hw-sdl")]
pub fn create_hardware() -> Result<sdl::SdlHardware> {
    sdl::SdlHardware::open()
}

pub enum HwEvent {
    Scroll(isize),
}

pub trait Hardware {
    fn xres(&self) -> u32;
    fn yres(&self) -> u32;
    fn max(&self) -> [u8; 3];
    fn poll_events(&mut self) -> Result<Vec<HwEvent>>;
    fn flip(&mut self, buffer: &Buffer) -> Result<()>;
}
