extern crate chrono;
// can use either crate:: or super::
// use crate::colors;
use super::colors;
use crate::some_styles;
use chrono::prelude::*;
use chrono::{Datelike, Duration, Timelike, Utc};
use iced::alignment::{Alignment, Horizontal, Vertical};
use iced::pure::{
    button, column, container, pick_list, row, scrollable, slider, text, text_input,
    widget::{
        button,
        canvas::event::{self, Event},
        canvas::{
            self, Cache, Canvas, Cursor, Fill, Frame, Geometry, LineCap, Path, Program, Stroke,
        },
        container, Button, Column, Row, Text, TextInput,
    },
    Element, Sandbox,
};
use iced::tooltip::{self, Tooltip};
// use iced::widget::{button, container, pick_list, scrollable, slider, text_input, Scrollable};
// use iced::Application;
use iced::pure::Application;
use iced::Font;
use iced::{
    clipboard, window, Background, Checkbox, Color, Command, Container, Length, PickList, Point,
    Radio, Rectangle, Settings, Size, Slider, Space, Vector,
};
// VerticalAlignment, Align, HorizontalAlignment,
use iced::{executor, mouse};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy)]
pub struct Whatever {}

#[derive(Default, Clone, Deserialize, Serialize)]
pub struct Borders {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}
impl Borders {
    pub fn trbl(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Borders {
            top: top,
            right: right,
            bottom: bottom,
            left: left,
        }
    }
    pub fn all(border: f32) -> Self {
        Self {
            top: border,
            right: border,
            bottom: border,
            left: border,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub enum Xinc {
    // Day1,
    Week1,
    Month1,
    Month3,
    // Month6,
    // Year,
}
impl Default for Xinc {
    fn default() -> Self {
        Self::Month1
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub enum ChartType {
    LineChart,
    StackedBarChart,
}
impl Default for ChartType {
    fn default() -> Self {
        Self::LineChart
    }
}

#[derive(Default, Clone, Deserialize, Serialize)]
pub struct LineChartData {
    // pub struct LineChart<'a> {
    // data: &'a ChartData,
    pub borders: Borders,
    pub dates: Vec<NaiveDate>,
    pub columns: Vec<ChartColumn>,
    pub xinc: Xinc,
    pub legend: bool,
    pub typex: ChartType,
}

#[derive(Default)]
pub struct ChartData {
    pub dates: Vec<NaiveDate>,
    pub columns: Vec<ChartColumn>,
}

#[derive(Default, Clone, Deserialize, Serialize)]
pub struct ChartColumn {
    pub name: Option<String>,
    // pub color: Option<Color>,
    // pub position: Option<usize>,
    pub data: Vec<Option<i64>>,
}
pub enum Wtf {
    Hi,
}
impl Default for Wtf {
    fn default() -> Self {
        Self::Hi
    }
}

// why does this need to be impl on a reference? because Canvas::new() wants to take ownership and we don't want to/can't move the data, and don't want to clone it if it is large
impl canvas::Program<Whatever> for &LineChartData {
    type State = Wtf;

    fn draw(&self, state: &Self::State, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        // println!("general line chart");
        // could add 'totals' as an option to add total line, but for now leave it for user to add to their data

        let mut frame = Frame::new(bounds.size());

        let chart_area_width = frame.width() - self.borders.left - self.borders.right;
        let chart_area_height = frame.height() - self.borders.top - self.borders.bottom;

        let line_thickness = 3.0;
        let color_strokes = colors::LIST
            .iter()
            .map(|color| Stroke {
                width: line_thickness,
                color: *color,
                line_cap: LineCap::Round,
                ..Stroke::default()
            })
            .collect::<Vec<_>>();

        let gridline_stroke = Stroke {
            width: 2.0,
            color: colors::LIGHT_MED_GREY,
            line_cap: LineCap::Butt,
            ..Stroke::default()
        };

        // ymin
        // this should be done outside of the line chart to give users the opportunity to reuse values from other datasets etc
        // doesn't yet handle empty data
        let y_min = self
            .columns
            .iter()
            .map(|col| {
                col.data
                    .iter()
                    .filter(|x| x.is_some())
                    .map(|x| x.unwrap())
                    .min()
                    .unwrap()
            })
            .min()
            .unwrap();

        let y_max = self
            .columns
            .iter()
            .map(|col| {
                col.data
                    .iter()
                    .filter(|x| x.is_some())
                    .map(|x| x.unwrap())
                    .max()
                    .unwrap()
            })
            .max()
            .unwrap();

        // calculate standardised ymin and ymax, and inc
        let y_range = y_max - y_min;
        // need an algorithm for increasing incs like 1 -> 2 -> 5 -> 10 -> 20 -> 50 -> 100 etc
        // then find the inc that produces number of gridlines closest to our specified optimal e.g. 5. So need to find the incs that produce >= and <= 5 gridlines and choose the closest

        // won't handle ranges under 5, for now
        let incs = vec![
            1,
            1,
            2,
            5,
            10,
            20,
            50,
            100,
            200,
            500,
            1000,
            2000,
            5000,
            10000,
            20000,
            50000,
            100000,
            200000,
            500000,
            1000000,
            2000000,
            5000000,
            10000000,
            20000000,
            50000000,
            100000000,
            200000000,
            500000000,
            1000000000,
            2000000000,
            5000000000,
            10000000000,
            20000000000,
            50000000000,
            100000000000,
            200000000000,
            500000000000,
            1000000000000,
        ];

        let optimal_number_incs = 5;
        let mut i = 1;
        while y_range as f32 / incs[i] as f32 >= optimal_number_incs as f32 {
            i = i + 1;
        }
        let top_inc = incs[i - 1]; // 2
        let bottom_inc = incs[i]; // 5

        let y_max_std = |y_inc| {
            if y_max > 0 {
                (y_max / y_inc + 1) * y_inc
            } else {
                (y_max / y_inc) * y_inc
            }
        };
        let y_min_std = |y_inc| {
            if y_min < 0 {
                (y_min / y_inc - 1) * y_inc
            } else {
                (y_min / y_inc) * y_inc
            }
        };

        let n_y_incs = |y_inc| (y_max_std(y_inc) - y_min_std(y_inc)) / y_inc;
        // top inc
        let y_inc = if n_y_incs(top_inc) - optimal_number_incs
            < optimal_number_incs - n_y_incs(bottom_inc)
        {
            top_inc
        } else {
            bottom_inc
        };

        let y_max_std = y_max_std(y_inc);
        let y_min_std = y_min_std(y_inc);

        // let (y_max_std, y_min_std, y_inc) = (3000000, -1000000, 500000);
        // let n_y_incs = (y_max_std - y_min_std) / y_inc;
        let n_y_incs = n_y_incs(y_inc);
        let y_chart_inc = chart_area_height / (n_y_incs as f32);

        // x increments
        let n_x_incs = self.dates.len();

        // bar chart x incs need +1 length to line charts because can't display a bar at both boundaries of the chart area
        let x_chart_inc = match self.typex {
            ChartType::LineChart => {
                if n_x_incs == 1 {
                    chart_area_width / 2.
                } else {
                    chart_area_width / (n_x_incs as f32 - 1.0)
                }
            }
            ChartType::StackedBarChart => chart_area_width / (n_x_incs as f32),
        };

        // can't just pass in data which is a subset of total time series because we won't know at which point the data starts (unless we provided a starting point, or it has xaxis keys)
        let make_data_line_path = |data: &Vec<Option<i64>>| {
            Path::new(|builder| {
                let mut started = false;

                // iterate over length of chart
                for (i, y_val) in data.iter().enumerate() {
                    // convert data point value to chart position value
                    let y_val_chart = match y_val {
                        Some(val) => Some(
                            self.borders.top
                                + chart_area_height * (y_max_std as f32 - *val as f32)
                                    / (y_max_std as f32 - y_min_std as f32),
                        ),
                        None => None,
                    };

                    // draw path
                    if y_val_chart.is_some() && !started {
                        started = true;
                        builder.move_to(Point::new(
                            self.borders.left + i as f32 * x_chart_inc,
                            y_val_chart.unwrap(),
                        ));
                    } else if y_val_chart.is_some() && started {
                        builder.line_to(Point::new(
                            self.borders.left + i as f32 * x_chart_inc,
                            y_val_chart.unwrap(),
                        ));
                    }
                }
            })
        };

        let paths = self
            .columns
            .iter()
            .map(|col| make_data_line_path(&col.data));

        // yaxis labels
        for i in 0..=n_y_incs {
            frame.fill_text(iced::widget::canvas::Text {
                content: (y_max_std as i64 - i * y_inc).to_string(),
                position: Point::new(
                    self.borders.left,
                    self.borders.top + (chart_area_height * i as f32 / n_y_incs as f32),
                ),
                vertical_alignment: Vertical::Center,
                horizontal_alignment: Horizontal::Right,
                ..Default::default()
            });
        }

        // xaxis labels
        // need to do analysis to work out how many labels to have - could just let user specific - day, month, quater, 6month, 1 year, 2 year, 5 year, etc
        // if self.dates is weekly, then probably want to count days so we can put dates in places other than Mondays, e.g. start of the month etc

        // this needs to extend past self.dates.last() for 7 days for Week1, 1 month for Month1, etc (though Week1, Month1 are just the chosen display, we need to know the actual increment - could just calculate it as self.dates[1] - self.dates[0]??)
        // let mut start = NaiveDate::from_ymd(2021, 12, 30);
        // let end = NaiveDate::(2022, 01, 2);
        // let dates = Vec::new();
        // while start < end {
        //     dates.push(start.add_days(1));
        // }

        let dates = if matches!(self.xinc, Xinc::Week1) {
            let mut new = Vec::new();
            let mut new_date = self.dates[0].clone();
            while new_date < *self.dates.last().unwrap() {
                new.push(new_date);
                new_date = new_date + Duration::days(1);
            }
            new
        } else {
            self.dates.clone()
        };
        let n_x_incs = dates.len();
        let x_chart_inc = match self.typex {
            ChartType::LineChart => chart_area_width / (n_x_incs as f32 - 1.0),
            ChartType::StackedBarChart => chart_area_width / (n_x_incs as f32),
        };
        for (i, date) in dates.iter().enumerate() {
            let position = Point::new(
                self.borders.left + (i as f32 * x_chart_inc),
                self.borders.top + chart_area_height,
            );
            let mut fill_text = |format: &str| {
                frame.fill_text(iced::widget::canvas::Text {
                    content: date.format(format).to_string(),
                    position,
                    horizontal_alignment: Horizontal::Center,
                    ..Default::default()
                });
            };
            match self.xinc {
                Xinc::Week1 => {
                    if date.day() == 1 {
                        if date.month() == 1 {
                            fill_text("%Y");
                        } else {
                            fill_text("%b");
                        }
                    }
                }
                Xinc::Month1 => {
                    if date.day() == 1 {
                        if date.month() == 1 {
                            fill_text("%Y");
                        } else {
                            fill_text("%b");
                        }
                    }
                }
                Xinc::Month3 => {
                    if date.day() == 1 {
                        if date.month() == 1 {
                            fill_text("%Y");

                        // Jan Apr July Oct
                        } else if [4, 7, 10].contains(&date.month()) {
                            fill_text("%b");
                        }
                    }
                }
            }
        }

        // let colors_list = vec![colors::RED, colors::GREEN, colors::BLUE, colors::PURPLE];
        let colors_list = colors::LIST;

        // legend
        if self.legend {
            for (i, category) in self.columns.iter().map(|col| col.name.clone()).enumerate() {
                frame.fill_rectangle(
                    Point::new(
                        self.borders.left + chart_area_width + 30.,
                        self.borders.top + 50. + i as f32 * 50.,
                    ),
                    Size::new(10., 10.),
                    colors_list[i % colors_list.len()],
                );
                frame.fill_text(iced::canvas::Text {
                    content: match category {
                        Some(name) => name,
                        None => format!("Column {}", i + 1),
                    },
                    position: Point::new(
                        self.borders.left + chart_area_width + 50.,
                        self.borders.top + 50. + i as f32 * 50.,
                    ),
                    vertical_alignment: Vertical::Center,
                    horizontal_alignment: Horizontal::Left,
                    ..Default::default()
                });
            }
        }

        // chart area
        frame.fill_rectangle(
            Point::new(self.borders.left, self.borders.top),
            Size::new(chart_area_width, chart_area_height),
            colors::LIGHT_GREY,
        );

        // gridlines
        for i in 1..n_y_incs {
            let i2 = i as f32;
            let gridline = Path::line(
                Point::new(self.borders.left, self.borders.top + y_chart_inc * i2),
                Point::new(
                    self.borders.left + chart_area_width,
                    self.borders.top + y_chart_inc * i2,
                ),
            );
            frame.stroke(&gridline, gridline_stroke);
        }

        // write data lines
        match self.typex {
            ChartType::LineChart => {
                paths
                    .enumerate()
                    .for_each(|(i, path)| frame.stroke(&path, color_strokes[i]));
            }
            ChartType::StackedBarChart => {
                // bars
                // this will break if someone tries to plot no columns
                let mut pos_cum = self.columns[0].data.iter().map(|x| 0).collect::<Vec<_>>();
                let mut neg_cum = pos_cum.clone();
                for (j, column_data) in self.columns.iter().enumerate() {
                    for (i, y_val) in column_data.data.iter().enumerate() {
                        // update cummulative
                        if y_val.is_some() {
                            if y_val.unwrap() > 0 {
                                pos_cum[i] = pos_cum[i] + y_val.unwrap()
                            } else {
                                neg_cum[i] = neg_cum[i] + y_val.unwrap()
                            }
                        }
                        // convert data point value to chart position value
                        let y_val_chart_cum = match y_val {
                            Some(val) => Some(
                                self.borders.top
                                    + chart_area_height
                                        * (y_max_std as f32
                                            - (if *val > 0 { pos_cum[i] } else { neg_cum[i] })
                                                as f32)
                                        / (y_max_std as f32 - y_min_std as f32),
                            ),
                            None => None,
                        };
                        let y_val_chart = match y_val {
                            Some(val) => Some(
                                self.borders.top
                                    + chart_area_height * (y_max_std as f32 - *val as f32)
                                        / (y_max_std as f32 - y_min_std as f32),
                            ),
                            None => None,
                        };

                        // draw bar
                        if y_val_chart_cum.is_some() {
                            let bar_height = self.borders.top
                                + chart_area_height * (y_max_std as f32)
                                    / (y_max_std as f32 - y_min_std as f32)
                                - y_val_chart.unwrap();

                            // staggered
                            // frame.fill_rectangle(
                            //     Point::new(
                            //         border_left + (j as f32 * x_chart_inc / 4.) + i as f32 * x_chart_inc,
                            //         y_val_chart_cum.unwrap(),
                            //     ),
                            //     Size::new(x_chart_inc / 4., bar_height),
                            //     colors_list[j],
                            // );

                            frame.fill_rectangle(
                                Point::new(
                                    self.borders.left + i as f32 * x_chart_inc,
                                    y_val_chart_cum.unwrap(),
                                ),
                                Size::new(x_chart_inc * 0.9, bar_height),
                                colors_list[j % colors_list.len()],
                            );
                        }
                    }
                }
            }
        }

        vec![frame.into_geometry()]
    }
}
