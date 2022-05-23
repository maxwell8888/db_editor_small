#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]

use std::fs;
extern crate chrono;
use crate::charts;
use crate::some_styles;
use crate::utils;
use bytecheck::CheckBytes;
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
use rkyv;
use rkyv::Archive;
use rusqlite::{params, Connection, Result, ToSql};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::File;
use std::time::SystemTime;

// #[derive(
//     Debug,
//     Clone,
//     Default,
//     Deserialize,
//     Serialize,
//     rkyv::Deserialize,
//     rkyv::Serialize,
//     rkyv::Archive,
// )]
// // This will generate a PartialEq impl between our unarchived and archived types
// #[archive(compare(PartialEq))]
// // To use the safe API, you have to derive CheckBytes for the archived type
// #[archive_attr(derive(CheckBytes, Debug))]
#[derive(Clone, Default, Deserialize, Serialize)]
pub struct CratesIo {
    pub categories: Vec<CratesIOCategory>,
    pub crates: Vec<CratesIOCrate>,
    pub crates2: Vec<CratesIOCrate2>,
    #[serde(skip_serializing, skip_deserializing)]
    pub categories_widgets: Vec<CratesIOCategoryWidget>,
}

impl CratesIo {
    pub fn new() -> Self {
        println!("{:?} starting CratesIo::new()", Utc::now());
        let conn = Connection::open("cratesio.db").unwrap();
        println!("{:?} finished opening db connection", Utc::now());

        let mut stmt = conn.prepare("select * from categories;").unwrap();
        let category_iter = stmt
            .query_map([], |row| {
                Ok(CratesIOCategory {
                    id: row.get(4)?,
                    name: row.get(0)?,
                    description: row.get(3)?,
                    // button_state: button::State::new(),
                })
            })
            .unwrap();
        let mut category_vec = Vec::new();
        for category in category_iter {
            category_vec.push(category.unwrap());
        }

        let mut stmt = conn.prepare("select * from crates;").unwrap();
        let crate_iter = stmt
            .query_map([], |row| {
                Ok(CratesIOCrate {
                    id: row.get(5)?,
                    name: row.get(7)?,
                    description: row.get(1)?,
                    readme: row.get(8)?,
                })
            })
            .unwrap();
        let mut crate_vec = Vec::new();
        for my_crate in crate_iter {
            crate_vec.push(my_crate.unwrap());
        }

        let mut stmt = conn.prepare("select * from crates_categories;").unwrap();
        let crate_iter = stmt
            .query_map([], |row| {
                Ok(CratesIOCategoryCrateMap {
                    category_id: row.get(0)?,
                    crate_id: row.get(1)?,
                })
            })
            .unwrap();
        let mut category_crate_vec = Vec::new();
        for my_crate in crate_iter {
            category_crate_vec.push(my_crate.unwrap());
        }

        let mut stmt = conn.prepare("select * from versions;").unwrap();
        let crate_iter = stmt
            .query_map([], |row| {
                Ok(CratesIOVersion {
                    id: row.get(5)?,
                    crate_id: row.get(0)?,
                    version_number: row.get(7)?,
                })
            })
            .unwrap();
        let mut versions_vec = Vec::new();
        for my_crate in crate_iter {
            versions_vec.push(my_crate.unwrap());
        }

        let mut stmt = conn.prepare("select * from version_downloads;").unwrap();
        let crate_iter = stmt
            .query_map([], |row| {
                Ok(CratesIOVersionDownload {
                    date: row.get(0)?,
                    downloads: row.get(1)?,
                    version_id: row.get(2)?,
                })
            })
            .unwrap();
        let mut version_downloads_vec = Vec::new();
        for my_crate in crate_iter {
            version_downloads_vec.push(my_crate.unwrap());
        }
        println!("{:?} finished reading all db data", Utc::now());

        let mut version_downloads_vec2 = Vec::new();
        let csv_file_path = "data/cratesio/2021-12-31-020031/data_no_header/version_downloads.csv";
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(csv_file_path)
            .unwrap();
        let boundary_date = NaiveDate::from_ymd(2021, 12, 6);
        for result in rdr.records() {
            let record = result.unwrap();
            let date = NaiveDate::parse_from_str(&record[0], "%Y-%m-%d").unwrap();
            // println!("{:?} {:?} {}", date, boundary_date, date > boundary_date);
            if date < boundary_date {
                // if true {
                version_downloads_vec2.push(CratesIOVersionDownload {
                    date: date,
                    downloads: record[1].parse().unwrap(),
                    version_id: record[2].parse().unwrap(),
                });
            }
        }
        println!("{:?} finished reading csv data", Utc::now());
        println!("version_downloads_vec: {}", version_downloads_vec.len());
        println!("version_downloads_vec2: {}", version_downloads_vec2.len());

        version_downloads_vec.append(&mut version_downloads_vec2);
        println!("version_downloads_vec: {}", version_downloads_vec.len());

        // sort both vectors by version_id, then should be able to just use .nex() on versions_vec, and checking equivalence, rather than checking the whole thing??
        version_downloads_vec.sort_by(|a, b| a.date.cmp(&b.date));
        version_downloads_vec.sort_by(|a, b| a.version_id.cmp(&b.version_id));
        println!(
            "{:?} finished sorting version_downloads_vec data by date",
            Utc::now()
        );
        // version_downloads_vec.dedup();

        versions_vec.sort_by(|a, b| a.id.cmp(&b.id));
        println!(
            "{:?} finished sorting versions_vec data by date",
            Utc::now()
        );

        let mut versions_vec_iter = versions_vec.iter();
        let mut current_version = versions_vec_iter.next().unwrap();
        let mut version_downloads_vec_with_crate_id = version_downloads_vec
            .iter()
            .map(|x| {
                // find version in versions_vec
                if current_version.id != x.version_id {
                    current_version = versions_vec_iter.next().unwrap();
                }
                if current_version.id != x.version_id {
                    panic!(
                        "couldn't match versions {} {}",
                        current_version.id, x.version_id
                    );
                }
                (x, current_version)
            })
            .collect::<Vec<_>>();
        println!("{:?} finished joining downloads data", Utc::now());

        // sort downloads by date then crate_id
        version_downloads_vec_with_crate_id.sort_by(|a, b| a.0.date.cmp(&b.0.date));
        println!("{:?} finished sorting downloads data by date", Utc::now());
        version_downloads_vec_with_crate_id.sort_by(|a, b| a.1.crate_id.cmp(&b.1.crate_id));
        println!(
            "{:?} finished sorting downloads data by crate_id",
            Utc::now()
        );
        // let tar_downloads = version_downloads_vec_with_crate_id
        //     .iter()
        //     .filter(|x| x.1.crate_id == 6)
        //     .collect::<Vec<_>>()
        //     .len();
        // println!("tar_downloads: {}", tar_downloads);

        let mut version_downloads_sum_vec = version_downloads_vec_with_crate_id
            .group_by(|a, b| (a.1.crate_id == b.1.crate_id) && (a.0.date == b.0.date))
            .map(|group| {
                // crate_id, date, downloads, version_number
                let last_row = group.last().unwrap();
                (
                    last_row.1.crate_id,
                    last_row.0.date,
                    group.iter().map(|thing| thing.0.downloads).sum::<usize>(), // last_row.1.version_number,
                )
            })
            .collect::<Vec<_>>();
        println!("{:?} finished grouping downloads", Utc::now());
        // check version_downloads_sum_vec is sorted by crate id
        version_downloads_sum_vec.sort_by(|a, b| a.0.cmp(&b.0));

        // only has last 90 days since this is all the raw dump provides
        let tar_downloads2 = version_downloads_sum_vec
            .iter()
            .filter(|x| x.0 == 6)
            .collect::<Vec<_>>()
            .len();
        println!("tar_downloads2: {}", tar_downloads2);

        // daily downloads doesn't include zeroes so needs extending
        let mut version_downloads_sum_iter = version_downloads_sum_vec.iter();
        let mut version_downloads_sum_current = version_downloads_sum_iter.next().unwrap();

        // sort crates and categories_crates by crate id
        crate_vec.sort_by(|a, b| a.id.cmp(&b.id));
        category_crate_vec.sort_by(|a, b| a.crate_id.cmp(&b.crate_id));
        println!(
            "{:?} finished sorting crates and categories_crates",
            Utc::now()
        );
        let mut category_crate_iter = category_crate_vec.iter();
        let mut category_crate_current = category_crate_iter.next().unwrap();
        println!("{:?} finished making iter", Utc::now());
        println!(
            "{:?} first crate id is {}",
            Utc::now(),
            category_crate_current.crate_id
        );

        // to get version number
        // sort versions_vec by crate id (previously sorted by version id)
        versions_vec.sort_by(|a, b| a.crate_id.cmp(&b.crate_id));
        println!(
            "{:?} finished sorting versions_vec data by crate_id",
            Utc::now()
        );
        let mut versions_vec_iter = versions_vec.iter();
        let mut versions_vec_current = versions_vec_iter.next().unwrap();

        let crate_vec2 = crate_vec
            .iter()
            .map(|x| {
                let mut cat_ids = Vec::new();
                while x.id == category_crate_current.crate_id {
                    cat_ids.push(category_crate_current.category_id);
                    category_crate_current = match category_crate_iter.next() {
                        Some(val) => val,
                        None => break,
                    };
                }

                let mut version_number = "no version number found".to_string();
                while x.id == versions_vec_current.crate_id {
                    version_number = versions_vec_current.version_number.clone();
                    versions_vec_current = match versions_vec_iter.next() {
                        Some(val) => val,
                        None => break,
                    }
                }

                let mut daily_downloads = Vec::new();
                while version_downloads_sum_current.0 == x.id {
                    daily_downloads.push(CratesIODownloadCount {
                        date: version_downloads_sum_current.1,
                        count: version_downloads_sum_current.2,
                    });
                    version_downloads_sum_current = match version_downloads_sum_iter.next() {
                        Some(val) => val,
                        None => break,
                    }
                }

                daily_downloads.sort_by(|a, b| a.date.cmp(&b.date));
                let monthly_downloads = daily_downloads
                    .group_by(|a, b| {
                        NaiveDate::from_ymd(a.date.year(), a.date.month(), 1)
                            == NaiveDate::from_ymd(b.date.year(), b.date.month(), 1)
                    })
                    .map(|group| CratesIODownloadCount {
                        date: NaiveDate::from_ymd(group[0].date.year(), group[0].date.month(), 1),
                        count: group.iter().map(|x| x.count).sum(),
                    })
                    .collect::<Vec<_>>();

                //  might be better to store IsoWeek rather than a NaiveDate???
                let weekly_downloads = daily_downloads
                    .group_by(|a, b| a.date.iso_week() == b.date.iso_week())
                    .map(|group| CratesIODownloadCount {
                        date: NaiveDate::from_isoywd(
                            group[0].date.year(),
                            group[0].date.iso_week().week(),
                            Weekday::Mon,
                        ),
                        count: group.iter().map(|x| x.count).sum(),
                    })
                    .collect::<Vec<_>>();

                let linechart_data = charts::LineChartData {
                    borders: charts::Borders::trbl(10., 10., 20., 40.),
                    dates: weekly_downloads.iter().map(|x| x.date).collect::<Vec<_>>(),
                    columns: vec![charts::ChartColumn {
                        name: Some("poo".to_string()),
                        data: weekly_downloads
                            .iter()
                            .map(|x| Some(x.count as i64))
                            .collect::<Vec<_>>(),
                    }],
                    xinc: charts::Xinc::Week1,
                    legend: false,
                    typex: charts::ChartType::LineChart,
                };
                CratesIOCrate2 {
                    id: x.id,
                    name: x.name.clone(),
                    description: x.description.clone(),
                    readme: x.readme.clone(),
                    categories: category_vec
                        .iter()
                        .filter(|cat| cat_ids.contains(&cat.id))
                        .map(|x| x.clone())
                        .collect::<Vec<_>>(),
                    version_number,
                    daily_downloads,
                    weekly_downloads,
                    monthly_downloads,
                    linechart_data,
                }
            })
            .collect::<Vec<_>>();
        println!("{:?} finished making crate2", Utc::now());

        let categories_widgets = category_vec
            .iter()
            .map(|cat| CratesIOCategoryWidget {
                cratesio_category: cat.clone(),
            })
            .collect::<Vec<_>>();

        CratesIo {
            categories: category_vec,
            crates: crate_vec,
            crates2: crate_vec2,
            categories_widgets: categories_widgets,
        }
    }

