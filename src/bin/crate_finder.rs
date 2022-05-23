#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use chrono::prelude::*;
use db_editor_small::charts;
use db_editor_small::colors;
use db_editor_small::crate_finder;
use db_editor_small::crate_finder::CrateMessage;
use db_editor_small::db_editor;
use db_editor_small::mytextbox;
use db_editor_small::personal_finance;
use db_editor_small::personal_finance::*;
use db_editor_small::some_styles;
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
use open;
use pulldown_cmark::{html, Options, Parser};
use std::borrow::BorrowMut;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::mem;

use crate::crate_finder::CratesIOCrate2;
pub fn main() -> iced::Result {
    Table::run(Settings::default())
}

// #[derive(Default)]
pub struct Table {
    crates_daaata: crate_finder::CratesIo,
    cratesio_filtered_and_sorted_crates: Vec<crate_finder::CratesIOCrate2>,
    cratesio_comparison_crates: Vec<usize>,
    current_crate_page: Vec<crate_finder::CratesIOCrate>,
    cratesio_show_category_list: bool,
    cratesio_show_comparison: bool,
    cratesio_category_filters: Vec<usize>,

    cratesio_downloads_filter_value: usize,

    cratesio_keyword_filter_value: String,

    cratesio_chart_comp_data: charts::LineChartData,

    cratesio_show_readme: Option<usize>,

    cratesio_links: Vec<IcedLink>,
}

#[derive(Debug, Clone)]
pub enum Message {
    DoNothing,
    CategorySelected(usize),
    ShowCategoryPicker(bool),
    ShowTrends(bool),
    DownloadFilterChanged(String),
    KeywordFilterChanged(String),
    AddCrateToComparison(usize),
    CopyToClipboard(String),
    ShowCrateReadme(usize),
    OpenPathOrUrl(String),
}

// impl Sandbox for Table {
impl Application for Table {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();
    // type Message = Message;

    fn new(_flags: ()) -> (Table, Command<Message>) {
        // read crates data
        // let crate_data = cratesio::CratesIo::new();
        // let path = "data/cratesio/preprocessed.json";
        // let crate_data = cratesio::CratesIo::from_json(path);
        let path = "data/cratesio/preprocessed.bincode";
        let crate_data = crate_finder::CratesIo::from_bincode(path);

        let mut poo = Table {
            crates_daaata: crate_data,
            cratesio_filtered_and_sorted_crates: Vec::default(),
            cratesio_comparison_crates: Vec::default(),
            current_crate_page: Vec::default(),
            cratesio_show_category_list: bool::default(),
            cratesio_show_comparison: bool::default(),
            cratesio_category_filters: Vec::default(),

            cratesio_downloads_filter_value: usize::default(),

            cratesio_keyword_filter_value: String::default(),

            cratesio_chart_comp_data: charts::LineChartData::default(),

            cratesio_show_readme: Option::default(),

            cratesio_links: Vec::default(),
        };

        let filter_and_sort_crates = || {
            println!("{:?} filter crates", Utc::now());
            let la_cat_filter = |my_crate: &CratesIOCrate2| {
                let cat_ids = my_crate.categories.iter().map(|x| x.id).collect::<Vec<_>>();
                poo.cratesio_category_filters
                    .iter()
                    .all(|filt| cat_ids.contains(filt))
            };
            if poo.cratesio_category_filters.len() > 0 {
                // crate_filters.push(Box::new(la_cat_filter));
            }

            let la_download_filter = |my_crate: &CratesIOCrate2| {
                my_crate.monthly_downloads.last().unwrap().count
                    > poo.cratesio_downloads_filter_value
            };
            if poo.cratesio_downloads_filter_value > 0 {
                // crate_filters.push(Box::new(la_download_filter));
            }

            let la_keyword_filter = |my_crate: &CratesIOCrate2| {
                let mut text = my_crate.name.clone();
                text.push_str(&my_crate.description);
                text.contains(&poo.cratesio_keyword_filter_value)
            };

            // --------------------------------------------
            // crates list
            // --------------------------------------------
            // doing lots of work in .update() because don't want to have to do expensive work on app load if the page won't be used? better than doing it every time we nav to page? Could always check and only do if it hasn't been done already?

            let mut crate_list = poo
                .crates_daaata
                .crates2
                .iter()
                .filter(|my_crate| {
                    [
                        if poo.cratesio_category_filters.len() > 0 {
                            la_cat_filter(*my_crate)
                        } else {
                            true
                        },
                        if poo.cratesio_downloads_filter_value > 0 {
                            la_download_filter(*my_crate)
                        } else {
                            true
                        },
                        if poo.cratesio_keyword_filter_value.len() > 0 {
                            la_keyword_filter(*my_crate)
                        } else {
                            true
                        },
                    ]
                    .iter()
                    .all(|x| *x)
                })
                .map(|x| x.clone())
                .collect::<Vec<_>>();

            println!("{:?} sort by downloads", Utc::now());
            crate_list.sort_by(|a, b| {
                a.monthly_downloads
                    .last()
                    .unwrap()
                    .count
                    .cmp(&b.monthly_downloads.last().unwrap().count)
            });
            println!("{:?} sort by keyword", Utc::now());
            if poo.cratesio_keyword_filter_value.len() > 0 {
                // want to preserve the download count sort order for case with equal relevance - not sure if below does it - so I will switch to sorting both THEN reversing
                crate_list.sort_by_key(|my_crate| {
                    // can't just pick last month, need last full month. this info needs to calculated from daily downloads
                    let download_multiplier = (my_crate
                        .daily_downloads
                        .iter()
                        .rev()
                        .take(30)
                        .map(|x| x.count)
                        .sum::<usize>()
                        + 5000)
                        / 5000;
                    if my_crate.name == poo.cratesio_keyword_filter_value {
                        4 * download_multiplier
                    } else if my_crate.name.contains(&poo.cratesio_keyword_filter_value) {
                        3 * download_multiplier
                    } else {
                        let text = my_crate.description.clone();
                        // this is too slow - need to make word freq list??
                        // even just using match on the descriptions is quite slow
                        // text.push_str(&my_crate.readme);
                        text.matches(&poo.cratesio_keyword_filter_value).count()
                            * download_multiplier
                    }
                });
            }
            println!("{:?} reverse list", Utc::now());
            crate_list.reverse();
            crate_list
        };
        poo.cratesio_filtered_and_sorted_crates = filter_and_sort_crates();

        (poo, Command::none())
    }

