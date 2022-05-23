
extern crate chrono;
extern crate csv;
extern crate qif_parser;

use crate::charts;
use crate::some_styles::{self, ConfigData};
use chrono::prelude::*;
use iced::alignment::{Alignment, Horizontal, Vertical};
use iced::pure::{
    button, column, container, pick_list, row, scrollable, slider, text, text_input,
    widget::{
        button,
        canvas::event::{self, Event},
        canvas::{
            self, Cache, Canvas, Cursor, Fill, Frame, Geometry, LineCap, Path, Program, Stroke,
        },
        container, Button, Column, Container, Row, Text, TextInput,
    },
    Element, Sandbox,
};
use iced::tooltip::{self, Tooltip};
// use iced::widget::{button, container, pick_list, scrollable, slider, text_input, Scrollable};
// use iced::Application;
use iced::pure::Application;
use iced::Font;
use iced::{
    clipboard, window, Background, Checkbox, Color, Command, Length, PickList, Point, Radio,
    Rectangle, Settings, Size, Slider, Space, Vector,
};
// VerticalAlignment, Align, HorizontalAlignment,
use iced::{executor, mouse};
use rusqlite::{params, Connection, Result};
use std::fmt;
use std::fs;

#[derive(Clone)]
struct Payee {
    raw_name: String,
    clean_name: Option<String>,
    category: Option<String>,
}
// #[derive(Clone)]
// struct Payee2 {
//     raw_name: String,
//     clean_name: Option<String>,
//     category: Option<Category>,
// }

// fn clean_payee(raw_payee: String, data: Vec<Payee>) -> Option<Payee> {
//     let payee = match data
//         .iter()
//         .cloned()
//         .find(|payee| payee.raw_name == raw_payee)
//     {
//         Some(payee) => {
//             // this shouldn't really be necessary as I should read the data directly into a data model
//             let category = if payee.category == "groceries" {
//                 Category::Groceries
//             } else if payee.category == "bills" {
//                 Category::Bills
//             } else if
//             Payee2 {

//             }
//         },
//         None => None,
//     };
// }

#[derive(Debug, Clone)]
pub struct Transaction {
    pub date: NaiveDate,
    // time: String,
    pub amount: i64,
    pub starting_balance: i64,
    pub closing_balance: i64,
    pub payee: String,
    // category: String,
    // cleared_status: String,
    // address: String,
    // splits: String,
}

#[derive(Debug, Clone)]
pub struct SqlTransaction {
    pub account_id: i64,
    pub date: NaiveDate,
    pub amount: i64,
    pub starting_balance: i64,
    pub closing_balance: i64,
    pub payee_id: i64,
    pub category_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct Transaction2 {
    pub date: NaiveDate,
    // time: String,
    pub amount: i64,
    pub starting_balance: i64,
    pub closing_balance: i64,
    pub payee: String,
    // this should probs be an enum
    pub payee_std: String,
    pub category: Category,
    // pub payee_std: Option<String>,
    // pub category: Option<Category>,
    // cleared_status: String,
    // address: String,
    // splits: String
}

#[derive(Debug, Clone)]
pub struct TimePoint {
    pub date: NaiveDate,
    pub amount: i64,
    pub starting_balance: i64,
    pub closing_balance: i64,
}
impl fmt::Display for TimePoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}\t{}\t{}\t{}",
            self.date,
            self.amount / 100,
            self.starting_balance / 100,
            self.closing_balance / 100,
        )
    }
}

pub struct TimePoint2 {
    pub date: NaiveDate,
    pub amount: Option<i64>,
    pub starting_balance: Option<i64>,
    pub closing_balance: Option<i64>,
}
impl From<TimePoint> for TimePoint2 {
    fn from(timepoint: TimePoint) -> Self {
        let TimePoint {
            date,
            amount,
            starting_balance,
            closing_balance,
        } = timepoint;
        TimePoint2 {
            date,
            amount: Some(amount),
            starting_balance: Some(starting_balance),
            closing_balance: Some(closing_balance),
        }
    }
}
impl fmt::Display for TimePoint2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}\t{}\t{}\t{}",
            self.date,
            match self.amount {
                Some(val) => (val / 100).to_string(),
                None => "".to_string(),
            },
            match self.starting_balance {
                Some(val) => (val / 100).to_string().to_string(),
                None => "".to_string(),
            },
            match self.closing_balance {
                Some(val) => (val / 100).to_string().to_string(),
                None => "".to_string(),
            },
        )
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum Category {
    Groceries,
    Bills,
    OtherIn,
    OtherOut,
    Salary,
    RentalIncome,
}
impl Category {
    pub fn iterator() -> impl Iterator<Item = Category> {
        [
            Category::Bills,
            Category::OtherIn,
            Category::OtherOut,
            Category::Groceries,
            Category::Salary,
            Category::RentalIncome,
        ]
        .iter()
        .copied()
    }
}
impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// this is an aggregation of Transaction with category
#[derive(Debug, Clone)]
pub struct TimePoint3 {
    pub date: NaiveDate,
    pub category: Category,
    pub amount: i64,
    pub starting_balance: i64,
    pub closing_balance: i64,
}