    pub fn from_json(path: &str) -> Self {
        println!("{:?} read from disk", Utc::now());
        let read_string = fs::read_to_string(path).unwrap();

        println!("{:?} deserialize", Utc::now());
        let deserialized: CratesIo = serde_json::from_str(&read_string).unwrap();
        println!("{:?} finished", Utc::now());
        deserialized
    }
    pub fn from_bincode(path: &str) -> Self {
        println!("{:?} bincode: read from disk", Utc::now());
        let read_bincode_string = fs::read(path).unwrap();

        println!("{:?} bincode: deserialize", Utc::now());
        let de_bincode: CratesIo = bincode::deserialize(&read_bincode_string[..]).unwrap();
        println!("{:?} finished", Utc::now());
        de_bincode
    }

    // get crates with given category
    pub fn get_crate_with_category(&self, category_id: usize) -> Vec<CratesIOCrate> {
        let conn = Connection::open("cratesio.db").unwrap();
        let mut stmt = conn
            .prepare(
                format!(
                    "SELECT * FROM crates
            INNER JOIN crates_categories on crates.id = crates_categories.crate_id
            INNER JOIN categories on crates_categories.category_id = categories.id
            WHERE categories.id = {};",
                    category_id
                )
                .as_str(),
            )
            .unwrap();
        let crate_iter = stmt
            .query_map([], |row| {
                Ok(CratesIOCrate {
                    id: row.get(5)?,
                    name: row.get(7)?,
                    description: row.get(1)?,
                    readme: row.get(8)?,
                })
            })
            .unwrap();
        let mut crate_vec = Vec::new();
        for my_crate in crate_iter {
            crate_vec.push(my_crate.unwrap());
        }
        crate_vec
    }

