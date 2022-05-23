#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![feature(slice_group_by)]

mod sqlite_editor_lib;
pub use crate::sqlite_editor_lib::db_editor;

mod cli_args_lib;
pub use crate::cli_args_lib::cli_args;

mod crate_finder_lib;
pub use crate::crate_finder_lib::crate_finder;

pub mod personal_finance;

pub mod some_styles;

pub mod colors;

pub mod charts;

pub mod utils {
    pub fn pretty_number(num: usize) -> String {
        if num > 1000000 {
            format!("{}M", num / 1000000)
        } else if num > 1000 {
            format!("{}K", num / 1000)
        } else {
            num.to_string()
        }
    }
}

pub mod mytextbox;