#[derive(Debug, Clone)]
pub struct PayeeMapping {
    pub original: String,
    pub standardised: String,
    pub category: Option<String>,
}
impl PayeeMapping {
    pub fn new(original: String, standardised: String, category: Option<String>) -> PayeeMapping {
        PayeeMapping {
            original,
            standardised,
            category,
        }
    }
}
impl fmt::Display for PayeeMapping {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}, {}, {}",
            self.original,
            self.standardised,
            match &self.category {
                Some(val) => val,
                None => "no cat",
            }
        )
    }
}

#[derive(Clone, Default)]
pub struct AccountGroup {
    pub accounts: Vec<Account>,
}

#[derive(Clone, Default)]
pub struct StackedBarData {
    pub stacked_bar_column_data: Vec<charts::ChartColumn>,
}
impl StackedBarData {
    pub fn new(accounts: &AccountGroup) -> Self {
        Self {
            stacked_bar_column_data: Category::iterator().fold(Vec::new(), |mut acc, x| {
                acc.push(charts::ChartColumn {
                    name: Some(x.to_string()),
                    data: accounts
                        .combined_range()
                        .iter()
                        .map(|date| {
                            match accounts.total_by_mmonth_and_cat().iter().find(|timepoint| {
                                timepoint.category == x && timepoint.date == *date
                            }) {
                                Some(timepoint) => Some(timepoint.amount),
                                None => None,
                            }
                        })
                        .collect::<Vec<_>>(),
                    ..charts::ChartColumn::default()
                });
                acc
            }),
        }
    }
}

impl AccountGroup {
    // pub fn all_data(&self) -> Vec
    // &self.combined_range()
    // fn monthly_agg_transactions(&self) -> Vec<TimePoint> {
    //     self.transactions
    //         .group_by(|a, b| {
    //             NaiveDate::from_ymd(a.date.year(), a.date.month(), 1)
    //                 == NaiveDate::from_ymd(b.date.year(), b.date.month(), 1)
    //         })
    //         .map(|group| TimePoint {
    //             date: NaiveDate::from_ymd(group[0].date.year(), group[0].date.month(), 1),
    //             amount: group.iter().map(|transaction| transaction.amount).sum(),
    //             starting_balance: group[0].starting_balance,
    //         })
    //         .collect::<Vec<_>>()
    // }
    pub fn new() -> Self {
        let santander_starting_balance = 0;
        let mut santander_dir = fs::read_dir("data/santander_credit_card")
            .unwrap()
            .map(|entry| entry.unwrap())
            .collect::<Vec<_>>();
        santander_dir.sort_by_key(|entry| {
            let filename = entry.file_name();
            let filename = filename.into_string().unwrap();
            let filename = filename.split('.').next().unwrap().to_string();
            filename
        });
        // println!("{:?}", santander_dir);

        // let santander_qif_strings = fs::read_dir("data/santander_credit_card")
        //     .unwrap()
        //     .map(|entry| fs::read_to_string(entry.unwrap().path()).unwrap())
        //     .collect::<Vec<_>>();
        let santander_qif_strings = santander_dir
            .iter()
            .map(|entry| fs::read_to_string(entry.path()).unwrap())
            .collect::<Vec<_>>();
        let santander_qifs = santander_qif_strings
            .iter()
            .map(|qif_string| qif_parser::parse(qif_string, "%d/%m/%Y").unwrap())
            .collect::<Vec<_>>();
        // need to sort by date?
        let santander_account = Account::from_qifs(
            String::from("santander"),
            santander_starting_balance,
            santander_qifs,
        );

        // monzo
        let monzo_starting_balance = 0;
        let monzo_dir = fs::read_dir("data/monzo_current_account")
            .unwrap()
            .map(|entry| entry.unwrap())
            .collect::<Vec<_>>();
        // monzo_dir.sort_by_key(|entry| {
        //     let filename = entry.file_name();
        //     let filename = filename.into_string().unwrap();
        //     let filename = filename.split('.').next().unwrap().to_string();
        //     filename
        // });

        let monzo_qif_strings = monzo_dir
            .iter()
            .map(|entry| fs::read_to_string(entry.path()).unwrap())
            .collect::<Vec<_>>();
        let monzo_qifs = monzo_qif_strings
            .iter()
            .map(|qif_string| qif_parser::parse(qif_string, "%d/%m/%Y").unwrap())
            .collect::<Vec<_>>();
        // need to sort by date?
        let monzo_account =
            Account::from_qifs(String::from("monzo"), monzo_starting_balance, monzo_qifs);

        let accounts = AccountGroup {
            accounts: vec![santander_account, monzo_account],
        };

        accounts
    }

