use argh::FromArgs;

use crate::screen::{Screen, create_screen};

mod screen;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

/// Trip or Treat
#[derive(FromArgs)]
struct Opt {
    /// activate debug mode
    #[argh(option, default="false")]
    debug: bool,

    /// station id
    #[argh(option)]
    station_id: u32,

    /// trafiklab api key
    #[argh(option)]
    api_key: String,
}

fn draw_pattern<S: Screen>(screen: &mut S) {
    for y in 0..screen.yres() {
        for x in 0..screen.xres() {
            let [r_max, _, b_max] = screen.max();
            let (r, g, b) : (u8, u8, u8) = (
                (x * r_max as u32 / screen.xres()) as u8,
                0,
                (x * b_max as u32 / screen.xres()) as u8,
            );

            screen.set_pixel(x, y, [r, g, b]);
        }
    }
}

fn main() -> Result<()> {
    let opt: Opt = argh::from_env();

    let mut screen = create_screen()?;

    loop {
        draw_pattern(&mut screen);
        screen.draw_text(10, 10, 32.0, "Testing with some smaller text");
        screen.flip();
        std::thread::sleep(std::time::Duration::from_millis(1000/60));
    }
}