    // get crates with given categories (will fail if Vec.len() == 0 !!!!)
    pub fn get_crate_with_categories(&self, category_ids: Vec<usize>) -> Vec<CratesIOCrate> {
        let conn = Connection::open("cratesio.db").unwrap();

        // OR:
        // WITH cats as
        // (SELECT * FROM crates_categories
        // INNER JOIN categories on crates_categories.category_id = categories.id
        // WHERE crates_categories.category_id = 309 OR crates_categories.category_id = 294
        // )
        // SELECT crates.id, crates.name, crates.description, cats.id as category_id,
        // cats.category as categories_name, cats.description as categories_descriptions FROM crates
        // INNER JOIN cats ON crates.id = cats.crate_id;
        // AND:
        // Todo

        let sql = format!(
            "SELECT * FROM crates
    INNER JOIN crates_categories on crates.id = crates_categories.crate_id
    INNER JOIN categories on crates_categories.category_id = categories.id
    WHERE {};",
            category_ids
                .iter()
                .map(|x| format!("categories.id = {}", x))
                .collect::<Vec<_>>()
                .join(" AND ")
        );
        println!("sql: {}", sql);
        let mut stmt = conn.prepare(sql.as_str()).unwrap();
        let crate_iter = stmt
            .query_map([], |row| {
                Ok(CratesIOCrate {
                    id: row.get(5)?,
                    name: row.get(7)?,
                    description: row.get(1)?,
                    readme: row.get(8)?,
                })
            })
            .unwrap();
        let mut crate_vec = Vec::new();
        for my_crate in crate_iter {
            crate_vec.push(my_crate.unwrap());
        }
        crate_vec
    }
}