    pub fn get_date_range(&self) -> (NaiveDate, NaiveDate) {
        let ranges = self
            .accounts
            .iter()
            .map(|account| account.get_date_range())
            .collect::<Vec<_>>();

        let min = ranges.iter().map(|range| range.0).min().unwrap();
        let max = ranges.iter().map(|range| range.1).max().unwrap();
        (min, max)
    }
    pub fn combined_range(&self) -> Vec<NaiveDate> {
        let mut combined_dates = Vec::new();
        let (start, end) = self.get_date_range();
        let mut curr_date = NaiveDate::from_ymd(start.year(), start.month(), 1);
        while curr_date < end {
            combined_dates.push(curr_date);
            let mut year = curr_date.year();
            let mut month = curr_date.month();
            if month == 12 {
                month = 1;
                year = year + 1;
            } else {
                month = month + 1;
            }
            curr_date = NaiveDate::from_ymd(year, month, 1);
        }
        combined_dates
        // let mut start_date =
    }
    // combined dates - basically out join on dates
    // this different to combined_range(), it will only have used dates - i.e. it might have gaps between dates
    fn combined_dates(&self) -> Vec<NaiveDate> {
        let mut combined_dates = Vec::new();
        for account in &self.accounts {
            combined_dates.extend(
                &account
                    .transactions
                    .iter()
                    .map(|t| t.date)
                    .collect::<Vec<_>>()
                    .clone(),
            );
        }
        combined_dates.sort();
        combined_dates.dedup();
        combined_dates
    }

    // this should really be using combined dates to return a vec with missing entries, then a second function to fill in the gaps
    pub fn total(&self) -> Vec<TimePoint> {
        let (start, end) = self.get_date_range();
        let acc_monthlys = self
            .accounts
            .iter()
            .map(|account| account.monthly_agg_transactions_between(start, end))
            .collect::<Vec<_>>();

        self.combined_range()
            .iter()
            .enumerate()
            .map(|(i, date)| {
                let amount = acc_monthlys
                    .iter()
                    .map(|ts| match ts[i].amount {
                        Some(val) => val,
                        None => 0,
                    })
                    .sum::<i64>();
                let starting_balance = acc_monthlys
                    .iter()
                    .map(|ts| match ts[i].starting_balance {
                        Some(val) => val,
                        None => 0,
                    })
                    .sum::<i64>();
                let closing_balance = acc_monthlys
                    .iter()
                    .map(|ts| match ts[i].closing_balance {
                        Some(val) => val,
                        None => 0,
                    })
                    .sum::<i64>();
                // let amount =
                TimePoint {
                    date: *date,
                    amount,
                    starting_balance,
                    closing_balance,
                }
            })
            .collect::<Vec<_>>()
    }

