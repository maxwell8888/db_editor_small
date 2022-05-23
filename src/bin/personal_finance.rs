#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use chrono::prelude::*;
use db_editor_small::charts;
use db_editor_small::colors;
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

pub fn main() -> iced::Result {
    Table::run(Settings::default())
}

// #[derive(Default)]
pub struct Table {
    selected_page: Page,

    account_group: AccountGroup,
    all_accs_data: AllAccountsWidget,
    pf_totals: Vec<TimePoint>,
    stacked_bar_columns: StackedBarData,

    linechart_data: charts::LineChartData,
    barchart_data: charts::LineChartData,
}

#[derive(Debug, Clone)]
pub enum Message {
    DoNothing,
    LanguageSelected(Page),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Santander,
    Monzo,
    MonthlyTable,
    MonthlyChart,
    MonthlyCategoryChart,
}

impl Page {
    const ALL: [Page; 5] = [
        Page::Santander,
        Page::Monzo,
        Page::MonthlyTable,
        Page::MonthlyChart,
        Page::MonthlyCategoryChart,
    ];
}

impl Default for Page {
    fn default() -> Page {
        Page::MonthlyChart
    }
}

impl std::fmt::Display for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Page::Santander => "Santander",
                Page::Monzo => "Monzo",
                Page::MonthlyTable => "MonthlyTable",
                Page::MonthlyChart => "MonthlyChart",
                Page::MonthlyCategoryChart => "MonthlyCategoryChart",
            }
        )
    }
}

// impl Sandbox for Table {
impl Application for Table {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();
    // type Message = Message;

    fn new(_flags: ()) -> (Table, Command<Message>) {
        let account_group = AccountGroup::new();
        let pf_totals = account_group.total();
        let stacked_bar_columns = StackedBarData::new(&account_group);

        // line chart
        let total_path = charts::ChartColumn {
            name: Some("Total".to_string()),
            data: pf_totals
                .iter()
                .map(|timepoint| Some(timepoint.starting_balance))
                .collect::<Vec<_>>(),
        };
        let monzo_path = charts::ChartColumn {
            name: Some("monzooo".to_string()),
            data: account_group.accounts[1]
                .monthly_agg_transactions_between(
                    account_group.get_date_range().0,
                    account_group.get_date_range().1,
                )
                .iter()
                .map(|timepoint| timepoint.starting_balance)
                .collect::<Vec<_>>(),
        };
        let santander_path = charts::ChartColumn {
            name: Some("Santander".to_string()),
            data: account_group.accounts[0]
                .monthly_agg_transactions_between(
                    account_group.get_date_range().0,
                    account_group.get_date_range().1,
                )
                .iter()
                .map(|timepoint| timepoint.starting_balance)
                .collect::<Vec<_>>(),
        };
        let linechart_data = charts::LineChartData {
            borders: charts::Borders::trbl(10., 320., 50., 50.),
            dates: account_group.combined_range(),
            columns: vec![monzo_path, santander_path, total_path],
            xinc: charts::Xinc::Month3,
            legend: true,
            typex: charts::ChartType::LineChart,
        };

        let barchart_data = charts::LineChartData {
            borders: charts::Borders::trbl(10., 320., 50., 50.),
            dates: account_group.combined_range(),
            columns: stacked_bar_columns.stacked_bar_column_data.clone(),
            xinc: charts::Xinc::Month3,
            legend: true,
            typex: charts::ChartType::StackedBarChart,
        };

        // let db_editor = db_editor::DbEditor::new("businesses.db");

        println!("init table");
        (
            Table {
                account_group,
                pf_totals,
                stacked_bar_columns,
                linechart_data,
                barchart_data,

                selected_page: Page::default(),

                all_accs_data: AllAccountsWidget::default(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("csv editor")
    }

    fn update(&mut self, event: Message) -> Command<Self::Message> {
        match event {
            Message::DoNothing => Command::none(),
            Message::LanguageSelected(language) => {
                self.selected_page = language;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let language_list = pick_list(
            &Page::ALL[..],
            Some(self.selected_page),
            Message::LanguageSelected,
        );

        let config_data = some_styles::ConfigData {
            col_width: 100,
            row_height: 35,
        };

        let acc1 = &self.account_group.accounts[0];
        let acc2 = &self.account_group.accounts[1];

        // inividual tables
        println!("{:?} iindiviudal tables", Utc::now());
        let account1 =
            personal_finance::individual_transactions(acc1).map(move |message| Message::DoNothing);
        let account2 =
            personal_finance::individual_transactions(acc2).map(move |message| Message::DoNothing);
        let date_width = 100;

        // combined table
        println!("{:?} combined table", Utc::now());
        let date_range = self.account_group.get_date_range();
        let all_accs = self
            .all_accs_data
            .make_all_accounts_widget(
                date_width,
                date_range,
                config_data,
                acc1,
                acc2,
                &self.pf_totals,
                &self.account_group,
            )
            .map(move |message| Message::DoNothing);

        let line_chart_canvas = Canvas::new(&self.linechart_data)
            .width(Length::Fill)
            .height(Length::Fill);

        let line_chart_widget: Element<charts::Whatever> = container(line_chart_canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into();

        // stacked bar chart
        println!("{:?} stacked bar chart", Utc::now());
        let bar_chart_canvas = Canvas::new(&self.barchart_data)
            .width(Length::Fill)
            .height(Length::Fill);

        let bar_chart_widget: Element<charts::Whatever> = container(bar_chart_canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into();

        let content: Element<_> = column()
            .spacing(20)
            .padding(20)
            .push(language_list)
            .push(text(self.selected_page.to_string()))
            // .push(chart_widget)
            .push(match self.selected_page {
                Page::Santander => account1,
                Page::Monzo => account2,
                Page::MonthlyTable => all_accs,
                Page::MonthlyChart => line_chart_widget.map(move |message| Message::DoNothing),
                Page::MonthlyCategoryChart => {
                    bar_chart_widget.map(move |message| Message::DoNothing)
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
