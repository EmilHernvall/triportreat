use std::rc::Rc;
use std::cell::{Cell,RefCell};

use once_cell::sync::OnceCell;
use rusttype::{point, Font, Scale};

use crate::buffer::{Buffer,PixelType};
use crate::buffer::Rgb;

fn font() -> &'static Font<'static> {
    static INSTANCE: OnceCell<Font> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        Font::try_from_bytes(ttf_noto_sans::REGULAR)
            .expect("Error constructing Font")
    })
}

#[derive(Debug)]
pub struct RenderRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub trait Gadget<P: PixelType> {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn dirty(&self) -> bool;
    fn render(&self, rect: RenderRect, buffer: &mut Buffer<P>);
}

pub struct HorizontalGadget<P: PixelType> {
    pub width: u32,
    pub height: u32,
    pub dirty: Cell<bool>,
    pub children: Vec<Rc<dyn Gadget<P>>>,
}

impl<P: PixelType> HorizontalGadget<P> {
    pub fn new(width: u32, height: u32) -> HorizontalGadget<P> {
        HorizontalGadget {
            width,
            height,
            dirty: Cell::new(true),
            children: vec![],
        }
    }
}

impl<P: PixelType> Gadget<P> for HorizontalGadget<P> {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn dirty(&self) -> bool {
        self.children.iter().any(|x| x.dirty()) || self.dirty.get()
    }

    fn render(&self, rect: RenderRect, buffer: &mut Buffer<P>) {
        let mut offset = 0;
        let dirty = self.dirty();
        for child in &self.children {
            if dirty || child.dirty() {
                let rect = RenderRect {
                    x: rect.x + offset,
                    y: rect.y,
                    width: child.width(),
                    height: rect.height,
                };
                child.render(rect, buffer);
            }
            offset += child.width();
        }
        self.dirty.set(false);
    }
}

pub struct ScrollGadget<P: PixelType> {
    pub width: u32,
    pub height: u32,
    pub scroll: Cell<usize>,
    pub dirty: Cell<bool>,
    pub children: Vec<Rc<dyn Gadget<P>>>,
}

impl<P: PixelType> ScrollGadget<P> {
    pub fn new(width: u32, height: u32) -> ScrollGadget<P> {
        ScrollGadget {
            width,
            height,
            scroll: Cell::new(0),
            dirty: Cell::new(true),
            children: vec![],
        }
    }

    pub fn scroll(&self, scroll: usize) {
        self.scroll.set(scroll);
        self.dirty.set(true);
    }
}

impl<P: PixelType> Gadget<P> for ScrollGadget<P> {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn dirty(&self) -> bool {
        self.children.iter().any(|x| x.dirty()) || self.dirty.get()
    }

    fn render(&self, rect: RenderRect, buffer: &mut Buffer<P>) {
        let mut offset = 0;
        let dirty = self.dirty();
        for child in self.children.iter().skip(self.scroll.get()) {
            if dirty || child.dirty() {
                let rect = RenderRect {
                    x: rect.x,
                    y: rect.y + offset,
                    width: rect.width,
                    height: child.height(),
                };
                child.render(rect, buffer);
            }
            offset += child.height();
        }
        self.dirty.set(false);
    }
}

pub struct TextGadget {
    text: RefCell<String>,
    size: f32,
    color: Rgb,
    dirty: Cell<bool>,
    width: u32,
    height: u32,
}

impl TextGadget {
    pub fn new(text: String, width: u32, height: u32, color: Rgb, size: f32) -> TextGadget {
        TextGadget {
            text: RefCell::new(text),
            color,
            size,
            dirty: Cell::new(true),
            width,
            height,
        }
    }

    pub fn text(&self, text: String) {
        let old_text = self.text.replace(text.clone());
        if text != old_text {
            self.dirty.set(true);
        }
    }
}

impl<P: PixelType> Gadget<P> for TextGadget {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn dirty(&self) -> bool {
        self.dirty.get()
    }

    fn render(&self, rect: RenderRect, buffer: &mut Buffer<P>) {
        let scale = Scale::uniform(self.size);

        let font = font();
        let v_metrics = font.v_metrics(scale);
        let p = point(rect.x as f32, rect.y as f32 + v_metrics.ascent);
        let glyphs: Vec<_> = font
            .layout(&self.text.borrow(), scale, p)
            .collect();

        for y in rect.y..(rect.y + rect.height) {
            for x in rect.x..(rect.x + rect.width) {
                buffer.clear_pixel(y, x);
            }
        }

        for glyph in glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                glyph.draw(|x, y, v| {
                    let x = buffer.height() - (x + bounding_box.min.x as u32);
                    let y = y + bounding_box.min.y as u32;

                    if x < rect.x { return; }
                    if y < rect.y { return; }
                    if x > rect.x + rect.width { return; }
                    if y > rect.y + rect.height { return; }

                    let bg = match buffer.get_pixel(y, x) {
                        Some(p) => p,
                        None => return,
                    };

                    let output = [
                        (v * self.color[0] + (1.0 - v) * bg[0]),
                        (v * self.color[1] + (1.0 - v) * bg[1]),
                        (v * self.color[2] + (1.0 - v) * bg[2]),
                    ];

                    buffer.set_pixel(y, x, output);
                });
            }
        }

        self.dirty.set(false);
    }
}