    // append all transactions
    fn all_transactions(&self, starting_balance: i64) -> Vec<Transaction> {
        let mut transactions = self.accounts.iter().fold(Vec::new(), |mut acc, x| {
            let mut transactions = x.transactions.clone();
            acc.append(&mut transactions);
            acc
        });
        transactions.sort_by_key(|transaction| transaction.date);
        // need to recalculate starting and closing now that multiple transactions have been mixed in together
        transactions = transactions
            .iter()
            .map(|transaction| Transaction {
                date: transaction.date,
                amount: transaction.amount,
                starting_balance: starting_balance,
                closing_balance: 0,
                payee: transaction.payee.clone(),
            })
            .collect::<Vec<_>>();
        let mut balance = starting_balance;
        for transaction in transactions.iter_mut() {
            transaction.starting_balance = balance;
            balance = balance + transaction.amount;
            transaction.closing_balance = balance;
        }
        transactions
    }
    pub fn all_transactions_mapped_payee(&self) -> Vec<Transaction2> {
        let conn = Connection::open("businesses.db").unwrap();
        let mut stmt = conn
            .prepare(
                "select p.raw_payee_name, b.name, c.name as category from payees p 
            left join business b on p.business_id = b.id
            left join category c on b.category_id = c.id;",
            )
            .unwrap();
        let payee_iter = stmt
            .query_map([], |row| {
                Ok(Payee {
                    raw_name: row.get(0)?,
                    clean_name: row.get(1)?,
                    category: row.get(2)?,
                })
            })
            .unwrap();
        let mut payee_vec = Vec::new();
        for payee in payee_iter {
            payee_vec.push(payee.unwrap());
        }
        self.all_transactions(0)
            .iter()
            .map(|transaction| {
                // I was going to use Options but grouping by Options seems like a harder problem which requires more though so just assuming all payees can be found for now
                let payee = payee_vec
                    .iter()
                    .cloned()
                    .find(|payee| payee.raw_name == transaction.payee)
                    .unwrap();
                let category = match payee.category {
                    Some(category) => {
                        if category == "groceries" {
                            Category::Groceries
                        } else if category == "bills" {
                            Category::Bills
                        } else if category == "other_out" {
                            Category::OtherOut
                        } else if category == "salary" {
                            Category::Salary
                        } else if category == "rental_income" {
                            Category::RentalIncome
                        } else {
                            panic!("error calculating category");
                        }
                    }
                    None => {
                        if transaction.amount > 0 {
                            Category::OtherIn
                            // this panics with < not <=, so there is 0 transactions - which there probably shouldn't be...
                        } else if transaction.amount <= 0 {
                            Category::OtherOut
                        } else {
                            panic!("error calculating category");
                        }
                    }
                };
                let payee_std = match payee.clean_name {
                    Some(clean_name) => clean_name,
                    None => transaction.payee.clone(),
                };

                Transaction2 {
                    date: transaction.date,
                    payee: transaction.payee.clone(),
                    payee_std,
                    category,
                    amount: transaction.amount,
                    starting_balance: transaction.starting_balance,
                    closing_balance: transaction.closing_balance,
                }
            })
            .collect::<Vec<_>>()
    }
    // the starting and closing are wrong, but leave for now since we only really want to plot amount anyway
    pub fn total_by_mmonth_and_cat(&self) -> Vec<TimePoint3> {
        let mut all_transactions = self.all_transactions_mapped_payee();
        all_transactions.sort_by(|a, b| a.category.cmp(&b.category));
        all_transactions.sort_by(|a, b| {
            NaiveDate::from_ymd(a.date.year(), a.date.month(), 1).cmp(&NaiveDate::from_ymd(
                b.date.year(),
                b.date.month(),
                1,
            ))
        });
        all_transactions
            .group_by(|a, b| {
                NaiveDate::from_ymd(a.date.year(), a.date.month(), 1)
                    == NaiveDate::from_ymd(b.date.year(), b.date.month(), 1)
                    && a.category == b.category
            })
            .map(|group| TimePoint3 {
                date: NaiveDate::from_ymd(group[0].date.year(), group[0].date.month(), 1),
                category: group[0].category,
                amount: group.iter().map(|transaction| transaction.amount).sum(),
                starting_balance: group[0].starting_balance,
                closing_balance: group.last().unwrap().closing_balance,
            })
            .collect::<Vec<_>>()
    }

    pub fn print(&self) {
        // print headings
        print!("\t\t");
        for account in &self.accounts {
            print!("{}\t\t", account.company_name);
        }
        print!("\ttotal");
        println!();

        print!("\t\t");
        for account in &self.accounts {
            print!("amount\tstart\tclose\t");
        }
        print!("amount\tstart\tclose");
        println!();
        // let mut combined_months = self
        //     .combined_dates()
        //     .iter()
        //     .map(|d| NaiveDate::from_ymd(d.year(), d.month(), 1))
        //     .collect::<Vec<_>>();
        // combined_months.dedup();
        for tdate in &self.combined_range() {
            // date
            print!("{}\t", tdate);

            // accounts
            for account in &self.accounts {
                let monthly_transactions = account.monthly_agg_transactions_dense();
                let tdate_transactions = monthly_transactions
                    .iter()
                    .filter(|t| t.date == *tdate)
                    .collect::<Vec<_>>();
                // let amount: i64 = tdate_transactions.iter().map(|t| t.amount).sum();
                let (amount, start_balance, close_balance) = if tdate_transactions.len() > 0 {
                    (
                        Some(tdate_transactions[0].amount / 100),
                        Some(tdate_transactions[0].starting_balance / 100),
                        Some(tdate_transactions[0].closing_balance / 100),
                    )
                } else {
                    (None, None, None)
                };
                print!(
                    "{}\t{}\t{}\t",
                    match amount {
                        Some(val) => val.to_string(),
                        None => "".to_string(),
                    },
                    match start_balance {
                        Some(val) => val.to_string(),
                        None => "".to_string(),
                    },
                    match close_balance {
                        Some(val) => val.to_string(),
                        None => "".to_string(),
                    },
                );
            }
            // total
            let mut amount_total = 0;
            let mut start_total = 0;
            let mut close_total = 0;
            for account in &self.accounts {
                let monthly_transactions = account.monthly_agg_transactions_dense();
                let tdate_transactions = monthly_transactions
                    .iter()
                    .filter(|t| t.date == *tdate)
                    .collect::<Vec<_>>();
                if tdate_transactions.len() > 0 {
                    amount_total += tdate_transactions[0].amount;
                    start_total += tdate_transactions[0].starting_balance;
                    close_total += tdate_transactions[0].closing_balance;
                }
            }
            print!(
                "{}\t{}\t{}\t",
                amount_total / 100,
                start_total / 100,
                close_total / 100
            );

            println!();
        }
    }
    pub fn get_all_payees_sorted_dedup(&self) -> Vec<String> {
        let mut payees = Vec::new();
        for account in &self.accounts {
            payees.extend(account.transactions.iter().map(|t| t.payee.clone()));
        }
        payees.sort();
        payees.dedup();
        payees
    }

