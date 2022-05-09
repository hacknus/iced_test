mod systemchart;
mod gui;
mod pulse;

// plotters-iced
//
// Iced backend for Plotters
// Copyright: 2022, Joylei <leingliu@gmail.com>
// License: MIT
extern crate iced;
extern crate plotters;
extern crate sysinfo;

use chrono::{DateTime, Utc};
use iced::{
    canvas::{Cache, Frame, Geometry},
    executor, scrollable, Align, Application, Clipboard, Column, Command, Container, Element, Font,
    HorizontalAlignment, Length, Row, Scrollable, Settings, Size, Space, Subscription,
    VerticalAlignment,
};
use plotters::prelude::ChartBuilder;
use plotters_iced::{Chart, ChartWidget};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use sysinfo::{ProcessorExt, RefreshKind, System, SystemExt};
use crate::gui::State;

const PLOT_SECONDS: usize = 60; //1 min
const TITLE_FONT_SIZE: u16 = 22;
const SAMPLE_EVERY: Duration = Duration::from_millis(1000);

const FONT_REGULAR: Font = Font::External {
    name: "sans-serif-regular",
    bytes: include_bytes!("./fonts/notosans-regular.ttf"),
};

const FONT_BOLD: Font = Font::External {
    name: "sans-serif-bold",
    bytes: include_bytes!("./fonts/notosans-bold.ttf"),
};

type Real = f32;


fn main() {
    State::run(Settings {
        antialiasing: true,
        default_font: Some(include_bytes!("./fonts/notosans-regular.ttf")),
        ..Settings::default()
    })
        .unwrap();
}
