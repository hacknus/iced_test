use std::collections::VecDeque;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use iced::{Align, Column, Container, Element, HorizontalAlignment, Length, Row, scrollable, Scrollable, Size, Space, VerticalAlignment};
use iced::canvas::{Cache, Frame, Geometry};
use plotters::chart::ChartBuilder;
use plotters_iced::{Chart, ChartWidget, DrawingBackend, plotters_backend};
use sysinfo::{ProcessorExt, RefreshKind, System, SystemExt};
use crate::{FONT_BOLD, FONT_REGULAR, PLOT_SECONDS, SAMPLE_EVERY};
use crate::gui::Message;

pub struct SystemChart {
    sys: System,
    last_sample_time: Instant,
    items_per_row: usize,
    processors: Vec<CpuUsageChart>,
    scroll: scrollable::State,
    chart_height: u16,
}

impl Default for SystemChart {
    fn default() -> Self {
        Self {
            sys: System::new_with_specifics(RefreshKind::new().with_cpu()),
            last_sample_time: Instant::now(),
            items_per_row: 3,
            processors: Default::default(),
            scroll: Default::default(),
            chart_height: 300,
        }
    }
}

impl SystemChart {
    #[inline]
    fn is_initialized(&self) -> bool {
        !self.processors.is_empty()
    }

    #[inline]
    fn should_update(&self) -> bool {
        !self.is_initialized() || self.last_sample_time.elapsed() > SAMPLE_EVERY
    }

    pub fn update(&mut self) {
        if !self.should_update() {
            return;
        }
        //eprintln!("refresh...");

        self.sys.refresh_cpu();
        self.last_sample_time = Instant::now();
        let now = Utc::now();
        let data = self.sys.processors().iter().map(|v| v.cpu_usage() as i32);

        //check if initialized
        if !self.is_initialized() {
            eprintln!("init...");
            let mut processors: Vec<_> = data
                .map(|percent| CpuUsageChart::new(vec![(now, percent)].into_iter()))
                .collect();
            self.processors.append(&mut processors);
        } else {
            //eprintln!("update...");
            for (percent, p) in data.zip(self.processors.iter_mut()) {
                p.push_data(now, percent);
            }
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        if !self.is_initialized() {
            iced::Text::new("Loading...")
                .horizontal_alignment(HorizontalAlignment::Center)
                .vertical_alignment(VerticalAlignment::Center)
                .into()
        } else {
            let mut scroll = Scrollable::new(&mut self.scroll)
                .width(Length::Fill)
                .height(Length::Fill);

            let chart_height = self.chart_height;
            let mut idx = 0;
            for chunk in self.processors.chunks_mut(self.items_per_row) {
                let mut row = Row::new()
                    .spacing(15)
                    .padding(20)
                    .width(Length::Fill)
                    .height(Length::Units(chart_height))
                    .align_items(Align::Center);
                for item in chunk {
                    row = row.push(item.view(idx));
                    idx += 1;
                }
                while idx % self.items_per_row != 0 {
                    row = row.push(Space::new(Length::Fill, Length::Fill));
                    idx += 1;
                }
                scroll = scroll.push(row);
            }

            scroll.into()
        }
    }
}

struct CpuUsageChart {
    cache: Cache,
    data_points: VecDeque<(DateTime<Utc>, i32)>,
    limit: Duration,
}

impl CpuUsageChart {
    fn new(data: impl Iterator<Item = (DateTime<Utc>, i32)>) -> Self {
        let data_points: VecDeque<_> = data.collect();
        Self {
            cache: Cache::new(),
            data_points,
            limit: Duration::from_secs(PLOT_SECONDS as u64),
        }
    }

    fn push_data(&mut self, time: DateTime<Utc>, value: i32) {
        let cur_ms = time.timestamp_millis();
        self.data_points.push_front((time, value));
        loop {
            if let Some((time, _)) = self.data_points.back() {
                let diff = Duration::from_millis((cur_ms - time.timestamp_millis()) as u64);
                if diff > self.limit {
                    self.data_points.pop_back();
                    continue;
                }
            }
            break;
        }
        self.cache.clear();
    }

    fn view(&mut self, idx: usize) -> Element<Message> {
        Container::new(
            Column::new()
                .width(Length::Fill)
                .height(Length::Fill)
                .spacing(5)
                .push(iced::Text::new(format!("Processor {}", idx)))
                .push(
                    ChartWidget::new(self).height(Length::Fill).resolve_font(
                        |_, style| match style {
                            plotters_backend::FontStyle::Bold => FONT_BOLD,
                            _ => FONT_REGULAR,
                        },
                    ),
                ),
        )
            .style(style::ChartContainer)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Align::Center)
            .align_y(Align::Center)
            .into()
    }
}

impl Chart<Message> for CpuUsageChart {
    // fn update(
    //     &mut self,
    //     event: Event,
    //     bounds: Rectangle,
    //     cursor: Cursor,
    // ) -> (event::Status, Option<Message>) {
    //     self.cache.clear();
    //     (event::Status::Ignored, None)
    // }

    #[inline]
    fn draw<F: Fn(&mut Frame)>(&self, bounds: Size, draw_fn: F) -> Geometry {
        self.cache.draw(bounds, draw_fn)
    }

    fn build_chart<DB: DrawingBackend>(&self, mut chart: ChartBuilder<DB>) {
        use plotters::{prelude::*, style::Color};

        const PLOT_LINE_COLOR: RGBColor = RGBColor(0, 175, 255);

        // Acquire time range
        let newest_time = self
            .data_points
            .front()
            .unwrap_or(&(
                chrono::DateTime::from_utc(
                    chrono::NaiveDateTime::from_timestamp(0, 0),
                    chrono::Utc,
                ),
                0,
            ))
            .0;
        let oldest_time = newest_time - chrono::Duration::seconds(PLOT_SECONDS as i64);
        let mut chart = chart
            .x_label_area_size(0)
            .y_label_area_size(28)
            .margin(20)
            .build_cartesian_2d(oldest_time..newest_time, 0..100)
            .expect("failed to build chart");

        chart
            .configure_mesh()
            .bold_line_style(&plotters::style::colors::WHITE.mix(0.1))
            .light_line_style(&plotters::style::colors::WHITE.mix(0.05))
            .axis_style(ShapeStyle::from(&plotters::style::colors::WHITE.mix(0.45)).stroke_width(1))
            .y_labels(10)
            .y_label_style(
                ("sans-serif", 15)
                    .into_font()
                    .color(&plotters::style::colors::WHITE.mix(0.65))
                    .transform(FontTransform::Rotate90),
            )
            .y_label_formatter(&|y| format!("{}%", y))
            .draw()
            .expect("failed to draw chart mesh");

        chart
            .draw_series(
                AreaSeries::new(
                    self.data_points.iter().map(|x| (x.0, x.1 as i32)),
                    0,
                    &PLOT_LINE_COLOR.mix(0.175),
                )
                    .border_style(ShapeStyle::from(&PLOT_LINE_COLOR).stroke_width(2)),
            )
            .expect("failed to draw chart data");
    }
}

mod style {
    use iced::Color;

    pub struct ChartContainer;
    impl iced::container::StyleSheet for ChartContainer {
        fn style(&self) -> iced::container::Style {
            iced::container::Style {
                background: Some(Color::BLACK.into()),
                text_color: Some(Color::WHITE),
                ..Default::default()
            }
        }
    }
}