    // pub fn create_sql_inserts_new_data(&self) -> Vec<SqlTransaction> {
    //     let updates: Vec<SqlTransaction>   = Vec::new();
    //     self.accounts.iter().map(|account| account.transactions)
    // }
}

#[derive(Debug, Clone)]
pub struct Account {
    pub company_name: String,
    pub transactions: Vec<Transaction>,
}
impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut display_string = String::from("");
        display_string.push_str(&self.company_name);
        display_string.push_str("\n");
        display_string.push_str("\ndate\t\tamount\tbalance\tpayee");
        for transaction in self.transactions.iter() {
            display_string.push_str(&format!(
                "\n{}\t{}\t{}\t{}",
                transaction.date,
                transaction.amount / 100,
                transaction.starting_balance / 100,
                transaction.payee
            ));
        }
        write!(f, "{}", display_string)
    }
}
impl Account {
    pub fn from_dir(path: &str, name: String) -> Self {
        // get list of files in folder
        let mut dir_entries = fs::read_dir(path)
            .unwrap()
            .map(|entry| entry.unwrap())
            .collect::<Vec<_>>();
        dir_entries.sort_by_key(|entry| {
            let filename = entry.file_name().into_string().unwrap();
            let filename = filename.split('.').next().unwrap().to_string();
            filename
        });

        // read and parse qif strings from folder. have to keep strings allocated as qif_parser only returns references
        let qif_strings = dir_entries
            .iter()
            .map(|entry| fs::read_to_string(entry.path()).unwrap())
            .collect::<Vec<_>>();
        let qifs = qif_strings
            .iter()
            .map(|qif_string| qif_parser::parse(qif_string, "%d/%m/%Y").unwrap())
            .collect::<Vec<_>>();
        // need to sort by date?
        Account::from_qifs(name, 0, qifs)
    }
    // not qifs must be in order. should add a check?
    pub fn from_qifs(
        company_name: String,
        starting_balance: i64,
        qifs: Vec<qif_parser::qif::Qif>,
    ) -> Self {
        let mut transactions = qifs
            .iter()
            .flat_map(|qif| {
                // transactions.reverse();
                qif.transactions.clone()
            })
            // initial balance seems to only be included when the balance doesn't start at 0 (i.e. is a new account) we simply drop it here but should be used to verify data and indentify missing transactions or other inconsistencies
            .filter(|transaction| transaction.payee != "INITIAL BALANCE")
            .map(|transaction| Transaction {
                date: NaiveDate::parse_from_str(&transaction.date, "%Y-%m-%d").unwrap(),
                amount: transaction.amount,
                starting_balance,
                closing_balance: 0,
                payee: transaction.payee.to_string(),
            })
            .collect::<Vec<_>>();
        let mut balance = starting_balance;
        transactions.iter_mut().for_each(|transaction| {
            transaction.starting_balance = balance;
            balance = balance + transaction.amount;
            transaction.closing_balance = balance;
        });
        Account {
            company_name: company_name,
            transactions: transactions,
        }
    }
    fn get_date_range(&self) -> (NaiveDate, NaiveDate) {
        (
            self.transactions[0].date.clone(),
            self.transactions.last().unwrap().date.clone(),
        )
    }
    fn daily_agg_transactions(&self) -> Vec<TimePoint> {
        self.transactions
            .group_by(|a, b| a.date == b.date)
            .map(|group| TimePoint {
                date: group[0].date,
                amount: group.iter().map(|transaction| transaction.amount).sum(),
                starting_balance: group[0].starting_balance,
                closing_balance: group[0].closing_balance,
            })
            .collect::<Vec<_>>()
    }
    fn monthly_agg_transactions_sparse(&self) -> Vec<TimePoint> {
        self.transactions
            .group_by(|a, b| {
                NaiveDate::from_ymd(a.date.year(), a.date.month(), 1)
                    == NaiveDate::from_ymd(b.date.year(), b.date.month(), 1)
            })
            .map(|group| TimePoint {
                date: NaiveDate::from_ymd(group[0].date.year(), group[0].date.month(), 1),
                amount: group.iter().map(|transaction| transaction.amount).sum(),
                starting_balance: group[0].starting_balance,
                closing_balance: group.last().unwrap().closing_balance,
            })
            .collect::<Vec<_>>()
    }
    pub fn monthly_agg_transactions_dense(&self) -> Vec<TimePoint> {
        let (start, end) = self.get_date_range();
        let curr_month = start.clone();
        let mut curr_month = NaiveDate::from_ymd(curr_month.year(), curr_month.month(), 1);
        let sparse = self.monthly_agg_transactions_sparse();
        let mut new_transactions = Vec::new();
        let mut close_balance = 0;
        while curr_month < end {
            let transaction = sparse
                .iter()
                .map(|t| t.clone())
                .filter(|t| t.date == curr_month)
                .collect::<Vec<_>>();
            if transaction.first().is_some() {
                let tt = transaction.first().unwrap().clone();
                close_balance = tt.closing_balance;
                new_transactions.push(tt);
            } else {
                new_transactions.push(TimePoint {
                    date: curr_month,
                    amount: 0,
                    starting_balance: close_balance,
                    closing_balance: close_balance,
                });
            }
            let mut year = curr_month.year();
            let mut month = curr_month.month();
            if month == 12 {
                year = year + 1;
                month = 1;
            } else {
                month = month + 1;
            }
            curr_month = NaiveDate::from_ymd(year, month, 1);
        }
        new_transactions
    }