#[derive(Clone)]
pub enum CrateMessage {
    // Hi(usize),
    HiFromLineChart(charts::Whatever),
    AddCrateToComparison(usize),
    CopyTomlToClipboard(String),
    ShowReadme(usize),
}

#[derive(Debug, Clone, Default)]
pub struct CratesIOCategoryCrateMap {
    pub crate_id: usize,
    pub category_id: usize,
}

// use semver crate/type for version_number https://docs.rs/semver/latest/semver/
#[derive(Debug, Clone, Default)]
pub struct CratesIOVersion {
    pub id: usize,
    pub crate_id: usize,
    pub version_number: String,
}

// #[derive(Debug, Clone, Deserialize)]
#[derive(Debug, Clone)]
pub struct CratesIOVersionDownload {
    pub date: NaiveDate,
    pub downloads: usize,
    pub version_id: usize,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CratesIOCategory {
    pub id: usize,
    pub name: String,
    pub description: String,
}

#[derive(Clone)]
pub enum CategoryMessage {
    Hi(usize),
}
#[derive(Debug, Clone, Default)]
pub struct CratesIOCategoryWidget {
    pub cratesio_category: CratesIOCategory,
}
impl CratesIOCategoryWidget {
    pub fn view(&self) -> Element<CategoryMessage> {
        button(Column::with_children(vec![
            text(self.cratesio_category.name.clone())
                .width(Length::Units(400))
                .into(),
            text(self.cratesio_category.description.clone())
                .width(Length::Units(800))
                .into(),
        ]))
        .on_press(CategoryMessage::Hi(self.cratesio_category.id))
        .into()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CratesIOCrate {
    pub id: usize,
    pub name: String,
    pub description: String,
    pub readme: String,
    // pub button_state: button::State,
}
impl CratesIOCrate {
    pub fn view(&self) -> Element<CrateMessage> {
        Row::with_children(vec![
            Text::new(self.name.clone())
                .width(Length::Units(400))
                .into(),
            Text::new(self.description.clone())
                .width(Length::Units(800))
                .into(),
        ])
        .into()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CratesIODownloadCount {
    pub date: NaiveDate,
    pub count: usize,
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct CratesIOCrate2 {
    pub id: usize,
    pub name: String,
    pub description: String,
    pub readme: String,
    pub categories: Vec<CratesIOCategory>,
    pub version_number: String,
    pub daily_downloads: Vec<CratesIODownloadCount>,
    pub weekly_downloads: Vec<CratesIODownloadCount>,
    pub monthly_downloads: Vec<CratesIODownloadCount>,
    pub linechart_data: charts::LineChartData,
}

impl CratesIOCrate2 {
    pub fn view(&self) -> Element<CrateMessage> {
        let linechart: Element<_> = Canvas::new(&self.linechart_data)
            .width(Length::Fill)
            .height(Length::Units(200))
            .into();
        container(
            Column::with_children(vec![
                Row::with_children(vec![
                    button(
                        text(format!(
                            "{} = \"{}\"",
                            self.name.clone(),
                            self.version_number
                        ))
                        .size(20)
                        .width(Length::Units(200)),
                        // daily downloads doesn't include zeroes so needs extending
                    )
                    .on_press(CrateMessage::CopyTomlToClipboard(format!(
                        "{} = \"{}\"",
                        self.name.clone(),
                        self.version_number
                    )))
                    .into(),
                    Text::new(format!(
                        "^{}",
                        utils::pretty_number(self.monthly_downloads.last().unwrap().count)
                    ))
                    .width(Length::Units(100))
                    .into(),
                    Text::new(format!("v{}", self.version_number))
                        .width(Length::Units(100))
                        .into(),
                    button("Add")
                        .on_press(CrateMessage::AddCrateToComparison(self.id))
                        .into(),
                    button("Show README")
                        .on_press(CrateMessage::ShowReadme(self.id))
                        .into(),
                ])
                // .spacing(20)
                .into(),
                self.categories
                    .iter()
                    .fold(row(), |acc, x| acc.push(Text::new(x.name.clone())))
                    .into(),
                Row::with_children(vec![
                    text(self.description.clone())
                        .width(Length::Units(400))
                        .into(),
                    linechart.map(CrateMessage::HiFromLineChart),
                ])
                .into(),
            ]),
            // .padding(10)
            // .spacing(10),
        )
        .padding(10)
        .style(some_styles::BlackBorder)
        .into()
    }
}
