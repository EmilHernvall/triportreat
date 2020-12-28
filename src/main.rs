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

mod trafiklab {
    use chrono::prelude::*;
    use serde_derive::Deserialize;

    #[derive(Debug,Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct RealtimeDepartureInfo {
        /// ex: tunnelbanans gröna linje
        pub group_of_line: String,
        /// ex: 1 min
        pub display_time: String,
        /// ex: METRO
        pub transport_mode: String,
        /// ex: Hagsätra
        pub destination: String,
        /// ex: 2
        pub journey_direction: u32,
        /// ex: Slussen
        pub stop_area_name: String,
        /// ex: 1011
        pub stop_area_number: u32,
        /// ex: 1012
        pub stop_point_number: u32,
        /// ex: 4
        pub stop_point_designation: String,
        /// ex: 2020-12-27T00:12:00
        pub time_tabled_date_time: NaiveDateTime,
        /// ex: 2020-12-27T00:12:00
        pub expected_date_time: NaiveDateTime,
        /// ex: 14759
        pub journey_number: u32,
    }

    #[derive(Debug,Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct RealtimeDeparturesResponseData {
        pub metros: Vec<RealtimeDepartureInfo>,
    }

    #[derive(Debug,Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct RealtimeDeparturesV4Response {
        pub response_data: RealtimeDeparturesResponseData,
    }
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

fn read_test_data() -> Result<trafiklab::RealtimeDeparturesResponseData> {
    let trafiklab::RealtimeDeparturesV4Response { response_data } =
        serde_json::from_str(
            &std::fs::read_to_string("./test/data/sl.json")?,
        )?;

    Ok(response_data)
}

fn fetch_from_server(api_key: &str, station_id: u32) -> Result<trafiklab::RealtimeDeparturesResponseData> {
    let url = format!(
        "https://api.sl.se/api2/realtimedeparturesV4.json?key={}&siteid={}&timewindow=60",
        api_key,
        station_id,
    );
    let response = dbg!(ureq::get(&url).call());
    if !response.ok() {
        panic!("error {}: {}", response.status(), response.into_string()?);
    }

    let trafiklab::RealtimeDeparturesV4Response { response_data } =
        response.into_json_deserialize()?;

    Ok(response_data)
}

fn main() -> Result<()> {
    let opt: Opt = argh::from_env();

    let data = if opt.debug {
        dbg!(read_test_data()?)
    } else {
        dbg!(fetch_from_server(&opt.api_key, opt.station_id)?)
    };

    let mut screen = create_screen()?;

    loop {
        // draw_pattern(&mut screen);
        let mut y_offset = 0;
        for departure in &data.metros {
            screen.draw_text(
                10,
                y_offset,
                32.0,
                &format!("{} {}", departure.destination, departure.display_time),
            );
            y_offset += 32;

            if y_offset + 32 > screen.yres() {
                break;
            }
        }
        screen.flip();
        std::thread::sleep(std::time::Duration::from_millis(1000/60));
    }
}