    // need a way of dertmining when an account is closed - only sure way is if there is a marker in e.g. the qif data.
    pub fn monthly_agg_transactions_between(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Vec<TimePoint2> {
        // let (start, end) = self.get_date_range();
        let curr_month = start.clone();
        let mut curr_month = NaiveDate::from_ymd(curr_month.year(), curr_month.month(), 1);
        let sparse = self.monthly_agg_transactions_sparse();
        let mut new_transactions = Vec::new();
        let mut close_balance = 0;
        let mut account_started = false;
        while curr_month < end {
            let transaction = sparse
                .iter()
                .map(|t| t.clone())
                .filter(|t| t.date == curr_month)
                .collect::<Vec<_>>();
            if transaction.first().is_some() {
                account_started = true;
                let tt = transaction.first().unwrap().clone();
                close_balance = tt.closing_balance;
                let tt2 = TimePoint2::from(tt);
                new_transactions.push(tt2);
            } else if account_started {
                new_transactions.push(TimePoint2 {
                    date: curr_month,
                    amount: Some(0),
                    starting_balance: Some(close_balance),
                    closing_balance: Some(close_balance),
                });
            } else {
                new_transactions.push(TimePoint2 {
                    date: curr_month,
                    amount: None,
                    starting_balance: None,
                    closing_balance: None,
                });
            }
            let mut year = curr_month.year();
            let mut month = curr_month.month();
            if month == 12 {
                year = year + 1;
                month = 1;
            } else {
                month = month + 1;
            }
            curr_month = NaiveDate::from_ymd(year, month, 1);
        }
        new_transactions
    }

    // pub fn monthly_agg_transactions_by_cat_between(
    //     &self,
    //     start: NaiveDate,
    //     end: NaiveDate,
    // ) -> Vec<TimePoint3> {
    //     // let (start, end) = self.get_date_range();
    //     let mut curr_month = start.clone();
    //     let mut curr_month = NaiveDate::from_ymd(curr_month.year(), curr_month.month(), 1);

    //     let sparse = self.monthly_agg_transactions_sparse();
    //     let mut new_transactions = Vec::new();
    //     let mut close_balance = 0;
    //     let mut account_started = false;
    //     while curr_month < end {
    //         let transaction = sparse
    //             .iter()
    //             .map(|t| t.clone())
    //             .filter(|t| t.date == curr_month)
    //             .collect::<Vec<_>>();
    //         if transaction.first().is_some() {
    //             account_started = true;
    //             let tt = transaction.first().unwrap().clone();
    //             close_balance = tt.closing_balance;
    //             let tt2 = TimePoint2::from(tt);
    //             new_transactions.push(tt2);
    //         } else if account_started {
    //             new_transactions.push(TimePoint2 {
    //                 date: curr_month,
    //                 amount: Some(0),
    //                 starting_balance: Some(close_balance),
    //                 closing_balance: Some(close_balance),
    //             });
    //         } else {
    //             new_transactions.push(TimePoint2 {
    //                 date: curr_month,
    //                 amount: None,
    //                 starting_balance: None,
    //                 closing_balance: None,
    //             });
    //         }
    //         let mut year = curr_month.year();
    //         let mut month = curr_month.month();
    //         if month == 12 {
    //             year = year + 1;
    //             month = 1;
    //         } else {
    //             month = month + 1;
    //         }
    //         curr_month = NaiveDate::from_ymd(year, month, 1);
    //     }
    //     new_transactions
    // }
}

#[derive(Clone)]
pub enum PfMessage {
    DingDong,
}

#[derive(Default)]
pub struct AllAccountsWidget {}
impl AllAccountsWidget {
    pub fn make_all_accounts_widget(
        &self,
        date_width: u16,
        date_range: (NaiveDate, NaiveDate),
        config_data: ConfigData,
        acc1: &Account,
        acc2: &Account,
        totals: &Vec<TimePoint>,
        account_group: &AccountGroup,
    ) -> Element<PfMessage> {
        Column::with_children(vec![
            container(Row::with_children(vec![
                Column::with_children(vec![
                    text("").width(Length::Units(date_width)).into(),
                    text("").width(Length::Units(date_width)).into(),
                ])
                .into(),
                account_header(&acc1.company_name, config_data)
                    .style(some_styles::BlueBackground)
                    .into(),
                account_header(&acc2.company_name, config_data)
                    .style(some_styles::GreenBackground)
                    .into(),
                account_header("Total", config_data.clone())
                    .style(some_styles::BlueBackground)
                    .into(),
            ]))
            // .style(style::BlackBorder)
            // .style(style::LightGreyBackground)
            .into(),
            scrollable(container(Row::with_children(vec![
                // self.account_group
                account_group
                    .combined_range()
                    .iter()
                    .fold(Column::new(), |column, date| {
                        column.push(
                            text(date.to_string())
                                .width(Length::Units(date_width))
                                .height(Length::Units(config_data.row_height))
                                .horizontal_alignment(Horizontal::Center)
                                .vertical_alignment(Vertical::Center),
                        )
                    })
                    .into(),
                account_data_view(&acc1, date_range, &config_data)
                    .style(some_styles::LightBlueBackground)
                    .into(),
                account_data_view(&acc2, date_range, &config_data)
                    .style(some_styles::LightGreenBackground)
                    .into(),
                account_data_view_totals(totals, date_range, &config_data)
                    .style(some_styles::LightBlueBackground)
                    .into(),
            ])))
            .into(),
        ])
        .into()
    }
}

pub fn individual_transactions(acc: &Account) -> Element<PfMessage> {
    Container::new(Column::with_children(vec![
        Container::new(Text::new(acc.company_name.clone())).into(),
        acc.transactions
            .iter()
            .fold(Column::new(), |column, transaction| {
                column.push(
                    Row::with_children(vec![
                        Text::new(transaction.date.to_string())
                            .width(Length::Units(100))
                            .horizontal_alignment(Horizontal::Right)
                            .into(),
                        Text::new(transaction.payee.to_string())
                            .width(Length::Units(350))
                            .into(),
                        Text::new((transaction.amount / 100).to_string())
                            .width(Length::Units(50))
                            .horizontal_alignment(Horizontal::Right)
                            .into(),
                        Text::new((transaction.starting_balance / 100).to_string())
                            .width(Length::Units(50))
                            .horizontal_alignment(Horizontal::Right)
                            .into(),
                        Text::new((transaction.closing_balance / 100).to_string())
                            .width(Length::Units(50))
                            .horizontal_alignment(Horizontal::Right)
                            .into(),
                    ])
                    .height(Length::Units(25)),
                )
            })
            .into(),
    ]))
    .into()
}

pub fn account_data_view(
    acc: &Account,
    date_range: (NaiveDate, NaiveDate),
    config: &some_styles::ConfigData,
) -> Container<'static, PfMessage> {
    let some_styles::ConfigData {
        col_width,
        row_height,
    } = *config;
    let (start, end) = date_range;
    Container::new(
        acc.monthly_agg_transactions_between(start, end)
            .iter()
            .fold(Column::new(), |column, transaction| {
                column.push(
                    Row::with_children(vec![
                        Container::new(
                            Text::new(match transaction.amount {
                                Some(val) => (val / 100).to_string(),
                                None => "".to_string(),
                            })
                            .width(Length::Units(col_width))
                            .horizontal_alignment(Horizontal::Center)
                            .height(Length::Units(row_height))
                            .vertical_alignment(Vertical::Center),
                        )
                        // .style(style::LightBlueBackground)
                        .into(),
                        Container::new(
                            Text::new(match transaction.starting_balance {
                                Some(val) => (val / 100).to_string(),
                                None => "".to_string(),
                            })
                            .width(Length::Units(col_width))
                            .horizontal_alignment(Horizontal::Center)
                            .height(Length::Units(row_height))
                            .vertical_alignment(Vertical::Center),
                        )
                        // .style(style::LightBlueBackground)
                        .into(),
                        Container::new(
                            Text::new(match transaction.closing_balance {
                                Some(val) => (val / 100).to_string(),
                                None => "".to_string(),
                            })
                            .width(Length::Units(col_width))
                            .horizontal_alignment(Horizontal::Center)
                            .height(Length::Units(row_height))
                            .vertical_alignment(Vertical::Center),
                        )
                        // .style(style::LightBlueBackground)
                        .into(),
                    ]),
                    // .align_items(iced::Align::Center)
                    // .vertical_alignment(VerticalAlignment::Center)
                    // .height(Length::Units(row_height)),
                )
                // .padding(20)
            }),
    )
}

