use std::rc::Rc;

use argh::FromArgs;

use crate::{
    gadget::{Gadget, HorizontalGadget, ScrollGadget, TextGadget, RenderRect},
    hardware::{create_hardware, Hardware, HwEvent},
};

pub mod buffer;
pub mod hardware;
pub mod gadget;

type Error = Box<dyn std::error::Error + Sync + Send + 'static>;
type Result<T> = std::result::Result<T, Error>;

/// Trip or Treat
#[derive(FromArgs)]
struct Opt {
    /// activate debug mode
    #[argh(option, default = "false")]
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

    #[derive(Debug, Deserialize)]
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

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct RealtimeDeparturesResponseData {
        pub metros: Vec<RealtimeDepartureInfo>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct RealtimeDeparturesV4Response {
        pub response_data: RealtimeDeparturesResponseData,
    }
}

fn read_test_data() -> Result<trafiklab::RealtimeDeparturesResponseData> {
    let trafiklab::RealtimeDeparturesV4Response { response_data } =
        serde_json::from_str(&std::fs::read_to_string("./test/data/sl.json")?)?;

    Ok(response_data)
}

fn fetch_from_server(
    api_key: &str,
    station_id: u32,
) -> Result<trafiklab::RealtimeDeparturesResponseData> {
    let url = format!(
        "https://api.sl.se/api2/realtimedeparturesV4.json?key={}&siteid={}&timewindow=60",
        api_key, station_id,
    );
    let response = ureq::get(&url).call();
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

    let mut hw = create_hardware()?;
    let mut buffer = buffer::Buffer::new(hw.xres(), hw.yres());

    let mut outer_layout = ScrollGadget::new(buffer.width(), buffer.height());
    let clock = Rc::new(TextGadget::new(
        format!("{}", chrono::Local::now().time()),
        buffer.width(),
        32,
        [1.0, 1.0, 1.0],
        32.0,
    ));
    outer_layout.children.push(clock.clone());

    let mut inner_layout = ScrollGadget::new(buffer.height(), buffer.width());
    for (i, departure) in data.metros.iter().enumerate() {
        inner_layout.children.push(
            Rc::new(TextGadget::new(
                format!("{} {} {}", i, departure.destination, departure.display_time),
                100,
                32,
                [1.0, 1.0, 0.0],
                32.0,
            )),
        );

        //let mut container = HorizontalGadget::new(buffer.height(), 32);

        //container.children.push(
        //    Rc::new(TextGadget::new(
        //        "foo".to_string(), // format!("{} {}", i, departure.destination),
        //        100,
        //        32,
        //        [1.0, 1.0, 0.0],
        //        32.0,
        //    )),
        //);

        //container.children.push(
        //    Rc::new(TextGadget::new(
        //        "bar".to_string(), // format!("{}", departure.display_time),
        //        50,
        //        32,
        //        [1.0, 1.0, 0.0],
        //        32.0,
        //    )),
        //);

        //inner_layout.children.push(Rc::new(container));
    }
    let inner_layout = Rc::new(inner_layout);
    outer_layout.children.push(inner_layout.clone());

    let mut scroll: isize = 0;
    loop {
        clock.text(format!("{}", chrono::Local::now().time().format("%H:%M:%S")));

        let y_offset = 40;
        let line_size = 32;
        let per_page = (hw.xres() - y_offset) / line_size;
        let max_scroll_pos = (data.metros.len() as isize - per_page as isize).max(0);

        let events = hw.poll_events()?;
        for event in events {
            match event {
                HwEvent::Scroll(delta) => {
                    let updated_scroll = scroll + delta;
                    if updated_scroll < 0 {
                        scroll = 0;
                    } else if updated_scroll > max_scroll_pos {
                        scroll = max_scroll_pos;
                    } else {
                        scroll = updated_scroll;
                    }
                    dbg!(scroll);
                    inner_layout.scroll(scroll as usize);
                }
            }
        }

        let start = std::time::Instant::now();
        let rect = RenderRect {
            x: 0,
            y: 0,
            width: buffer.height(),
            height: buffer.width(),
        };
        if outer_layout.dirty() {
            outer_layout.render(rect, &mut buffer);
            hw.flip(&buffer)?;
            let end = std::time::Instant::now();
            println!("Rendered frame in {}ms", (end - start).as_millis());
        } else {
            std::thread::sleep(std::time::Duration::from_millis(1000 / 60));
        }
    }
}
