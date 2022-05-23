use clap::Parser;
use db_editor_small::cli_args::MyArgs;
use db_editor_small::db_editor::DbEditor;
use iced::{pure::Application, Settings};

pub fn main() -> iced::Result {
    let args = MyArgs::parse();
    DbEditor::run(Settings::with_flags(args))
}