// have made function essentially identical to above, could probs use one function and just pass the result of acc.monthly_agg_transactions_between(), but that is Timepoint2 and the totals are TimePoint, so maybe refactor later
pub fn account_data_view_totals(
    acc: &Vec<TimePoint>,
    date_range: (NaiveDate, NaiveDate),
    config: &some_styles::ConfigData,
) -> Container<'static, PfMessage> {
    let some_styles::ConfigData {
        col_width,
        row_height,
    } = *config;
    let (start, end) = date_range;
    Container::new(acc.iter().fold(Column::new(), |column, transaction| {
        column.push(
            Row::with_children(vec![
                Container::new(
                    Text::new((transaction.amount / 100).to_string())
                        .width(Length::Units(col_width))
                        .horizontal_alignment(Horizontal::Center)
                        .height(Length::Units(row_height))
                        .vertical_alignment(Vertical::Center),
                )
                // .style(style::LightBlueBackground)
                .into(),
                Container::new(
                    Text::new((transaction.starting_balance / 100).to_string())
                        .width(Length::Units(col_width))
                        .horizontal_alignment(Horizontal::Center)
                        .height(Length::Units(row_height))
                        .vertical_alignment(Vertical::Center),
                )
                // .style(style::LightBlueBackground)
                .into(),
                Container::new(
                    Text::new((transaction.closing_balance / 100).to_string())
                        .width(Length::Units(col_width))
                        .horizontal_alignment(Horizontal::Center)
                        .height(Length::Units(row_height))
                        .vertical_alignment(Vertical::Center),
                )
                // .style(style::LightBlueBackground)
                .into(),
            ]),
            // .align_items(iced::Align::Center)
            // .vertical_alignment(VerticalAlignment::Center)
            // .height(Length::Units(row_height)),
        )
        // .padding(20)
    }))
}