    fn title(&self) -> String {
        String::from("csv editor")
    }

    fn update(&mut self, event: Message) -> Command<Self::Message> {
        match event {
            Message::DoNothing => Command::none(),

            Message::CategorySelected(id) => {
                self.cratesio_category_filters.push(id);
                self.current_crate_page = self
                    .crates_daaata
                    .get_crate_with_categories(self.cratesio_category_filters.clone());
                self.cratesio_show_category_list = false;
                Command::none()
            }
            Message::ShowCategoryPicker(choice) => {
                self.cratesio_show_category_list = choice;
                Command::none()
            }
            Message::ShowTrends(choice) => {
                self.cratesio_show_comparison = choice;
                let selected_crates = self
                    .crates_daaata
                    .crates2
                    .iter()
                    .filter(|x| self.cratesio_comparison_crates.contains(&x.id))
                    .collect::<Vec<_>>();
                self.cratesio_chart_comp_data = charts::LineChartData {
                    borders: charts::Borders::trbl(10., 320., 50., 50.),
                    dates: selected_crates[0]
                        .monthly_downloads
                        .iter()
                        .map(|x| x.date)
                        .collect::<Vec<_>>(),
                    columns: selected_crates
                        .iter()
                        .map(|x| charts::ChartColumn {
                            name: Some(x.name.clone()),
                            data: x
                                .monthly_downloads
                                .iter()
                                .map(|x| Some(x.count as i64))
                                .collect::<Vec<_>>(),
                        })
                        .collect::<Vec<_>>(),
                    xinc: charts::Xinc::Month1,
                    legend: true,
                    typex: charts::ChartType::LineChart,
                };
                Command::none()
            }
            Message::DownloadFilterChanged(val) => {
                self.cratesio_downloads_filter_value = val.parse::<usize>().unwrap();
                Command::none()
            }
            Message::KeywordFilterChanged(val) => {
                self.cratesio_keyword_filter_value = val.clone();
                Command::none()
            }
            Message::AddCrateToComparison(id) => {
                self.cratesio_comparison_crates.push(id);
                Command::none()
            }
            Message::CopyToClipboard(val) => clipboard::write(val.clone()),
            Message::ShowCrateReadme(id) => {
                self.cratesio_show_readme = Some(id);
                Command::none()
            }
            Message::OpenPathOrUrl(destination) => {
                let result = open::that(destination);
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let config_data = some_styles::ConfigData {
            col_width: 100,
            row_height: 35,
        };

        // cratesio
        // --------------------------------------------
        // categories
        // --------------------------------------------

        // let mut crate_filters: Vec<Box<CrateFilterType>> = Vec::new();

        let content: Element<_> = column()
            .spacing(20)
            .padding(20)
            .push({
                println!("{:?} filter_and_sort_crates", Utc::now());

                let crates_list_len = self.cratesio_filtered_and_sorted_crates.len();

                println!("{:?} make crate widgets", Utc::now());
                let crates_list_column = self
                    .cratesio_filtered_and_sorted_crates
                    .iter()
                    .take(50)
                    .enumerate()
                    .fold(Column::new(), |acc, (i, x)| {
                        // this is the list of crates, so surely where we should be forwarding the add to comp event
                        acc.push(x.view().map(move |message| match message {
                            CrateMessage::AddCrateToComparison(id) => {
                                Message::AddCrateToComparison(id)
                            }
                            CrateMessage::HiFromLineChart(thing) => Message::DoNothing,
                            CrateMessage::CopyTomlToClipboard(val) => Message::CopyToClipboard(val),
                            CrateMessage::ShowReadme(id) => Message::ShowCrateReadme(id),
                        }))
                    });

                let crate_list_content = container(Column::with_children(vec![
                    text(format!("results: {}", crates_list_len)).into(),
                    // a scrollbar or increment buttons would be safest, but for now just do text input
                    Row::with_children(vec![
                        Text::new("filter by downloads >").into(),
                        text_input(
                            "type a number",
                            &self.cratesio_downloads_filter_value.to_string(),
                            Message::DownloadFilterChanged,
                        )
                        .into(),
                    ])
                    .into(),
                    Row::with_children(vec![
                        text("filter by keyword").into(),
                        text_input(
                            "type keyword",
                            &self.cratesio_keyword_filter_value.to_string(),
                            Message::KeywordFilterChanged,
                        )
                        .into(),
                    ])
                    .into(),
                    button("Add category filter")
                        .on_press(Message::ShowCategoryPicker(true))
                        .into(),
                    button("Show trends.io")
                        .on_press(Message::ShowTrends(true))
                        .into(),
                    self.crates_daaata
                        .categories
                        .iter()
                        .filter(|x| self.cratesio_category_filters.contains(&x.id))
                        .fold(Row::new(), |acc, x| acc.push(Text::new(x.name.clone())))
                        .into(),
                    scrollable(crates_list_column).into(),
                ]));

                if self.cratesio_show_category_list {
                    // --------------------------------
                    // categories widgets
                    // --------------------------------
                    container(scrollable(
                        self.crates_daaata
                            .categories_widgets
                            .iter()
                            .enumerate()
                            .fold(Column::new(), |acc, (i, x)| {
                                acc.push(x.view().map(move |message| match message {
                                    crate_finder::CategoryMessage::Hi(tang) => {
                                        Message::CategorySelected(tang)
                                    }
                                }))
                            }),
                    ))
                    .into()
                } else if self.cratesio_show_comparison {
                    // --------------------------------
                    // trends.io
                    // --------------------------------

                    let trendschart: Element<_> = container(
                        Canvas::new(&self.cratesio_chart_comp_data)
                            .height(Length::Fill)
                            .width(Length::Fill),
                    )
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .into();
                    trendschart.map(move |message| Message::DoNothing)
                } else if self.cratesio_show_readme.is_some() {
                    let markdown_input = &self
                        .crates_daaata
                        .crates2
                        .iter()
                        .find(|x| x.id == self.cratesio_show_readme.unwrap())
                        .unwrap()
                        .readme;
                    // let markdown_input = "# heading 1\n## heading 2\n### heading 3\n####heading 4\n##### heading 5\n######heading 6\nsome text";
                    let options = Options::empty();
                    let parser = Parser::new_ext(markdown_input, options);

                    let mut readmecol_vec: Vec<Element<Message>> = Vec::new();
                    let mut links_inc = 0;
                    let mut current_heading_level = None;
                    let mut inlink = false;
                    let mut inlinkdest = "".to_string();

                    // todo reimpl this without mut
                    // self.cratesio_links.clear();

                    for event in parser {
                        match event {
                            pulldown_cmark::Event::Start(tag) => match tag {
                                pulldown_cmark::Tag::Heading(heading_level, o1, o2) => {
                                    current_heading_level = Some(heading_level);
                                }

                                pulldown_cmark::Tag::Link(link_type, destination, title) => {
                                    inlink = true;
                                    inlinkdest = destination.to_string();
                                }

                                _ => (),
                            },
                            pulldown_cmark::Event::End(tag) => match tag {
                                pulldown_cmark::Tag::Heading(heading_level, o1, o2) => {
                                    current_heading_level = None;
                                }
                                pulldown_cmark::Tag::Link(link_type, destination, title) => {
                                    inlink = false;
                                }
                                _ => (),
                            },
                            pulldown_cmark::Event::Text(text) => match current_heading_level {
                                Some(heading_level) => match heading_level {
                                    pulldown_cmark::HeadingLevel::H1 => {
                                        readmecol_vec
                                            .push(Text::new(text.to_string()).size(60).into());
                                    }
                                    pulldown_cmark::HeadingLevel::H2 => {
                                        readmecol_vec
                                            .push(Text::new(text.to_string()).size(40).into());
                                    }
                                    pulldown_cmark::HeadingLevel::H3 => {
                                        readmecol_vec
                                            .push(Text::new(text.to_string()).size(30).into());
                                    }
                                    pulldown_cmark::HeadingLevel::H4 => {
                                        readmecol_vec
                                            .push(Text::new(text.to_string()).size(25).into());
                                    }
                                    _ => (),
                                },
                                None => {
                                    if inlink {
                                        let mylink =
                                            IcedLink::new(text.to_string(), inlinkdest.clone());

                                        // todo reimpl this without mut in view
                                        // self.cratesio_links.push(mylink);
                                        links_inc = links_inc + 1;
                                    } else {
                                        readmecol_vec.push(Text::new(text.to_string()).into());
                                    }
                                }
                            },
                            pulldown_cmark::Event::HardBreak => {
                                readmecol_vec.push(Text::new("hard break").into());
                            }
                            pulldown_cmark::Event::SoftBreak => {
                                readmecol_vec.push(Text::new("soft break").into());
                            }

                            // pulldown_cmark::Event:: Code(code) => {
                            //     readmecol.push(Text::new(text.to_string()).font());
                            // }

                            // pulldown_cmark::Event::Text()
                            _ => (),
                        };
                    }

                    // Write to String buffer.
                    // let mut html_output: String = String::with_capacity(markdown_input.len() * 3 / 2);
                    // html::push_html(&mut html_output, parser);

                    // Text::new(html_output).into()
                    let links_col: Element<_> = self
                        .cratesio_links
                        .iter()
                        .fold(Column::new(), |acc, x| {
                            acc.push(x.view().map(move |message| match message {
                                IcedLinkMessage::OpenLink(dest) => Message::OpenPathOrUrl(dest),
                            }))
                        })
                        .into();
                    let other_col = Column::with_children(readmecol_vec).into();

                    scrollable(Column::with_children(vec![links_col, other_col])).into()
                    // Scrollable::new(&mut self.cratesio_readme_scroll)
                    //     .push(Text::new(markdown_input))
                    //     .into()
                } else {
                    // --------------------------------
                    // crates list
                    // --------------------------------
                    crate_list_content.into()
                }
            })
            .into();

        // let content = content.explain(Color::BLACK);

        container(content)
            .height(Length::Fill)
            .width(Length::Fill)
            // .center_y()
            .into()
    }
}

pub struct IcedLink {
    title: String,
    destination: String,
}

#[derive(Clone)]
pub enum IcedLinkMessage {
    OpenLink(String),
}
impl IcedLink {
    fn new(title: String, destination: String) -> Self {
        Self { title, destination }
    }
    fn view(&self) -> Element<IcedLinkMessage> {
        button(text(self.title.clone()))
            .on_press(IcedLinkMessage::OpenLink(self.destination.clone()))
            .into()
    }
}