// fn cell_widget()
pub fn account_header<'a>(
    company_name: &str,
    config: some_styles::ConfigData,
) -> Container<'a, PfMessage> {
    let some_styles::ConfigData {
        col_width,
        row_height,
    } = config;
    Container::new(Column::with_children(vec![
        Row::with_children(vec![Text::new(company_name)
            .horizontal_alignment(Horizontal::Center)
            .width(Length::Units(col_width * 3))
            .into()])
        .height(Length::Units(row_height))
        .into(),
        Row::with_children(
            ["amount", "start", "close"]
                .map(|x| {
                    Text::new(x)
                        .width(Length::Units(col_width))
                        .horizontal_alignment(Horizontal::Center)
                        .into()
                })
                .into(),
        )
        .height(Length::Units(row_height))
        .into(),
    ]))
}

// , background_color: impl iced::container::StyleSheet +'static
// fn data_cell(val: String  ) -> Container<Message> { // this creates lifetime problems
// fn data_cell(val: i64  ) -> Container<Message> { // also creates lifetime problems????
pub fn data_cell(val: &str) -> Container<PfMessage> {
    // reference is no good cos it gets lost..?

    let col_width = 100;
    let row_height = 35;

    Container::new(
        // Text::new(  value/100   )
        Text::new(val)
            .width(Length::Units(col_width))
            .height(Length::Units(row_height))
            .horizontal_alignment(Horizontal::Center)
            .vertical_alignment(Vertical::Center),
    )
    // .style(background_color)
}

fn standardise_payee(payee: String, standarised_payees: Vec<PayeeMapping>) -> String {
    standarised_payees
        .iter()
        .filter(|p| p.original == payee)
        .collect::<Vec<_>>()
        .first()
        .unwrap()
        .standardised
        .clone()
}

pub fn print_vector_display<T: std::fmt::Display>(v: Vec<T>) {
    for x in v.iter() {
        println!("{}", x);
    }
}
