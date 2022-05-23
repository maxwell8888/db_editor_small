use crate::cli_args::MyArgs;
use crate::mytextbox::{self, MyTextbox};
use crate::{colors, some_styles};

use chrono::prelude::*;
use iced::alignment::{Alignment, Horizontal, Vertical};
use iced::keyboard::KeyCode;
use iced::pure::widget::Container;
use iced::pure::{
    button, column, container, pick_list, row, scrollable, slider, text, text_input,
    widget::{
        button,
        canvas::event::{self, Event},
        canvas::{
            self, Cache, Canvas, Cursor, Fill, Frame, Geometry, LineCap, Path, Program, Stroke,
        },
        container, text_input, Button, Column, Row, Text, TextInput,
    },
    Element, Sandbox,
};
use iced::tooltip::{self, Tooltip};
use iced_native;
use iced_native::event::Status;
use palette::{self, convert::FromColor, Hsl, Srgb};
use std::fs;
// use iced::widget::{button, container, pick_list, scrollable, slider, text_input, Scrollable};
// use iced::Application;
use iced::pure::Application;
use iced::Renderer;
use iced::{
    clipboard, keyboard, window, Background, Checkbox, Color, Command, Length, PickList, Point,
    Radio, Rectangle, Settings, Size, Slider, Space, Vector,
};
use iced::{Font, Subscription};
// VerticalAlignment, Align, HorizontalAlignment,
use iced::{executor, mouse};
use rusqlite::MAIN_DB;
use rusqlite::{
    params,
    types::{Type, Value},
    Connection, Result, ToSql,
};
use sqlparser::ast::{
    ColumnDef, ColumnOption, ColumnOptionDef, DataType, Ident, ObjectName, Statement,
    TableConstraint,
};
use sqlparser::dialect::{GenericDialect, SQLiteDialect};
use sqlparser::parser::Parser;
use std::collections::HashMap;
use std::fmt::{self, Debug};

#[derive(Debug)]
enum OutputText {
    SucessfulSql(String),
    Other(String),
    Error(String),
}

struct Tables {
    selected_table: Option<String>,
    reference_data: Vec<String>,
    tables: Vec<CoolTable>,
}

impl Tables {
    fn get_table(&self, table_name: &str) -> &CoolTable {
        self.tables
            .iter()
            .find(|table| table.name == table_name)
            .unwrap()
    }
    fn get_mut_table(&mut self, table_name: &str) -> &mut CoolTable {
        self.tables
            .iter_mut()
            .find(|table| table.name == table_name)
            .unwrap()
    }
    fn set_selected_table(&mut self, new_table: CoolTable) {
        match &self.selected_table {
            Some(selected_table) => {
                let pos = self
                    .tables
                    .iter()
                    .position(|table| &table.name == selected_table)
                    .unwrap();
                self.tables[pos] = new_table;
            }
            None => {}
        }
    }
    fn get_selected_table(&self) -> Option<&CoolTable> {
        self.selected_table.as_ref().and_then(|selected_table| {
            self.tables
                .iter()
                .find(|table| &table.name == selected_table)
        })
    }
    fn get_mut_selected_table(&mut self) -> Option<&mut CoolTable> {
        self.selected_table.as_ref().and_then(|selected_table| {
            self.tables
                .iter_mut()
                .find(|table| &table.name == selected_table)
        })
    }
    fn get_mut_selected_table_i(&mut self) -> Option<(usize, &mut CoolTable)> {
        self.selected_table.as_ref().and_then(|selected_table| {
            self.tables
                .iter_mut()
                .enumerate()
                .find(|(_, table)| &table.name == selected_table)
        })
    }
    fn get_names(&mut self) -> Vec<String> {
        self.tables
            .iter()
            .map(|table| table.name.clone())
            .collect::<Vec<_>>()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
// todo IMPORTANT is this rowid or row index?
// should be rowid, because it is in the name, and is consistent with using colname not id for DbEditor.current_cell
pub enum CurrentCellRowid {
    Row(i64),
    ColumnHeaders,
}

// #[derive(Default)]
pub struct DbEditor {
    schema: DbSchema,
    conn: Connection,
    text_output: Vec<OutputText>,
    tables: Tables,

    // state
    // selection
    // this is rowid, not index
    selected_rows: Vec<i64>,
    selected_columns: Vec<String>,
    show_schema: bool,
    show_schema2: bool,

    // reference selector
    show_reference_table: Option<String>,
    containing_row_index_for_row_being_edited: Option<usize>,
    containing_rowid_for_row_being_edited: Option<i64>,
    col_name_for_ref_being_chosen: Option<String>,
    reference_id_being_edited: Option<Value>,

    // alter table
    create_table_name: String,
    create_column_name: String,
    create_column_data_type: Type,
    create_column_nullability: Nullability,
    create_column_reference: Option<MyAstForeignKey>,
}

#[derive(Debug, Clone)]
pub enum Message {
    // system
    CopyToClipboard(String),
    Event(iced_native::Event),

    // DbEditor
    TableSelected(Option<String>),
    DoNothing,
    SetColumnSortOrder {
        column_name: String,
        sort_order: SortOrder,
    },
    SetColumnWidth {
        column_name: String,
        width: u16,
    },
    ToggleSchema2,

    // TableEditor
    AddNewRow,
    DeleteSelectedRows,
    DeleteSelectedColumns,
    // ToggleRowSelected(Value),
    // ToggleColumnSelected(String),
    ClickCell(CurrentCellRowid, String),

    CellUpdated {
        row_index: Option<usize>,
        where_value: Option<i64>,
        col_name: String,
        new_value: Value,
    },
    OpenReferenceTable {
        containing_row_index: Option<usize>,
        containing_rowid: Option<i64>,
        col_being_updated_name: String,
        previous_reference_row_id: Option<Value>,
        table_name: String,
    },
    CloseReferenceChooser,
    CreateTableNameUpdated(String),
    CreateTable,
    DropTable,
    CreateColumnNameUpdated(String),
    CreateColumnDataTypeUpdated(Type),
    CreateColumnNullabilityUpdated(Nullability),
    CreateColumnRefTableUpdated(Option<MyAstForeignKey>),
    CreateColumn,
    DoSomething,
}

// impl Sandbox for Table {
impl Application for DbEditor {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = MyArgs;

    fn title(&self) -> String {
        String::from("db editor")
    }

    fn subscription(&self) -> Subscription<Message> {
        // iced_native::subscription::events().map(Message::Event)
        iced_native::subscription::events_with(|event, status| match status {
            Status::Ignored => match event {
                iced_native::Event::Keyboard(_) => Some(event),
                _ => None,
            },
            Status::Captured => None,
        })
        .map(Message::Event)
    }

    fn new(flags: MyArgs) -> (DbEditor, Command<Message>) {
        dbg!(&flags);
        // let db_path = "businesses.db";
        let db_path = "test.db";
        let conn = match flags.path {
            Some(path) => {
                if flags.schema {
                    let conn = Connection::open_in_memory().unwrap();
                    let schema_string = fs::read_to_string(path).unwrap();
                    conn.execute_batch(&format!("BEGIN; {schema_string} COMMIT;"))
                        .unwrap();
                    conn
                } else {
                    Connection::open(path).unwrap()
                }
            }
            None => Connection::open_in_memory().unwrap(),
        };
        // todo PRAGMA foreign_keys = ON;
        // should first arg be None?
        conn.pragma_update(Some(MAIN_DB), "foreign_keys", 1_i64)
            .unwrap();

        let mut schema_strings: Vec<String> = Vec::new();
        {
            let mut stmt = conn
                .prepare(
                    "SELECT sql FROM sqlite_schema
                        ORDER BY tbl_name, type DESC, name;",
                )
                .unwrap();
            let mut rows = stmt.query([]).unwrap();

            while let Some(row) = rows.next().unwrap() {
                schema_strings.push(row.get(0).unwrap());
            }
        }

        let schema = DbSchema::new(schema_strings);
        let mut table_names: Vec<String> = schema.ast.my_table_names();
        let selected_table: Option<&String> = table_names.get(0);

        // todo for now just read all tables, but in future only read tables as they are used, and drop data after say 50 tables?
        let mut tables: Vec<CoolTable> = schema
            .ast
            .iter()
            .map(|table_schema| {
                CoolTable::from_schema(&conn, &table_schema.my_table_name(), &table_schema)
            })
            .collect::<Vec<_>>();
        // need to add new_row_inputs2 now that we have all the data
        let tables2 = tables.clone();
        tables.iter_mut().for_each(|table| {
            table.new_row_inputs2 = table
                .columns
                .iter()
                .map(|column| {
                    if column.notnull {
                        match column.data_type {
                            Type::Integer => {
                                // if col is not nullable, need to get an actual value from referenced table. assumes at least one exists
                                // todo should fail gracefully if it doesn't
                                if column.fk.is_some() {
                                    // todo don't get ref pk, get actual referenced column like:
                                    let MyAstForeignKey {
                                        table_name,
                                        column_name,
                                    } = column.fk.as_ref().unwrap();
                                    tables2
                                        .iter()
                                        .find(|table| &table.name == table_name)
                                        .unwrap()
                                        .columns
                                        .iter()
                                        .find(|col| &col.name == column_name)
                                        .unwrap()
                                        .data[0]
                                        .clone()
                                    // reference_data[0].get_pk_value(0)
                                } else {
                                    column.data_type.default_value()
                                }
                            }
                            Type::Real => column.data_type.default_value(),
                            Type::Text => column.data_type.default_value(),
                            _ => {
                                panic!("different data type")
                            }
                        }
                    } else {
                        // nullable columns default to NULL
                        Value::Null
                    }
                })
                .collect::<Vec<_>>();
        });

        // get reference table names for initially selected table
        let editable_data: Option<&CoolTable> =
            tables.iter().find_map(|table| match selected_table {
                Some(selected_table) => {
                    if &table.name == selected_table {
                        Some(table)
                    } else {
                        None
                    }
                }
                None => None,
            });
        let reference_col_names = match editable_data {
            Some(editable_data) => editable_data
                .columns
                .iter()
                .filter_map(|col| col.fk.as_ref().map(|fk| &fk.table_name))
                .collect::<Vec<_>>(),
            None => vec![],
        };
        let reference_data_tables = &tables
            .iter()
            .filter(|table| reference_col_names.contains(&&table.name))
            .collect::<Vec<_>>();

        let reference_data = reference_data_tables
            .iter()
            .map(|tb| tb.name.clone())
            .collect::<Vec<_>>();

        // if a column is a reference then can't just set it to any default value, it needs to be one which exists in the reference table

        let tables = Tables {
            selected_table: selected_table.map(|x| x.clone()),
            tables,
            reference_data,
        };

        (
            Self {
                schema,
                conn,
                text_output: vec![OutputText::Other("Loaded business.db".to_string())],
                tables,

                // Table_editor
                show_schema: false,
                show_schema2: false,

                selected_rows: Default::default(),
                selected_columns: Default::default(),

                show_reference_table: Default::default(),
                containing_rowid_for_row_being_edited: Default::default(),
                containing_row_index_for_row_being_edited: Default::default(),
                col_name_for_ref_being_chosen: Default::default(),
                reference_id_being_edited: Default::default(),
                create_table_name: Default::default(),
                create_column_name: Default::default(),
                create_column_data_type: Type::Integer,
                create_column_nullability: Nullability::Nullable,
                create_column_reference: None,
                // ..Self::default()
            },
            Command::none(),
        )
    }

    fn update(&mut self, message: Message) -> Command<Self::Message> {
        match message {
            Message::CopyToClipboard(val) => clipboard::write(val.clone()),
            Message::Event(ev) => {
                // dbg!(&ev);
                let selected_table = self.tables.get_mut_selected_table().unwrap();
                match ev {
                    iced_native::Event::Keyboard(kbe) => {
                        match kbe {
                            // A keyboard key was pressed.
                            keyboard::Event::KeyPressed {
                                // The key identifier
                                key_code,

                                // The state of the modifier keys
                                modifiers,
                            } => match key_code {
                                KeyCode::Left => {
                                    if modifiers.shift() {
                                        if selected_table.current_cell.1 != "rowid".to_string() {
                                            let current_col_index = selected_table
                                                .columns
                                                .iter()
                                                .enumerate()
                                                .find_map(|(i, cola)| {
                                                    if cola.name == selected_table.current_cell.1 {
                                                        Some(i)
                                                    } else {
                                                        None
                                                    }
                                                })
                                                .unwrap();
                                            if current_col_index != 0 {
                                                let row_index = match selected_table.current_cell.0
                                                {
                                                    CurrentCellRowid::Row(rowid) => {
                                                        selected_table.get_row_index(rowid)
                                                    }
                                                    CurrentCellRowid::ColumnHeaders => 0,
                                                };
                                                let col_index = if selected_table.current_cell.1
                                                    == "rowid".to_string()
                                                {
                                                    0
                                                } else {
                                                    selected_table
                                                        .columns
                                                        .iter()
                                                        .enumerate()
                                                        .find_map(|(i, col)| {
                                                            if col.name
                                                                == selected_table.current_cell.1
                                                            {
                                                                Some(i)
                                                            } else {
                                                                None
                                                            }
                                                        })
                                                        .unwrap()
                                                        + 1
                                                };
                                                selected_table.selected_cells[row_index]
                                                    [col_index] = true;
                                                selected_table.current_cell.1 = selected_table
                                                    .columns[current_col_index - 1]
                                                    .name
                                                    .clone();
                                            }
                                        }
                                    } else {
                                        selected_table
                                            .selected_cells
                                            .iter_mut()
                                            .for_each(|row| row.fill(false));
                                        if selected_table.current_cell.1 != "rowid".to_string() {
                                            let current_col_index = selected_table
                                                .columns
                                                .iter()
                                                .enumerate()
                                                .find_map(|(i, cola)| {
                                                    if cola.name == selected_table.current_cell.1 {
                                                        Some(i)
                                                    } else {
                                                        None
                                                    }
                                                })
                                                .unwrap();
                                            if current_col_index == 0 {
                                                selected_table.current_cell.1 = "rowid".to_string();
                                            } else {
                                                selected_table.current_cell.1 = selected_table
                                                    .columns[current_col_index - 1]
                                                    .name
                                                    .clone();
                                            }
                                        }
                                    }
                                }
                                KeyCode::Right => {
                                    selected_table
                                        .selected_cells
                                        .iter_mut()
                                        .for_each(|row| row.fill(false));
                                    if selected_table.current_cell.1 == "rowid".to_string() {
                                        selected_table.current_cell.1 =
                                            selected_table.columns[0].name.clone();
                                    } else if selected_table.current_cell.1
                                        != selected_table.columns.last().unwrap().name
                                    {
                                        let current_col_index = selected_table
                                            .columns
                                            .iter()
                                            .enumerate()
                                            .find_map(|(i, cola)| {
                                                if cola.name == selected_table.current_cell.1 {
                                                    Some(i)
                                                } else {
                                                    None
                                                }
                                            })
                                            .unwrap();
                                        selected_table.current_cell.1 = selected_table.columns
                                            [current_col_index + 1]
                                            .name
                                            .clone();
                                    }
                                }
                                KeyCode::Up => match selected_table.current_cell.0 {
                                    CurrentCellRowid::Row(rowid) => {
                                        let index = selected_table
                                            .rowid_column
                                            .iter()
                                            .enumerate()
                                            .find_map(|(i, rowid2)| {
                                                if rowid as i64 == *rowid2 {
                                                    Some(i)
                                                } else {
                                                    None
                                                }
                                            })
                                            .unwrap();
                                        if index == 0 {
                                            selected_table.current_cell.0 =
                                                CurrentCellRowid::ColumnHeaders
                                        } else {
                                            selected_table.current_cell.0 = CurrentCellRowid::Row(
                                                selected_table.rowid_column[index - 1],
                                            )
                                        }
                                    }
                                    CurrentCellRowid::ColumnHeaders => {}
                                },
                                KeyCode::Down => match selected_table.current_cell.0 {
                                    CurrentCellRowid::Row(rowid) => {
                                        let index = selected_table
                                            .rowid_column
                                            .iter()
                                            .enumerate()
                                            .find_map(|(i, rowid2)| {
                                                if rowid as i64 == *rowid2 {
                                                    Some(i)
                                                } else {
                                                    None
                                                }
                                            })
                                            .unwrap();
                                        if index < selected_table.rowid_column.len() - 1 {
                                            selected_table.current_cell.0 = CurrentCellRowid::Row(
                                                selected_table.rowid_column[index + 1],
                                            )
                                        }
                                    }
                                    CurrentCellRowid::ColumnHeaders => {
                                        selected_table.current_cell.0 =
                                            CurrentCellRowid::Row(selected_table.rowid_column[0])
                                    }
                                },
                                _ => {}
                            },

                            // A keyboard key was released.
                            keyboard::Event::KeyReleased {
                                /// The key identifier
                                key_code,

                                /// The state of the modifier keys
                                modifiers,
                            } => {}

                            // A unicode character was received.
                            keyboard::Event::CharacterReceived(char) => {}

                            // The keyboard modifiers have changed.
                            keyboard::Event::ModifiersChanged(modifiers) => {}
                        }
                    }
                    _ => {}
                };
                Command::none()
            }
            // read each table (and reference table) each time the table is selected (ie refresh data)
            Message::SetColumnSortOrder {
                column_name,
                sort_order,
            } => {
                let (i, selecta) = self.tables.get_mut_selected_table_i().unwrap();
                selecta
                    .sort_order
                    .retain(|(col_name, _)| col_name != &column_name);
                if let SortOrder::Asc | SortOrder::Desc = sort_order {
                    selecta.sort_order.push((column_name, sort_order));
                }
                let selected_name = selecta.name.clone();
                let (sql_output, new_table) = selecta.reload_column_data_from_schema(
                    &self.conn,
                    &selected_name,
                    self.schema
                        .ast
                        .iter()
                        .find(|statement| statement.my_table_name() == selected_name.clone())
                        .unwrap(),
                );
                self.tables.tables[i] = new_table;
                self.text_output.push(OutputText::SucessfulSql(sql_output));

                Command::none()
            }
            Message::SetColumnWidth { column_name, width } => {
                let (i, selecta) = self.tables.get_mut_selected_table_i().unwrap();
                let mycol = selecta
                    .columns
                    .iter_mut()
                    .find(|col| col.name == column_name)
                    .unwrap();
                mycol.width = width;
                let selected_name = selecta.name.clone();

                Command::none()
            }

            Message::TableSelected(selected_table) => {
                self.tables.selected_table = selected_table;
                Command::none()
            }
            Message::ToggleSchema2 => {
                self.show_schema2 = !self.show_schema2;
                Command::none()
            }

            // should we have a separate save button to confirm changes rather than do them auto? this isn't how easy tools like gsheets work... but should at least have a buffer that only writes every second or so maybe, in case writing on every keystroke kills the performance? need to test this
            // row id here is actual just the vec index
            // should use pk if it exists for where clauses, otherwise use rowid, or just use rowid anyway? Or just say that the whole table is replaced with in memory data unless a primary key exists?
            Message::CellUpdated {
                row_index,
                where_value,
                col_name,
                new_value,
            } => {
                // if row_index is some then so is where_value
                match row_index {
                    Some(row_index) => {
                        // todo this has id hard coded where it should find the pk name
                        // todo shouldn't new_value be parsed to a Value before being sent, since in some for reference selection it is already a value? remember this is called on every keystroke so should be optimised for text cells. if we parsed them beforehand, we would the need to parse back to a string for the sql anyway. but it does feel less clean this way. and I plan to batch the sql stuff eventually anyway. and remember Value::Text() is probs basically just a pointer, so not doing any allocation if we wrap the string in it then take the string back out
                        let sql = format!(
                            // "update {} set {} = ? where id = ?;",
                            "UPDATE \"{}\" SET \"{}\" = ? WHERE rowid = ?;",
                            self.tables.selected_table.as_ref().unwrap(),
                            col_name,
                        );
                        let output = match self.conn.prepare(&sql) {
                            Ok(mut stmt) => {
                                // todo impl ToSql
                                let where_value = where_value.unwrap();
                                let bad_params3 = vec![
                                    new_value.to_sql().unwrap(),
                                    where_value.to_sql().unwrap(),
                                ];
                                let params2 = bad_params3
                                    .iter()
                                    .map(|x| x as &dyn ToSql)
                                    .collect::<Vec<_>>();
                                match stmt.execute(&params2[..]) {
                                    Ok(_) => {
                                        let bound_sql_string = stmt.expanded_sql().unwrap();
                                        OutputText::SucessfulSql(bound_sql_string)
                                    }
                                    Err(err) => OutputText::Error(err.to_string()),
                                }
                            }
                            Err(err) => OutputText::Error(err.to_string()),
                        };

                        let editable_data = self.tables.get_mut_selected_table().unwrap();
                        let (col_index, my_col) = editable_data
                            .columns
                            .iter_mut()
                            .enumerate()
                            .find(|(i, col)| col.name == col_name)
                            .unwrap();
                        my_col.data[row_index] = new_value;

                        self.add_sql_string(output);
                    }
                    None => {
                        let editable_data = self.tables.get_mut_selected_table().unwrap();
                        let (col_index, my_col) = editable_data
                            .columns
                            .iter_mut()
                            .enumerate()
                            .find(|(i, col)| col.name == col_name)
                            .unwrap();
                        editable_data.new_row_inputs2[col_index] = new_value;
                    }
                };

                self.reference_id_being_edited = None;
                self.containing_row_index_for_row_being_edited = None;
                self.containing_rowid_for_row_being_edited = None;
                self.show_reference_table = None;
                Command::none()
            }

            Message::DeleteSelectedRows => {
                let sql = format!(
                    "DELETE FROM \"{}\" WHERE rowid IN ({});",
                    self.tables.selected_table.clone().unwrap(),
                    self.selected_rows
                        .iter()
                        .map(|selected_row| "?")
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                let output = match self.conn.prepare(&sql) {
                    Ok(mut stmt) => {
                        let bad_params3 = self
                            .selected_rows
                            .iter()
                            .map(|selected_row| selected_row.to_sql().unwrap())
                            .collect::<Vec<_>>();
                        let params2 = bad_params3
                            .iter()
                            .map(|x| x as &dyn ToSql)
                            .collect::<Vec<_>>();
                        match stmt.execute(&params2[..]) {
                            Ok(_) => {
                                let bound_sql_string = stmt.expanded_sql().unwrap();
                                OutputText::SucessfulSql(bound_sql_string)
                            }
                            Err(err) => OutputText::Error(err.to_string()),
                        }
                    }
                    Err(err) => OutputText::Error(err.to_string()),
                };

                // update local data
                // assuming first column exists because SQLite tables must always have atleast 1 colum
                let editable_data = self.tables.get_selected_table().unwrap();
                let (output2, new_table) = editable_data.reload_everything_from_schema(
                    &self.conn,
                    &editable_data.name,
                    &self.schema.ast.my_get_table_schema(&editable_data.name),
                    &self.tables.tables,
                );
                self.tables.set_selected_table(new_table);

                self.selected_rows.clear();
                self.add_sql_string(output);
                // self.add_sql_string(output2);
                Command::none()
            }
            Message::DeleteSelectedColumns => {
                // experienced a bug where I after deleting a column(s?) I ended up with a malformed sqlite schema (why does sqlite allow this!!!) like create table poop (id Integer, cool Text, Integer);
                let sql = self
                    .selected_columns
                    .iter()
                    .map(|col_name| {
                        format!(
                            "ALTER TABLE \"{}\" DROP \"{}\";",
                            self.tables.selected_table.as_ref().unwrap(),
                            col_name,
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                let output = match self.conn.execute_batch(&sql) {
                    Ok(()) => OutputText::SucessfulSql(sql),
                    Err(err) => OutputText::Error(err.to_string()),
                };
                // must reload schema every time we change it!
                self.reload_schema();

                // self.get_mut_selected_table()
                // todo break .tables and .selected table out into a separate type that the below can be implemented on
                // update local data
                let editable_data = self.tables.get_selected_table().unwrap();
                let (output2, new_table) = editable_data.reload_everything_from_schema(
                    &self.conn,
                    &editable_data.name,
                    &self.schema.ast.my_get_table_schema(&editable_data.name),
                    &self.tables.tables,
                );
                self.tables.set_selected_table(new_table);
                // self.tables
                //     .tables
                //     .iter_mut()
                //     .find(|table| &table.name == self.tables.selected_table.as_ref().unwrap())
                //     .unwrap()
                //     // NOTE can't do this because the indexes are no longer valid after you remove a column! (was previously looping over columns and calling col.remove(i))
                //     // https://users.rust-lang.org/t/removing-multiple-indices-from-a-vector/65599
                //     .columns
                //     .retain(|col| !self.selected_columns.contains(&col.name));

                self.selected_columns.clear();
                self.reload_schema();

                self.add_sql_string(output);
                Command::none()
            }

            Message::AddNewRow => {
                // shouldn't hardcode NULL here, should determine whether we are auto incrementing and add it to self.new_row_inputs2
                let editable_data = self.tables.get_mut_selected_table().unwrap();
                let sql = format!(
                    "INSERT INTO \"{}\" VALUES ({});",
                    editable_data.name,
                    // TODO assumes that empty string should be NULL. need UI to allow differentiation between empty string and NULL
                    // TODO do I need to/I should wrap text in "" for safety...
                    editable_data
                        .new_row_inputs2
                        .iter()
                        .map(|_| "?")
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                let output = match self.conn.prepare(&sql) {
                    Ok(mut stmt) => {
                        let params2 = editable_data
                            .new_row_inputs2
                            .iter()
                            .map(|x| x as &dyn ToSql)
                            .collect::<Vec<_>>();
                        match stmt.insert(&params2[..]) {
                            Ok(rowid) => {
                                // update local data
                                let editable_data = self.tables.get_selected_table().unwrap();
                                let (output2, new_table) = editable_data
                                    .reload_everything_from_schema(
                                        &self.conn,
                                        &editable_data.name,
                                        &self.schema.ast.my_get_table_schema(&editable_data.name),
                                        &self.tables.tables,
                                    );
                                self.tables.set_selected_table(new_table);

                                let bound_sql_string = stmt.expanded_sql().unwrap();
                                OutputText::SucessfulSql(bound_sql_string)
                            }
                            Err(err) => OutputText::Error(err.to_string()),
                        }
                    }
                    Err(err) => {
                        // let bound_sql_string = stmt.expanded_sql().unwrap();
                        let sqlite_error = err.sqlite_error().unwrap();

                        OutputText::Error(format!(
                            "err.to_string(): {}; &sql: {}; self
                            .new_row_inputs2: {:?};",
                            err.to_string(),
                            sql,
                            editable_data.new_row_inputs2
                        ))
                    }
                };

                self.add_sql_string(output);
                Command::none()
            }

            Message::ClickCell(row, col) => {
                let selected_table = self.tables.get_mut_selected_table().unwrap();

                // updated selected cells
                let row_index = match selected_table.current_cell.0 {
                    CurrentCellRowid::Row(rowid) => selected_table.get_row_index(rowid),
                    CurrentCellRowid::ColumnHeaders => 0,
                };
                let col_index = if selected_table.current_cell.1 == "rowid".to_string() {
                    0
                } else {
                    selected_table
                        .columns
                        .iter()
                        .enumerate()
                        .find_map(|(i, col)| {
                            if col.name == selected_table.current_cell.1 {
                                Some(i)
                            } else {
                                None
                            }
                        })
                        .unwrap()
                        + 1
                };
                selected_table
                    .selected_cells
                    .iter_mut()
                    .for_each(|row| row.fill(false));
                selected_table.selected_cells[row_index][col_index] = true;

                // update current cell
                selected_table.current_cell.0 = row;
                selected_table.current_cell.1 = col;
                Command::none()
            }

            Message::OpenReferenceTable {
                containing_row_index,
                containing_rowid: containing_row_id,
                col_being_updated_name,
                previous_reference_row_id,
                table_name,
            } => {
                self.containing_row_index_for_row_being_edited = containing_row_index;
                self.containing_rowid_for_row_being_edited = containing_row_id;
                // don't need this now but will be useful for highlighting currently selected row
                self.reference_id_being_edited = previous_reference_row_id;
                self.show_reference_table = Some(table_name);
                self.col_name_for_ref_being_chosen = Some(col_being_updated_name);
                Command::none()
            }
            Message::CloseReferenceChooser => {
                self.containing_row_index_for_row_being_edited = None;
                self.containing_rowid_for_row_being_edited = None;
                self.reference_id_being_edited = None;
                self.show_reference_table = None;
                self.col_name_for_ref_being_chosen = None;
                Command::none()
            }

            Message::CreateTableNameUpdated(val) => {
                self.create_table_name = val;
                Command::none()
            }
            Message::CreateTable => {
                // SQLite cannot create an empty table https://stackoverflow.com/questions/4567180/sqlite3-creating-table-with-no-columns
                // SQLite does not allow parameters for table names or column names https://www.sqlite.org/cintro.html (chapter 6)
                let sql = format!(
                    "CREATE TABLE \"{}\" ( id INTEGER PRIMARY KEY );",
                    self.create_table_name
                );
                let output = match self.conn.execute(&sql, []) {
                    Ok(_) => OutputText::SucessfulSql(sql),
                    Err(err) => OutputText::Error(err.to_string()),
                };

                self.add_sql_string(output);
                self.reload_schema();
                // todo don't need to load all tables, just push the new one
                // self.tables.tables = self.load_tables();
                // todo this is local manipulation which we were avoiding and should maybe have a method which just reloads all tables? yes plus it assumes the table was created successfully and will panic if it wasn't and can't find it in the schema
                self.tables.tables.push(CoolTable::from_schema_with_newrow(
                    &self.conn,
                    &self.create_table_name,
                    &self.schema.ast.my_get_table_schema(&self.create_table_name),
                    &self.tables.tables,
                ));
                self.create_table_name = "".to_string();
                Command::none()
            }

            Message::DropTable => {
                let sql = format!(
                    "DROP TABLE \"{}\";",
                    self.tables.selected_table.as_ref().unwrap()
                );
                let output = match self.conn.execute(&sql, []) {
                    Ok(_) => OutputText::SucessfulSql(sql),
                    Err(err) => OutputText::Error(err.to_string()),
                };

                // self.create_table_name = "".to_string();
                self.add_sql_string(output);
                self.reload_schema();
                // need to load new table because current no longer exists
                // self.tables.tables = self.load_tables();
                // todo this is local manipulation which we were avoiding and should maybe have a method which just reloads all tables?
                self.tables
                    .tables
                    .retain(|table| &table.name != self.tables.selected_table.as_ref().unwrap());
                Command::none()
            }
            Message::CreateColumnNameUpdated(val) => {
                self.create_column_name = val;
                self.reload_schema();
                Command::none()
            }
            Message::CreateColumn => {
                let (turn_of_pragma, sql) = {
                    let (turn_of_pragma, nullability_and_reference) = match &self
                        .create_column_reference
                    {
                        Some(MyAstForeignKey {
                            table_name,
                            column_name,
                        }) => match self.create_column_nullability {
                            Nullability::Nullable => (
                                false,
                                format!(
                                    " REFERENCES \"{table_name}\"(\"{column_name}\") DEFAULT NULL"
                                ),
                            ),
                            Nullability::NotNull => {
                                // todo need to get a valid default value from reference table. what if table is empty? then don't show it in list. should we assume all refs point to primary key so only show ref tables with pks and auto select pk as column? yes, for now.

                                // assumption ref table has a pk and > 0 rows
                                let default_value = self
                                    .tables
                                    .get_table(&table_name)
                                    .get_pk_value(0)
                                    .my_to_sql();

                                // [x] todo this works but is probably better to use a prepared statement. nope - can't use placehodlers for default values
                                // rustfmt gives up on the following line. maybe it is too long? it works fines with inline var format! above
                                (true, format!(" NOT NULL REFERENCES \"{table_name}\"(\"{column_name}\") DEFAULT {default_value}"))
                            }
                        },
                        None => match self.create_column_nullability {
                            Nullability::Nullable => (false, "".to_string()),
                            Nullability::NotNull => {
                                let default_value = self.create_column_data_type.default_value();

                                // [x] todo this works but is probably better to use a prepared statement. nope - can't use placehodlers for default values
                                let default_value = default_value.my_to_sql();

                                (false, format!(" NOT NULL DEFAULT {}", default_value))
                            }
                        },
                    };
                    (
                        turn_of_pragma,
                        format!(
                            "ALTER TABLE \"{}\" ADD \"{}\" {}{};",
                            self.tables.selected_table.as_ref().unwrap(),
                            self.create_column_name.clone(),
                            self.create_column_data_type,
                            nullability_and_reference
                        ),
                    )
                };

                if turn_of_pragma {
                    self.add_sql_string(OutputText::SucessfulSql(
                        "PRAGMA foreign_keys = 0;".to_string(),
                    ));
                    self.conn
                        .pragma_update(Some(MAIN_DB), "foreign_keys", 0_i64)
                        .unwrap();
                }
                let output = match self.conn.execute(&sql, []) {
                    Ok(_) => OutputText::SucessfulSql(sql),
                    Err(err) => OutputText::Error(err.to_string()),
                };
                self.add_sql_string(output);
                if turn_of_pragma {
                    self.add_sql_string(OutputText::SucessfulSql(
                        "PRAGMA foreign_keys = 1;".to_string(),
                    ));
                    self.conn
                        .pragma_update(Some(MAIN_DB), "foreign_keys", 1_i64)
                        .unwrap();
                }

                // must reload schema every time we change it!
                self.reload_schema();

                // update local data
                // todo IMPORTANT below should not run if SQL fails
                // todo not a big deal but no point running below when there is an error above
                let editable_data = self.tables.get_selected_table().unwrap();
                let (output2, new_table) = editable_data.reload_everything_from_schema(
                    &self.conn,
                    &editable_data.name,
                    &self.schema.ast.my_get_table_schema(&editable_data.name),
                    &self.tables.tables,
                );
                self.tables.set_selected_table(new_table);

                self.create_column_name = "".to_string();

                Command::none()
            }
            Message::CreateColumnDataTypeUpdated(data_type) => {
                self.create_column_data_type = data_type;
                Command::none()
            }
            Message::CreateColumnNullabilityUpdated(nullability) => {
                self.create_column_nullability = nullability;
                Command::none()
            }
            Message::CreateColumnRefTableUpdated(fk) => {
                self.create_column_reference = fk;
                Command::none()
            }

            Message::DoSomething => Command::none(),
            Message::DoNothing => Command::none(),
        }
    }
    fn view(&self) -> Element<Message> {
        let table_selector = pick_list(
            self.tables
                .tables
                .iter()
                .map(|table| table.name.clone())
                .collect::<Vec<_>>(),
            self.tables.selected_table.clone(),
            |table_name| Message::TableSelected(Some(table_name)),
        );

        // Scrollable has .snap_to() but this doesn't seem to be implemented for pure
        let output_text = scrollable(Column::with_children(
            self.text_output
                .iter()
                .rev()
                .map(|text2| {
                    match text2 {
                        OutputText::SucessfulSql(output_text) => {
                            text(output_text).color(Color::from_rgb(0.75, 0.7, 0.))
                        }
                        OutputText::Other(output_text) => text(output_text).color(Color::BLACK),
                        OutputText::Error(output_text) => {
                            text(output_text).color(Color::from_rgb(1., 0., 0.))
                        }
                    }
                    .into()
                })
                .collect::<Vec<_>>(),
        ))
        .height(Length::Units(100))
        .into();

        // match &self.editable_data {
        let table_editor =
            match self.tables.get_selected_table() {
                Some(editable_data) => {
                    let pre_col_style = if editable_data.current_cell.0
                        == CurrentCellRowid::ColumnHeaders
                        && editable_data.current_cell.1 == "rowid".to_string()
                    {
                        CellTheme::CurrentCell
                    } else if editable_data.selected_cells[0][0] {
                        CellTheme::Highlight
                    } else {
                        CellTheme::Normal
                    };
                    fn pre_column_headers<'a>(style: CellTheme) -> Row<'a, Message> {
                        Row::with_children(vec![
                            cell_text("").width(Length::Units(50)).into(),
                            container(cell_text("rowid").width(Length::Units(50)))
                                .style(style)
                                .into(),
                        ])
                    }
                    let column_headers = editable_data.columns.iter().enumerate().fold(
                        Row::new(),
                        |acc, (i, col)| {
                            acc.push(
                                button(text(&col.name))
                                    // .on_press(Message::ToggleColumnSelected(col.name.clone()))
                                    .on_press(Message::ClickCell(
                                        CurrentCellRowid::ColumnHeaders,
                                        col.name.clone(),
                                    ))
                                    .style(
                                        if editable_data.current_cell.0
                                            == CurrentCellRowid::ColumnHeaders
                                            && editable_data.current_cell.1 == col.name.clone()
                                        {
                                            CellTheme::CurrentCell
                                        } else if editable_data.selected_cells[0][i] {
                                            CellTheme::Highlight
                                        } else {
                                            CellTheme::Normal
                                        },
                                    )
                                    .width(Length::Units(col.width)),
                            )
                        },
                    );
                    let column_headers = Row::with_children(vec![
                        pre_column_headers(pre_col_style).into(),
                        column_headers.into(),
                    ]);

                    let column_sorters = editable_data.columns.iter().enumerate().fold(
                        Row::new(),
                        |acc, (i, col)| {
                            acc.push(
                                pick_list(
                                    vec![SortOrder::None, SortOrder::Asc, SortOrder::Desc],
                                    editable_data
                                        .sort_order
                                        .iter()
                                        .find_map(|(col_name, sort_order)| {
                                            if col_name == &col.name {
                                                Some(sort_order.clone())
                                            } else {
                                                None
                                            }
                                        })
                                        .or(Some(SortOrder::None)),
                                    |sort_order| Message::SetColumnSortOrder {
                                        column_name: col.name.clone(),
                                        sort_order,
                                    },
                                )
                                .width(Length::Units(col.width)),
                            )
                        },
                    );
                    let column_sorters = Row::with_children(vec![
                        pre_column_headers(pre_col_style).into(),
                        column_sorters.into(),
                    ]);

                    let width_selector = editable_data.columns.iter().enumerate().fold(
                        Row::new(),
                        |acc, (i, col)| {
                            acc.push(
                                text_input(
                                    "ERROR: no placeholder",
                                    &col.width.to_string(),
                                    |width_string| Message::SetColumnWidth {
                                        column_name: col.name.clone(),
                                        width: width_string.parse::<u16>().unwrap(),
                                    },
                                )
                                .width(Length::Units(col.width)),
                            )
                        },
                    );
                    let width_selector = Row::with_children(vec![
                        pre_column_headers(pre_col_style).into(),
                        width_selector.into(),
                    ]);

                    let nullability_selector = editable_data.columns.iter().enumerate().fold(
                        Row::new(),
                        |acc, (i, col)| {
                            acc.push(
                                pick_list(
                                    vec![Nullability::Nullable, Nullability::NotNull],
                                    if col.notnull || col.is_pk {
                                        Some(Nullability::NotNull)
                                    } else {
                                        Some(Nullability::Nullable)
                                    },
                                    |nullability| Message::DoNothing,
                                )
                                .width(Length::Units(col.width)),
                            )
                        },
                    );
                    let nullability_selector = Row::with_children(vec![
                        pre_column_headers(pre_col_style).into(),
                        nullability_selector.into(),
                    ]);

                    let selector_and_rowid = vec![(50, "selector"), (50, "rowid")].iter().fold(
                        row(),
                        |acc, (width, name)| {
                            let width = *width;
                            acc.push(
                                editable_data.rowid_column.iter().fold(
                                    column().push(
                                        container(cell_text("").width(Length::Units(width)))
                                            .style(some_styles::LightBlueBackground),
                                    ),
                                    |acc, rowid| {
                                        let thing = if name == &"selector" {
                                            container(
                                                button("")
                                                    // .on_press(Message::ToggleRowSelected(
                                                    //     rowid.clone(),
                                                    // ))
                                                    .on_press(Message::ClickCell(
                                                        CurrentCellRowid::Row(*rowid),
                                                        "rowid".to_string(),
                                                    ))
                                                    .width(Length::Units(width))
                                                    .height(Length::Units(30)),
                                            )
                                        } else {
                                            container(
                                                cell_text(rowid.to_string())
                                                    .width(Length::Units(width)),
                                            )
                                        };
                                        let thing = thing.style(
                                            if match editable_data.current_cell.0 {
                                                CurrentCellRowid::ColumnHeaders => false,
                                                CurrentCellRowid::Row(rodi) => rodi == *rowid,
                                            } && editable_data.current_cell.1
                                                == "rowid".to_string()
                                            {
                                                CellTheme::CurrentCell
                                            } else if self.selected_rows.contains(rowid) {
                                                CellTheme::Highlight
                                            } else {
                                                CellTheme::Normal
                                            },
                                        );
                                        acc.push(thing)
                                    },
                                ),
                            )
                        },
                    );
                    let cells = editable_data.columns.iter().enumerate().fold(
                        Row::new(),
                        |acc, (i, col)| {
                            // let column_theme = if self.selected_columns.contains(&col.name) {
                            //     CellTheme::Highlight
                            // } else {
                            //     CellTheme::Normal
                            // };

                            acc.push(match col.col_type {
                                EditorColumnType::Pk => col.data.iter().enumerate().fold(
                                    column().push(
                                        self.make_editable(
                                            None,
                                            i,
                                            &editable_data.new_row_inputs2[i],
                                            col,
                                        )
                                        .style(some_styles::LightBlueBackground),
                                    ),
                                    |acc, (numi, vali)| {
                                        acc.push(self.make_editable(Some(numi), i, vali, col))
                                    },
                                ),
                                EditorColumnType::Editable => col.data.iter().enumerate().fold(
                                    column().push(
                                        self.make_editable(
                                            None,
                                            i,
                                            &editable_data.new_row_inputs2[i],
                                            col,
                                        )
                                        .style(some_styles::LightBlueBackground),
                                    ),
                                    |acc, (numi, vali)| {
                                        acc.push(self.make_editable(Some(numi), i, vali, col))
                                    },
                                ),
                                EditorColumnType::Fk => {
                                    // todo
                                    let ref_data = self
                                        .tables
                                        .tables
                                        .iter()
                                        .find(|table| {
                                            table.name == col.fk.as_ref().unwrap().table_name
                                        })
                                        .unwrap();
                                    col.data.iter().enumerate().fold(
                                        column().push(
                                            self.make_fk_container(
                                                None,
                                                i,
                                                &editable_data.new_row_inputs2[i],
                                                col,
                                            )
                                            .style(some_styles::LightBlueBackground),
                                        ),
                                        |acc, (numi, vali)| {
                                            acc.push(self.make_fk_container(
                                                Some(numi),
                                                i,
                                                vali,
                                                col,
                                            ))
                                        },
                                    )
                                }
                            })
                        },
                    );

                    let final_content: Element<_> = if self.show_reference_table.is_some() {
                        let ref_data = self
                            .tables
                            .tables
                            .iter()
                            .find(|table| {
                                &table.name == self.show_reference_table.as_ref().unwrap()
                            })
                            .unwrap();

                        let ref_chooser_table = ref_data.rows_iter().fold(
                            column().push(button("X").on_press(Message::CloseReferenceChooser)),
                            |acc, row_vec| {
                                let referencing_col = editable_data
                                    .columns
                                    .iter()
                                    .find(|col| {
                                        &col.name
                                            == self.col_name_for_ref_being_chosen.as_ref().unwrap()
                                    })
                                    .unwrap();
                                let fk_name = &referencing_col.fk.as_ref().unwrap().column_name;
                                let fk_pos = ref_data
                                    .columns
                                    .iter()
                                    .enumerate()
                                    .find(|(_, col)| &col.name == fk_name)
                                    .unwrap()
                                    .0;
                                let fk_value = row_vec[fk_pos].clone();

                                acc.push(
                                    button(text(row_vec.label(2)))
                                        .on_press(Message::CellUpdated {
                                            row_index: self
                                                .containing_row_index_for_row_being_edited,
                                            where_value: self
                                                .containing_rowid_for_row_being_edited
                                                .clone(),
                                            col_name: self
                                                .col_name_for_ref_being_chosen
                                                .clone()
                                                .unwrap(),
                                            new_value: fk_value,
                                        })
                                        .style(TransparentButtonStyle {})
                                        .width(Length::Units(200))
                                        .height(Length::Units(40)),
                                )
                            },
                        );
                        ref_chooser_table.into()
                    } else {
                        Column::with_children(vec![
                            column_headers.into(),
                            column_sorters.into(),
                            width_selector.into(),
                            nullability_selector.into(),
                            scrollable(Row::with_children(vec![
                                selector_and_rowid.into(),
                                cells.into(),
                            ]))
                            .into(),
                        ])
                        .into()
                    };
                    final_content.into()
                }
                None => text("no table").into(),
            };

        // iterate through tables to put them in an ordered tree structure.
        // remember we can have strucutres like:
        //         -
        // -   - - -   -
        // - - -     -   - -
        // -   - - -   -
        // -

        // find first tables
        let first_tables = self
            .tables
            .tables
            .iter()
            .filter(|table| table.has_fk())
            .collect::<Vec<_>>();
        let schema2 = self
            .tables
            .tables
            .iter()
            .fold(row(), |acc, table| {
                acc.push(table.columns.iter().fold(
                    column().push(text(table.name.as_str()).size(35)),
                    |acc2, col| {
                        acc2.push(
                            row()
                                .push(
                                    text(format!(
                                        "{}{}",
                                        if col.notnull || col.is_pk { "" } else { "?" },
                                        col.data_type.to_string()
                                    ))
                                    .width(Length::Units(100)),
                                )
                                .push(text(col.name.as_str()).size(25)),
                        )
                    },
                ))
            })
            .padding(30)
            .spacing(30);

        let add_table_controls = Row::with_children(vec![
            text_input(
                "table_name",
                &self.create_table_name,
                Message::CreateTableNameUpdated,
            )
            .width(Length::Units(200))
            .into(),
            button("add table").on_press(Message::CreateTable).into(),
        ]);
        let other_controls = self.other_controls();

        container(Column::with_children(vec![
            table_selector.into(),
            output_text,
            button("show schema2")
                .on_press(Message::ToggleSchema2)
                .into(),
            if self.show_schema {
                self.schema.view().map(move |message| Message::DoNothing)
            } else if self.show_schema2 {
                schema2.into()
            } else {
                // really header widths should be dynamic and depend on column contents, but that would probably need a custom table component

                Column::with_children(vec![
                    Row::with_children(vec![add_table_controls.into(), other_controls.into()])
                        .into(),
                    table_editor,
                ])
                .into()
            },
        ]))
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
    }
}

impl DbEditor {
    fn add_sql_string(&mut self, output_text: OutputText) {
        dbg!(&output_text);
        self.text_output.push(output_text);
    }

    fn other_controls(&self) -> Row<Message> {
        let my_data_types = vec![Type::Integer, Type::Real, Type::Text, Type::Blob];
        // .iter()
        // .map(|sql_type| sql_type.to_string())
        // .collect::<Vec<_>>();

        let other_controls = match self.tables.selected_table {
            Some(_) => Row::with_children(vec![
                button("add row").on_press(Message::AddNewRow).into(),
                button("delete row(s)")
                    .on_press(Message::DeleteSelectedRows)
                    .into(),
                button("delete table").on_press(Message::DropTable).into(),
                text_input(
                    "column_name",
                    &self.create_column_name,
                    Message::CreateColumnNameUpdated,
                )
                .width(Length::Units(200))
                .into(),
                pick_list(
                    my_data_types.clone(),
                    Some(self.create_column_data_type.clone()),
                    |data_type| Message::CreateColumnDataTypeUpdated(data_type),
                )
                .into(),
                // NOT NULL should just be a single checkbox, not a pick_list
                pick_list(
                    vec![Nullability::Nullable, Nullability::NotNull],
                    Some(self.create_column_nullability.clone()),
                    |nullability| Message::CreateColumnNullabilityUpdated(nullability),
                )
                .into(),
                pick_list(
                    self.tables
                        .tables
                        .iter()
                        .filter(|table| {
                            table.pk().is_some()
                                && match &self.create_column_nullability {
                                    Nullability::Nullable => true,
                                    Nullability::NotNull => table.columns[0].data.len() > 0,
                                }
                                && &table.name != self.tables.selected_table.as_ref().unwrap()
                        })
                        .map(|table| table.name.clone())
                        .chain(vec!["none".to_string()].iter().cloned())
                        .collect::<Vec<_>>(),
                    self.create_column_reference
                        .as_ref()
                        .map(|fk| fk.table_name.clone()),
                    |table_name| {
                        if table_name == "none" {
                            Message::CreateColumnRefTableUpdated(None)
                        } else {
                            let column_name = self
                                .tables
                                .get_table(&table_name)
                                .pk()
                                .unwrap()
                                .name
                                .clone();

                            Message::CreateColumnRefTableUpdated(Some(MyAstForeignKey {
                                table_name,
                                column_name,
                            }))
                        }
                    },
                )
                .into(),
                button("add column").on_press(Message::CreateColumn).into(),
                button("delete column(s)")
                    .on_press(Message::DeleteSelectedColumns)
                    .into(),
            ]),
            None => row().push(text("this database has no tables")),
        };
        other_controls
    }

    fn make_editable<'a>(
        &'a self,
        // None for new_row editor
        row_index: Option<usize>,
        col_index: usize,
        value: &'a Value,
        col: &'a CoolColumn,
    ) -> Container<'a, Message> {
        let editable_data = self.tables.get_selected_table().unwrap();
        let rowid = row_index.map(|row_index| editable_data.get_rowid(row_index));
        let master_message = move |new_value| Message::CellUpdated {
            row_index,
            // where_value: row_index.map(|row_index| editable_data.get_pk_value(row_index)),
            where_value: rowid.clone(),
            col_name: col.name.clone(),
            new_value,
        };

        let my_container = if col.notnull {
            // not null
            match value {
                Value::Integer(_) | Value::Real(_) | Value::Text(_) | Value::Null => container(
                    my_input(value.clone(), master_message)
                        .padding(5)
                        .style(TextboxStyle {}),
                ),
                Value::Blob(_) => container(cell_text("BLOB").width(Length::Units(col.width))),
            }
        } else {
            // nullable
            let null_text = cell_text("NULL")
                .width(Length::Units(col.width - 30))
                .horizontal_alignment(Horizontal::Center);
            let add_button = button("+")
                .on_press(master_message(col.data_type.default_value()))
                .width(Length::Units(30));

            let delete_button = button("x")
                .on_press(master_message(Value::Null))
                .width(Length::Units(30));

            match &value {
                Value::Null => container(row().push(null_text).push(add_button)),
                _ => {
                    // need to call this in here otherwise it will get called when we have Value::Null
                    container(
                        row()
                            .push(match value {
                                Value::Integer(_)
                                | Value::Real(_)
                                | Value::Text(_)
                                | Value::Null => container(
                                    my_input(value.clone(), master_message)
                                        .padding(5)
                                        .style(TextboxStyle {}),
                                )
                                // todo why do I need this here it was fine before I wrapped it in a container since the size is defined on the TextInput
                                .width(Length::Units(col.width - 30))
                                .height(Length::Units(30)),
                                Value::Blob(_) => {
                                    container(cell_text("BLOB").width(Length::Units(120)))
                                }
                            })
                            .push(delete_button),
                    )
                }
            }
        };

        // column_theme
        my_container
            .width(Length::Units(col.width))
            .height(Length::Units(30))
            .style(
                if match editable_data.current_cell.0 {
                    CurrentCellRowid::ColumnHeaders => false,
                    CurrentCellRowid::Row(rowid2) => match rowid {
                        Some(rowid) => rowid == rowid2,
                        None => false,
                    },
                } && editable_data.current_cell.1 == col.name
                {
                    CellTheme::CurrentCell
                } else if row_index.is_some()
                    && editable_data.selected_cells[match row_index {
                        Some(index) => index + 1,
                        None => 0,
                    }][col_index + 1]
                {
                    CellTheme::Highlight
                } else {
                    CellTheme::Normal
                },
            )
            .into()
    }

    fn make_fk_container<'a>(
        &'a self,
        // None for new_row editor
        row_index: Option<usize>,
        col_index: usize,
        value: &'a Value,
        col: &'a CoolColumn,
    ) -> Container<'a, Message> {
        let editable_data = self.tables.get_selected_table().unwrap();
        let rowid = row_index.map(|row_index| editable_data.get_rowid(row_index));
        let MyAstForeignKey {
            column_name: referenced_col_name,
            table_name: referenced_table_name,
        } = col.fk.as_ref().unwrap();
        let ref_data = self.tables.get_table(referenced_table_name);

        let message = Message::OpenReferenceTable {
            containing_row_index: row_index,
            // containing_row_id: numi.map(|numi| editable_data.get_pk_value(numi)),
            containing_rowid: row_index.map(|index| editable_data.get_rowid(index)),
            col_being_updated_name: col.name.clone(),
            previous_reference_row_id: Some(value.clone()),
            // table_name: editable_data
            //     .table_schema
            //     .my_get_ref_table_name(col.name.clone()),
            table_name: referenced_table_name.clone(),
        };

        let null_text = cell_text("NULL")
            .width(Length::Units(col.width - 30))
            .horizontal_alignment(Horizontal::Center);
        let add_button = button("+")
            .on_press(message.clone())
            .width(Length::Units(30));
        let delete_button = button("x")
            .on_press(Message::CellUpdated {
                row_index,
                // where_value: numi.map(|numi| editable_data.get_pk_value(numi)),
                where_value: row_index.map(|index| editable_data.get_rowid(index)),
                col_name: col.name.clone(),
                new_value: Value::Null,
            })
            .width(Length::Units(30));

        let container = match value {
            // todo
            Value::Integer(_) | Value::Text(_) | Value::Real(_) | Value::Blob(_) => {
                // todo this needs to take into account the referenced column in the referenced table. value is the value in that column. really we should find the rowid for value in the referenced column and pass that to .row_label(). but also the ref selector table creates labels from individual rows. so maybe row_label() should be passed a row, and we just extract the correct row here first, rather than the rowid

                // get rowid of referenced table row which fk value points to
                let fk_pos = ref_data
                    .columns
                    .iter()
                    .enumerate()
                    .find(|(i, col)| &col.name == referenced_col_name)
                    .unwrap()
                    .0;
                let rowid = ref_data
                    .rowid_column
                    .iter()
                    .zip(ref_data.rows_iter())
                    .find(|(rowid, row)| &row[fk_pos] == value)
                    .unwrap()
                    .0
                    .clone();

                // let selector = button(text("dsa".into()))
                let selector = button(text(ref_data.get_row(rowid).label(2)))
                    .on_press(message.clone())
                    .style(TransparentButtonStyle {})
                    .height(Length::Units(30));

                if col.notnull {
                    container(selector.width(Length::Units(col.width)))
                } else {
                    container(
                        row()
                            .push(selector.width(Length::Units(col.width - 30)))
                            .push(delete_button),
                    )
                }
            }
            Value::Null => container(row().push(null_text).push(add_button)),
        };

        container.style(
            if match editable_data.current_cell.0 {
                CurrentCellRowid::ColumnHeaders => false,
                CurrentCellRowid::Row(rowid2) => match rowid {
                    Some(rowid) => rowid == rowid2,
                    None => false,
                },
            } && editable_data.current_cell.1 == col.name
            {
                CellTheme::CurrentCell
            } else if row_index.is_some()
                && editable_data.selected_cells[match row_index {
                    Some(index) => index + 1,
                    None => 0,
                }][col_index + 1]
            {
                CellTheme::Highlight
            } else {
                CellTheme::Normal
            },
        )
    }

    // fn load_tables(&self) -> Vec<CoolTable> {
    //     self.schema
    //         .ast
    //         .iter()
    //         .map(|statement| {
    //             let table_name = statement.my_table_name();
    //             CoolTable::from_schema(&self.conn, &table_name, statement)
    //         })
    //         .collect::<Vec<_>>()
    //     // match table_name.clone() {
    //     //     Some(selected_table) => {
    //     //         let primary_table = Some(CoolTable::from_schema(
    //     //             &self.conn,
    //     //             selected_table.clone(),
    //     //             self.schema.ast.my_get_table_schema(selected_table),
    //     //         ));
    //     //         let reference_tables = self
    //     //             .schema
    //     //             .ast
    //     //             .my_get_reference_table_names(selected_table)
    //     //             .iter()
    //     //             .map(|table_name| {
    //     //                 CoolTable::from_schema(
    //     //                     &self.conn,
    //     //                     table_name,
    //     //                     self.schema.ast.my_get_table_schema(table_name),
    //     //                 )
    //     //             })
    //     //             .collect::<Vec<_>>();
    //     //         TableEditor::new(primary_table.clone(), reference_tables)
    //     //     }
    //     //     None => TableEditor::new(None, vec![]),
    //     // }
    // }

    fn reload_schema(&mut self) {
        let mut schema_strings: Vec<String> = Vec::new();
        {
            let mut stmt = self
                .conn
                .prepare(
                    "SELECT sql FROM sqlite_schema
                        ORDER BY tbl_name, type DESC, name;",
                )
                .unwrap();
            let mut rows = stmt.query([]).unwrap();

            while let Some(row) = rows.next().unwrap() {
                schema_strings.push(row.get(0).unwrap());
            }
        }

        self.schema = DbSchema::new(schema_strings);
        // self.table_names = self.schema.ast.my_table_names();
        // update self.selected table if it not longer exists
        // todo better to deafult to the previous table, not the first table
        // todo blocking below out, need to make sure self.tables is updated when necessary
        // if !self
        //     .table_names
        //     .contains(self.selected_table.as_ref().unwrap())
        // {
        //     self.selected_table = self.table_names.get(0).cloned();
        // }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SortOrder {
    None,
    Asc,
    Desc,
}

impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            SortOrder::None => f.pad("Not sorted"),
            SortOrder::Asc => f.pad("Ascending"),
            SortOrder::Desc => f.pad("Descenind"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Nullability {
    Nullable,
    NotNull,
}

impl fmt::Display for Nullability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Nullability::Nullable => f.pad("Nullable"),
            Nullability::NotNull => f.pad("NotNull"),
        }
    }
}

trait StringStuff {
    fn my_parse_int(&self) -> String;
    fn my_parse_real(&self) -> String;
}

impl StringStuff for String {
    fn my_parse_int(&self) -> String {
        self.chars()
            .filter(|char| char.is_numeric())
            .collect::<String>()
    }
    fn my_parse_real(&self) -> String {
        self.chars()
            .filter(|char| char.is_numeric() || *char == '.')
            .collect::<String>()
    }
}

trait MyType {
    fn default_value(&self) -> Value;
}

impl MyType for Type {
    fn default_value(&self) -> Value {
        match self {
            Type::Integer => Value::Integer(0),
            Type::Real => Value::Real(1.1),
            Type::Text => Value::Text("".to_string()),
            Type::Blob => Value::Blob(vec![]),
            _ => {
                panic!("should not be called on NULL (?)")
            }
        }
    }
}

fn integer_text_input<'a, Message>(
    value: i64,
    on_change: impl Fn(i64) -> Message + 'a,
) -> TextInput<'a, Message>
where
    Message: Clone,
{
    TextInput::new("", &value.to_string(), move |new_value| {
        if new_value.is_empty() {
            // panic!("no no 0 please");
            on_change(0)
        } else {
            on_change(new_value.my_parse_int().parse().unwrap())
        }
    })
}

fn decimal_text_input<'a, Message>(
    value: f64,
    on_change: impl Fn(f64) -> Message + 'a,
) -> TextInput<'a, Message>
where
    Message: Clone,
{
    TextInput::new("", &value.to_string(), move |new_value| {
        if new_value.is_empty() {
            on_change(1.1)
        } else {
            on_change(new_value.my_parse_real().parse().unwrap())
        }
    })
}

fn my_input<'a, Message>(
    value: Value,
    on_change: impl Fn(Value) -> Message + 'a,
) -> TextInput<'a, Message>
where
    Message: Clone,
{
    match value {
        Value::Integer(val) => integer_text_input(val, move |val| on_change(Value::Integer(val))),
        Value::Text(val) => text_input("empty!!", &val, move |val| on_change(Value::Text(val))),
        Value::Real(val) => decimal_text_input(val, move |val| on_change(Value::Real(val))),
        _ => {
            panic!("fdsaf");
        }
    }
}

fn cell_text(text2: impl Into<String>) -> Text {
    text(text2)
        .height(Length::Units(30))
        .vertical_alignment(Vertical::Center)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellTheme {
    Normal,
    CurrentCell,
    Highlight,
}

impl CellTheme {
    pub const ALL: [CellTheme; 2] = [CellTheme::Normal, CellTheme::Highlight];
}

impl Default for CellTheme {
    fn default() -> CellTheme {
        CellTheme::Normal
    }
}

impl<'a> From<CellTheme> for Box<dyn container::StyleSheet + 'a> {
    fn from(theme: CellTheme) -> Self {
        match theme {
            CellTheme::Normal => MyContainerStyle {}.into(),
            CellTheme::CurrentCell => MyContainerStyleGreen {}.into(),
            CellTheme::Highlight => MyContainerStyleRed {}.into(),
        }
    }
}

#[derive(Clone)]
struct MyContainerStyle {}

impl container::StyleSheet for MyContainerStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: None,
            ..container::Style::default()
        }
    }
    // other methods in Stylesheet have a default impl
}

#[derive(Clone)]
struct MyContainerStyleGreen {}

impl container::StyleSheet for MyContainerStyleGreen {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(iced::Background::Color(colors::GREEN)),
            ..Default::default()
        }
    }
    // other methods in Stylesheet have a default impl
}

#[derive(Clone)]
struct MyContainerStyleRed {}

impl container::StyleSheet for MyContainerStyleRed {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(iced::Background::Color(colors::RED)),
            ..Default::default()
        }
    }
    // other methods in Stylesheet have a default impl
}

impl<'a> From<CellTheme> for Box<dyn button::StyleSheet + 'a> {
    fn from(theme: CellTheme) -> Self {
        match theme {
            CellTheme::Normal => CellButtonStyle {}.into(),
            CellTheme::CurrentCell => CellButtonStyleGreen {}.into(),
            CellTheme::Highlight => CellButtonStyleRed {}.into(),
        }
    }
}

#[derive(Clone)]
struct CellButtonStyle {}

impl button::StyleSheet for CellButtonStyle {
    fn active(&self) -> button::Style {
        button::Style {
            background: Some(iced::Background::Color(colors::LIGHT_GREY)),
            ..button::Style::default()
        }
    }
    // other methods in Stylesheet have a default impl
}

#[derive(Clone)]
struct CellButtonStyleGreen {}

impl button::StyleSheet for CellButtonStyleGreen {
    fn active(&self) -> button::Style {
        button::Style {
            background: Some(iced::Background::Color(colors::GREEN)),
            ..Default::default()
        }
    }
    // other methods in Stylesheet have a default impl
}

#[derive(Clone)]
struct CellButtonStyleRed {}

impl button::StyleSheet for CellButtonStyleRed {
    fn active(&self) -> button::Style {
        button::Style {
            background: Some(iced::Background::Color(colors::RED)),
            ..Default::default()
        }
    }
    // other methods in Stylesheet have a default impl
}

struct TransparentButtonStyle {}

impl button::StyleSheet for TransparentButtonStyle {
    fn active(&self) -> button::Style {
        button::Style {
            background: None,
            ..button::Style::default()
        }
    }
    // other methods in Stylesheet have a default impl
}

#[derive(Default)]
pub struct DbSchema {
    schema_strings: Vec<String>,
    ast: Vec<Statement>,
    schema_canvas: SchemaDiagram,
}

pub enum DbSchemaMessage {
    DoSomething,
}

impl DbSchema {
    fn new(schema_strings: Vec<String>) -> Self {
        // let sql_string = schema_strings.join("\n").replace("\'", "\"");
        let dialect = SQLiteDialect {};
        // let dialect = GenericDialect {};
        let ast = schema_strings
            .iter()
            .map(|sql_string| sql_string.replace("\'", "\""))
            .map(|sql_string| Parser::parse_sql(&dialect, &sql_string).unwrap().remove(0))
            .collect::<Vec<_>>();

        let schema_canvas = SchemaDiagram {
            ast: ast.clone(),
            borders: Borders::all(20.),
        };
        Self {
            schema_strings,
            ast,
            schema_canvas,
            ..Self::default()
        }
    }

    fn view(&self) -> Element<DbSchemaMessage> {
        let canvas: Element<_> = Canvas::new(&self.schema_canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        canvas.map(move |message| DbSchemaMessage::DoSomething)
    }
}

#[derive(Default)]
pub struct SchemaDiagram {
    ast: Vec<Statement>,
    borders: Borders,
}

pub enum SchemaDiagramMessage {
    DoSomething,
}

#[derive(Default, Clone)]
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

// assume
// all tables have pk named id
// all fks are named <reference_table_name>_id
#[derive(Debug, Clone)]
pub struct SchemaBox {
    name: String,
    // (name, NOT NULL)
    columns: Vec<(String, bool)>,
    // (reference_table_name, NOT NULL)
    reference: Option<(String, bool)>,
}

impl canvas::Program<SchemaDiagramMessage> for &SchemaDiagram {
    type State = ();

    fn draw(&self, state: &Self::State, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        // first assume size of canvas is large. probably want to calc neccessary size from number of tables then add scrolling.
        let mut frame = Frame::new(bounds.size());

        let chart_area_width = frame.width() - self.borders.left - self.borders.right;
        let chart_area_height = frame.height() - self.borders.top - self.borders.bottom;

        // draw table boxes

        let box_spacing = 40.;
        let box_padding = 20.;
        // let grid_width = chart_area_width / 4.;
        let grid_width = 350.;
        let grid_height = 250.;
        let box_width = grid_width - box_spacing * 2.;
        let box_height = grid_height - box_spacing * 2.;

        // eventually a better approach is probably to use the same types as the library but create a function that standardises, e.g. moves all table constraints to column constraints

        // add first box to list Vec<Vec<Table>>
        // scan through list and add any boxes that depend on it after it
        // do the same for the next boxes
        // do same going other way but add boxes to front of list?

        let mut mydata = self
            .ast
            .iter()
            .enumerate()
            .map(|(i, statement)| SchemaBox {
                name: match statement {
                    Statement::CreateTable { name, .. } => {
                        name.0[0].clone().value.clone().replace("\"", "")
                    }
                    _ => "Fuck".to_string(),
                },
                columns: match statement {
                    Statement::CreateTable { columns, .. } => columns
                        .iter()
                        // .filter(|x| !x.name.to_string().replace("\"", "").ends_with("id"))
                        .filter(|x| x.name.to_string().replace("\"", "") != "id".to_string())
                        .map(|x| {
                            (
                                x.name.to_string().replace("\"", ""),
                                x.options.iter().any(|op| match op.option {
                                    ColumnOption::NotNull => true,
                                    _ => false,
                                }),
                            )
                        })
                        .collect::<Vec<_>>(),
                    _ => vec![("Fuck".to_string(), false)],
                },
                reference: match statement {
                    Statement::CreateTable { constraints, .. } => {
                        let myting = constraints.iter().find(|x| match x {
                            TableConstraint::ForeignKey { .. } => true,
                            _ => false,
                        });
                        match myting {
                            Some(constraint) => match constraint {
                                TableConstraint::ForeignKey { foreign_table, .. } => {
                                    // TODO need to get not null from columns...
                                    Some((foreign_table.to_string().replace("\"", ""), false))
                                }
                                _ => None,
                            },
                            None => None,
                        }
                    }
                    _ => Some(("Fuck".to_string(), false)),
                },
            })
            .collect::<Vec<_>>();

        let mut new_order = Vec::new();
        // do we still need to remove it from mydata?
        // first is addressx which has siblings, so need to start with something else
        // new_order.push(vec![mydata.remove(0)]);
        new_order.push(vec![mydata.remove(1)]);
        while new_order.iter().map(|x| x.len()).sum::<usize>() < mydata.len() {
            let new_order2 = new_order.clone();

            // need to iterate over whole vec, but just taking first element for now
            let first_box = new_order2.first().unwrap()[0].clone();
            let last_box = new_order2.last().unwrap()[0].clone();

            let mut first_vec = Vec::new();
            let mut last_vec = Vec::new();

            for k in 0..mydata.len() {
                println!(
                    "{}, {}",
                    last_box.name,
                    match mydata[k].reference.clone() {
                        Some(val) => val.0,
                        None => "none".to_string(),
                    }
                );
                // if box has ref to last box
                if match mydata[k].reference.clone() {
                    Some(val) => val.0 == last_box.name,
                    None => false,
                } {
                    last_vec.push(mydata[k].clone());
                }

                // if first box has ref to box
                if match first_box.reference.clone() {
                    Some(val) => val.0 == mydata[k].name,
                    None => false,
                } {
                    first_vec.push(mydata[k].clone());
                }
            }
            if first_vec.len() > 0 {
                new_order.insert(0, first_vec)
            }
            if last_vec.len() > 0 {
                new_order.push(last_vec);
            }
        }

        new_order
            .iter()
            .enumerate()
            .for_each(|(i, schema_box_vec)| {
                // vertical lines
                let black_stroke = Stroke {
                    width: 2.,
                    color: colors::BLUE,
                    line_cap: LineCap::Butt,
                    ..Stroke::default()
                };
                let grid_top_left =
                    Point::new(self.borders.left + grid_width * i as f32, self.borders.top);
                let mypath = Path::new(|builder| {
                    builder.move_to(Point::new(
                        grid_top_left.x,
                        grid_top_left.y + grid_height / 2.,
                    ));

                    builder.line_to(Point::new(
                        grid_top_left.x,
                        grid_top_left.y
                            + grid_height / 2.
                            + (schema_box_vec.len() - 1) as f32 * grid_height,
                    ));
                });
                frame.stroke(&mypath, black_stroke);

                schema_box_vec
                    .iter()
                    .enumerate()
                    .for_each(|(m, schema_box)| {
                        let grid_top_left = Point::new(
                            self.borders.left + grid_width * i as f32,
                            self.borders.top + grid_height * m as f32,
                        );
                        let box_top_left = Point::new(
                            grid_top_left.x + box_spacing,
                            grid_top_left.y + box_spacing,
                        );
                        // box
                        frame.fill_rectangle(
                            box_top_left,
                            Size::new(box_width, box_height),
                            colors::LIGHT_BLUE,
                        );

                        // lines
                        if i != 0 {
                            // horizontal
                            let mypath = Path::new(|builder| {
                                builder.move_to(Point::new(
                                    grid_top_left.x,
                                    grid_top_left.y + grid_height / 2.,
                                ));

                                builder.line_to(Point::new(
                                    grid_top_left.x + box_spacing,
                                    grid_top_left.y + grid_height / 2.,
                                ));
                            });
                            frame.stroke(&mypath, black_stroke);
                        }
                        if i != new_order.len() - 1 {
                            let mypath = Path::new(|builder| {
                                builder.move_to(Point::new(
                                    grid_top_left.x + box_spacing + box_width,
                                    grid_top_left.y + grid_height / 2.,
                                ));

                                builder.line_to(Point::new(
                                    grid_top_left.x + box_spacing * 2. + box_width,
                                    grid_top_left.y + grid_height / 2.,
                                ));
                            });
                            frame.stroke(&mypath, black_stroke);
                        }

                        // title
                        frame.fill_text(iced::canvas::Text {
                            content: schema_box.name.clone(),
                            position: Point::new(
                                box_top_left.x + box_padding,
                                box_top_left.y + box_padding,
                            ),
                            size: 30.,
                            vertical_alignment: Vertical::Center,
                            horizontal_alignment: Horizontal::Left,
                            ..Default::default()
                        });

                        // fields
                        schema_box
                            .columns
                            .iter()
                            .enumerate()
                            .for_each(|(j, column)| {
                                frame.fill_text(iced::canvas::Text {
                                    content: format!(
                                        "{} {}",
                                        if column.1 { "" } else { "?" },
                                        column.0
                                    ),
                                    position: Point::new(
                                        box_top_left.x + box_padding,
                                        box_top_left.y + box_padding + (j + 2) as f32 * 20.,
                                    ),
                                    size: 20.,
                                    vertical_alignment: Vertical::Center,
                                    horizontal_alignment: Horizontal::Left,
                                    ..Default::default()
                                });
                            });
                    })
            });

        vec![frame.into_geometry()]
    }
}

struct TextboxStyle {}

impl text_input::StyleSheet for TextboxStyle {
    fn active(&self) -> text_input::Style {
        text_input::Style {
            background: iced::Background::Color(iced::Color::TRANSPARENT),
            border_radius: 5.0,
            border_width: 1.0,
            border_color: iced::Color::from_rgb(0.7, 0.7, 0.7),
            ..Default::default()
        }
    }

    fn focused(&self) -> text_input::Style {
        text_input::Style {
            border_color: iced::Color::from_rgb(0.5, 0.5, 0.5),
            ..self.active()
        }
    }

    fn placeholder_color(&self) -> iced::Color {
        iced::Color::from_rgb(0.7, 0.7, 0.7)
    }

    fn value_color(&self) -> iced::Color {
        iced::Color::from_rgb(0.3, 0.3, 0.3)
    }

    fn selection_color(&self) -> iced::Color {
        iced::Color::from_rgb(0.8, 0.8, 1.0)
    }

    // other methods in Stylesheet have a default impl
}

pub trait MyValue {
    fn my_nullable_to_string_null(&self) -> String;
    fn my_nullable_to_string_empty(&self) -> String;
    fn my_notnull_to_string(&self) -> String;
    fn my_to_sql(&self) -> String;
    fn my_get_int(&self) -> String;
}

impl MyValue for Value {
    fn my_nullable_to_string_null(&self) -> String {
        match self {
            Value::Integer(val) => val.to_string(),
            Value::Real(val) => val.to_string(),
            Value::Text(val) => val.to_string(),
            Value::Null => "NULL".to_string(),
            _ => panic!("dsafdsfa"),
        }
    }
    fn my_nullable_to_string_empty(&self) -> String {
        match self {
            Value::Integer(val) => val.to_string(),
            Value::Real(val) => val.to_string(),
            Value::Text(val) => val.to_string(),
            Value::Null => "".to_string(),
            _ => panic!("dsafdsfa"),
        }
    }
    fn my_notnull_to_string(&self) -> String {
        match self {
            Value::Integer(val) => val.to_string(),
            Value::Real(val) => val.to_string(),
            Value::Text(val) => val.to_string(),
            _ => panic!("dsafdsfa"),
        }
    }
    fn my_to_sql(&self) -> String {
        match self {
            Value::Integer(val) => val.to_string(),
            Value::Real(val) => val.to_string(),
            Value::Text(val) => format!("'{}'", val.to_string()),
            Value::Null => "NULL".to_string(),
            _ => panic!("dsafdsfa"),
        }
    }
    fn my_get_int(&self) -> String {
        match self {
            Value::Integer(val) => val.to_string(),
            _ => panic!("dsafdsfa"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum EditorColumnType {
    Pk,
    Editable,
    Fk,
}

#[derive(Clone, Debug)]
pub struct CoolColumn {
    pub data: Vec<Value>,
    pub name: String,
    pub is_pk: bool,
    pub notnull: bool,
    pub fk: Option<MyAstForeignKey>,
    pub data_type: Type,
    pub col_type: EditorColumnType,
    pub width: u16,
}
// need access to whole table schema to determine column type because of table constraints

// i.e. a "row" - should really wrap this as a type to ensure it is distinguishable from CoolColumn.data?
trait CoolRow {
    fn label(&self, len: usize) -> String;
}

impl CoolRow for Vec<Value> {
    fn label(&self, len: usize) -> String {
        self.iter()
            .take(len)
            .map(|value| value.my_nullable_to_string_null())
            .collect::<Vec<_>>()
            .join(" : ")
    }
}

#[derive(Clone)]
pub struct CoolTable {
    pub name: String,
    // todo keeping as Vec<Value> for now to make it easy to share code when using a pk where for CellUpdated, but would be less loose to use Vec<i64> plus primary keys might have multiple values anyway...
    pub rowid_column: Vec<i64>,
    pub columns: Vec<CoolColumn>,
    pub new_row_inputs2: Vec<Value>,
    pub sort_order: Vec<(String, SortOrder)>,

    current_cell: (CurrentCellRowid, String),
    // would be easier and more performant to use Vec<Vec<bool>>, that way can just use indexes to update and check for selections. but this would mean we need to keep the matrix the same size as the sql table which is not easy - say a column is deleted, when table is refreshed from db, need to work out what the new matrix should be. maybe can just always drop all selection whenever a table schema is changed?
    // selected_cells: Vec<(CurrentCellRowid, String)>,
    // todo use 2darray here
    selected_cells: Vec<Vec<bool>>,
}

// todo remove all hard coding of id from here
impl CoolTable {
    fn has_fk(&self) -> bool {
        self.columns.iter().any(|col| col.fk.is_some())
    }
    fn pk(&self) -> Option<&CoolColumn> {
        self.columns.iter().find(|col| col.is_pk)
    }
    fn get_pk_value(&self, row_index: usize) -> Value {
        self.columns.iter().find(|col| col.is_pk).unwrap().data[row_index].clone()
    }
    pub fn get_row_index(&self, rowid: i64) -> usize {
        self.rowid_column
            .iter()
            .enumerate()
            .find_map(|(i, rowid2)| if rowid == *rowid2 { Some(i) } else { None })
            .unwrap()
    }
    pub fn get_rowid(&self, row_index: usize) -> i64 {
        self.rowid_column[row_index]
    }
    pub fn get_row(&self, rowid: i64) -> Vec<Value> {
        self.rowid_column
            .iter()
            .zip(self.rows_iter())
            .find(|(rowid2, row)| rowid2 == &&rowid)
            .unwrap()
            .1
    }

    // todo important note this will leave new_row_inputs2 empty because we need to create all tables data before we can look up default values from other table! Should use intermediary type like CoolTableTemp without new_row_inputs2 field?
    pub fn from_schema(conn: &Connection, table_name: &str, table_schema: &Statement) -> CoolTable {
        let rows;
        {
            let mut stmt = conn
                .prepare(&format!("select ROWID, * from \"{}\"", table_name))
                .unwrap();

            let col_count = stmt.column_count();
            let iter = stmt
                .query_map([], |row| {
                    let mut value_row = Vec::new();
                    for i in 0..col_count {
                        value_row.push(row.get::<_, Value>(i)?);
                    }
                    Ok(value_row)
                })
                .unwrap();
            rows = iter.map(|x| x.unwrap()).collect::<Vec<_>>();
        }

        let rowid_column = rows
            .iter()
            .map(|row| match row[0] {
                Value::Integer(rowid) => rowid,
                _ => panic!("rowid is not Value::Integer"),
            })
            .collect::<Vec<_>>();
        let columns = table_schema.my_get_columns();
        let columns = columns
            .iter()
            .enumerate()
            .map(|(i, col_def)| {
                let name = col_def.name.value.clone();
                // first cell in row is rowid, so need to use i + 1
                let col_type = table_schema.my_editor_column_type(name.clone());
                CoolColumn {
                    data: rows
                        .iter()
                        .map(|row| row[i + 1].clone())
                        .collect::<Vec<_>>(),
                    name: name.clone(),
                    is_pk: table_schema.my_is_pk(name.clone()),
                    notnull: col_def.notnull(),
                    fk: table_schema.my_get_fk_ast(name.clone()),
                    data_type: match col_def.data_type {
                        // todo SQLite accepts lots of other DataTypes,like Varchar etc
                        DataType::Int(..) => Type::Integer,
                        DataType::Real => Type::Real,
                        DataType::Text => Type::Text,
                        DataType::Custom(..) => Type::Blob,
                        _ => {
                            panic!("not supporting non sqlite data types (yet?)");
                        }
                    },
                    width: match col_type {
                        EditorColumnType::Pk => 150,
                        EditorColumnType::Editable => 150,
                        EditorColumnType::Fk => 200,
                    },
                    col_type,
                }
            })
            .collect::<Vec<_>>();

        // let new_row_inputs2 = columns
        //     .iter()
        //     .map(|column| {
        //         if column.notnull {
        //             match column.data_type {
        //                 Type::Integer => {
        //                     // if col is not nullable, need to get an actual value from referenced table. assumes at least one exists
        //                     // todo should fail gracefully if it doesn't
        //                     if column.fk.is_some() {
        //                         // todo don't get ref pk, get actual referenced column like:
        //                         let MyAstForeignKey {
        //                             table_name,
        //                             column_name,
        //                         } = column.fk.as_ref().unwrap();
        //                         reference_data_tables
        //                             .iter()
        //                             .find(|table| &table.name == table_name)
        //                             .unwrap()
        //                             .columns
        //                             .iter()
        //                             .find(|col| &col.name == column_name)
        //                             .unwrap()
        //                             .data[0]
        //                             .clone()
        //                         // reference_data[0].get_pk_value(0)
        //                     } else {
        //                         column.data_type.default_value()
        //                     }
        //                 }
        //                 Type::Real => column.data_type.default_value(),
        //                 Type::Text => column.data_type.default_value(),
        //                 _ => {
        //                     panic!("different data type")
        //                 }
        //             }
        //         } else {
        //             // nullable columns default to NULL
        //             Value::Null
        //         }
        //     })
        //     .collect::<Vec<_>>();

        let selected_cells = std::ops::Range {
            start: 0,
            end: columns[0].data.len() + 1,
        }
        .map(|_| {
            std::ops::Range {
                start: 0,
                end: columns.len() + 1,
            }
            .map(|_| false)
            .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

        CoolTable {
            name: table_name.to_string(),
            rowid_column,
            columns,
            sort_order: vec![],
            new_row_inputs2: vec![],
            current_cell: (CurrentCellRowid::ColumnHeaders, "rowid".to_string()),
            selected_cells,
        }
    }

    // maybe should just use the without newrow method and then have a specific function for generating newrow?
    pub fn from_schema_with_newrow(
        conn: &Connection,
        table_name: &str,
        table_schema: &Statement,
        other_tables: &Vec<CoolTable>,
    ) -> CoolTable {
        let rows;
        {
            let mut stmt = conn
                .prepare(&format!("select ROWID, * from \"{}\"", table_name))
                .unwrap();
            let col_count = stmt.column_count();
            let iter = stmt
                .query_map([], |row| {
                    let mut value_row = Vec::new();
                    for i in 0..col_count {
                        value_row.push(row.get::<_, Value>(i)?);
                    }
                    Ok(value_row)
                })
                .unwrap();
            rows = iter.map(|x| x.unwrap()).collect::<Vec<_>>();
        }

        let rowid_column = rows
            .iter()
            .map(|row| match row[0] {
                Value::Integer(rowid) => rowid,
                _ => panic!("rowid not Value::Integer"),
            })
            .collect::<Vec<_>>();
        let columns = table_schema.my_get_columns();
        let columns = columns
            .iter()
            .enumerate()
            .map(|(i, col_def)| {
                let name = col_def.name.value.clone();
                // first cell in row is rowid, so need to use i + 1
                let col_type = table_schema.my_editor_column_type(name.clone());
                CoolColumn {
                    data: rows
                        .iter()
                        .map(|row| row[i + 1].clone())
                        .collect::<Vec<_>>(),
                    name: name.clone(),
                    is_pk: table_schema.my_is_pk(name.clone()),
                    notnull: col_def.notnull(),
                    fk: table_schema.my_get_fk_ast(name.clone()),
                    data_type: match col_def.data_type {
                        // todo SQLite accepts lots of other DataTypes,like Varchar etc
                        DataType::Int(..) => Type::Integer,
                        DataType::Real => Type::Real,
                        DataType::Text => Type::Text,
                        DataType::Custom(..) => Type::Blob,
                        _ => {
                            panic!("not supporting non sqlite data types (yet?)");
                        }
                    },
                    width: match col_type {
                        EditorColumnType::Pk => 150,
                        EditorColumnType::Editable => 150,
                        EditorColumnType::Fk => 200,
                    },
                    col_type,
                }
            })
            .collect::<Vec<_>>();

        let new_row_inputs2 = columns
            .iter()
            .map(|column| {
                if column.notnull {
                    match column.data_type {
                        Type::Integer => {
                            // if col is not nullable, need to get an actual value from referenced table. assumes at least one exists
                            // todo should fail gracefully if it doesn't
                            if column.fk.is_some() {
                                // todo don't get ref pk, get actual referenced column like:
                                let MyAstForeignKey {
                                    table_name,
                                    column_name,
                                } = column.fk.as_ref().unwrap();
                                other_tables
                                    .iter()
                                    .find(|table| &table.name == table_name)
                                    .unwrap()
                                    .columns
                                    .iter()
                                    .find(|col| &col.name == column_name)
                                    .unwrap()
                                    .data[0]
                                    .clone()
                                // reference_data[0].get_pk_value(0)
                            } else {
                                column.data_type.default_value()
                            }
                        }
                        Type::Real => column.data_type.default_value(),
                        Type::Text => column.data_type.default_value(),
                        _ => {
                            panic!("different data type")
                        }
                    }
                } else {
                    // nullable columns default to NULL
                    Value::Null
                }
            })
            .collect::<Vec<_>>();

        let selected_cells = std::ops::Range {
            start: 0,
            end: columns[0].data.len() + 1,
        }
        .map(|_| {
            std::ops::Range {
                start: 0,
                end: columns.len() + 1,
            }
            .map(|_| false)
            .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

        CoolTable {
            name: table_name.to_string(),
            rowid_column,
            columns,
            sort_order: vec![],
            new_row_inputs2,
            current_cell: (CurrentCellRowid::ColumnHeaders, "rowid".to_string()),
            selected_cells,
        }
    }

    // todo this also returns a sql string which also is just a select is a good idea and we should maybe be doing it for other mehtods?
    pub fn reload_column_data_from_schema(
        &self,
        conn: &Connection,
        table_name: &str,
        table_schema: &Statement,
    ) -> (String, CoolTable) {
        let rows;
        let bound_sql_string;

        {
            let mut stmt = conn
                .prepare(&format!(
                    "select ROWID, * from \"{}\"{};",
                    table_name,
                    if self.sort_order.len() > 0 {
                        format!(
                            " ORDER BY \"{}\"",
                            self.sort_order
                                .iter()
                                .rev()
                                .map(|(col_name, sort_order)| format!(
                                    "{} {:?}",
                                    col_name, sort_order
                                ))
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    } else {
                        "".to_string()
                    }
                ))
                .unwrap();

            let col_count = stmt.column_count();
            bound_sql_string = stmt.expanded_sql().unwrap();
            let iter = stmt
                .query_map([], |row| {
                    let mut value_row = Vec::new();
                    for i in 0..col_count {
                        value_row.push(row.get::<_, Value>(i)?);
                    }
                    Ok(value_row)
                })
                .unwrap();
            rows = iter.map(|x| x.unwrap()).collect::<Vec<_>>();
        }

        let rowid_column = rows
            .iter()
            .map(|row| match row[0] {
                Value::Integer(rowid) => rowid,
                _ => panic!("rowid not Value::Integer"),
            })
            .collect::<Vec<_>>();
        let columns = table_schema.my_get_columns();
        let columns = columns
            .iter()
            .enumerate()
            .map(|(i, col_def)| {
                let name = col_def.name.value.clone();
                // first cell in row is rowid, so need to use i + 1
                let col_type = table_schema.my_editor_column_type(name.clone());
                CoolColumn {
                    data: rows
                        .iter()
                        .map(|row| row[i + 1].clone())
                        .collect::<Vec<_>>(),
                    name: name.clone(),
                    is_pk: table_schema.my_is_pk(name.clone()),
                    notnull: col_def.notnull(),
                    fk: table_schema.my_get_fk_ast(name.clone()),
                    data_type: match col_def.data_type {
                        // todo SQLite accepts lots of other DataTypes,like Varchar etc
                        DataType::Int(..) => Type::Integer,
                        DataType::Real => Type::Real,
                        DataType::Text => Type::Text,
                        DataType::Custom(..) => Type::Blob,
                        _ => {
                            panic!("not supporting non sqlite data types (yet?)");
                        }
                    },
                    width: match col_type {
                        EditorColumnType::Pk => 150,
                        EditorColumnType::Editable => 150,
                        EditorColumnType::Fk => 200,
                    },
                    col_type,
                }
            })
            .collect::<Vec<_>>();

        let selected_cells = std::ops::Range {
            start: 0,
            end: columns[0].data.len() + 1,
        }
        .map(|_| {
            std::ops::Range {
                start: 0,
                end: columns.len() + 1,
            }
            .map(|_| false)
            .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

        (
            bound_sql_string,
            CoolTable {
                name: table_name.to_string(),
                rowid_column,
                columns,
                sort_order: self.sort_order.clone(),
                new_row_inputs2: self.new_row_inputs2.clone(),
                current_cell: (CurrentCellRowid::ColumnHeaders, "rowid".to_string()),
                selected_cells,
            },
        )
    }

    // reload data and schema info, keeping app state like sort order (todo keeping app state)
    // should maybe return sql string? remember we don't return strings for all sql that is run, e.g. making reference selector. but it is useful to return a string if there is an error, otherwise it will fail silently
    pub fn reload_everything_from_schema(
        &self,
        conn: &Connection,
        table_name: &str,
        table_schema: &Statement,
        other_tables: &Vec<CoolTable>,
    ) -> (String, CoolTable) {
        let rows;
        let bound_sql_string;
        {
            let mut stmt = conn
                .prepare(&format!("select ROWID, * from \"{}\"", table_name))
                .unwrap();
            let col_count = stmt.column_count();
            bound_sql_string = stmt.expanded_sql().unwrap();
            let iter = stmt
                .query_map([], |row| {
                    let mut value_row = Vec::new();
                    for i in 0..col_count {
                        value_row.push(row.get::<_, Value>(i)?);
                    }
                    Ok(value_row)
                })
                .unwrap();
            rows = iter.map(|x| x.unwrap()).collect::<Vec<_>>();
        }

        let rowid_column = rows
            .iter()
            .map(|row| match row[0] {
                Value::Integer(rowid) => rowid,
                _ => panic!("rowid not Value::Integer"),
            })
            .collect::<Vec<_>>();
        let columns = table_schema.my_get_columns();
        let columns = columns
            .iter()
            .enumerate()
            .map(|(i, col_def)| {
                let name = col_def.name.value.clone();
                // first cell in row is rowid, so need to use i + 1
                let col_type = table_schema.my_editor_column_type(name.clone());
                CoolColumn {
                    data: rows
                        .iter()
                        .map(|row| row[i + 1].clone())
                        .collect::<Vec<_>>(),
                    name: name.clone(),
                    is_pk: table_schema.my_is_pk(name.clone()),
                    notnull: col_def.notnull(),
                    fk: table_schema.my_get_fk_ast(name.clone()),
                    data_type: match col_def.data_type {
                        // todo SQLite accepts lots of other DataTypes,like Varchar etc
                        DataType::Int(..) => Type::Integer,
                        DataType::Real => Type::Real,
                        DataType::Text => Type::Text,
                        DataType::Custom(..) => Type::Blob,
                        _ => {
                            panic!("not supporting non sqlite data types (yet?)");
                        }
                    },
                    width: match col_type {
                        EditorColumnType::Pk => 150,
                        EditorColumnType::Editable => 150,
                        EditorColumnType::Fk => 200,
                    },
                    col_type,
                }
            })
            .collect::<Vec<_>>();

        let new_row_inputs2 = columns
            .iter()
            .map(|column| {
                if column.notnull {
                    match column.data_type {
                        Type::Integer => {
                            // if col is not nullable, need to get an actual value from referenced table. assumes at least one exists
                            // todo should fail gracefully if it doesn't
                            if column.fk.is_some() {
                                // todo don't get ref pk, get actual referenced column like:
                                let MyAstForeignKey {
                                    table_name,
                                    column_name,
                                } = column.fk.as_ref().unwrap();
                                other_tables
                                    .iter()
                                    .find(|table| &table.name == table_name)
                                    .unwrap()
                                    .columns
                                    .iter()
                                    .find(|col| &col.name == column_name)
                                    .unwrap()
                                    .data[0]
                                    .clone()
                                // reference_data[0].get_pk_value(0)
                            } else {
                                column.data_type.default_value()
                            }
                        }
                        Type::Real => column.data_type.default_value(),
                        Type::Text => column.data_type.default_value(),
                        _ => {
                            panic!("different data type")
                        }
                    }
                } else {
                    // nullable columns default to NULL
                    Value::Null
                }
            })
            .collect::<Vec<_>>();

        let col_names = columns.iter().map(|col| &col.name).collect::<Vec<_>>();
        let sort_order = self
            .sort_order
            .iter()
            .cloned()
            .filter(|(col_name, _)| col_names.contains(&col_name))
            .collect::<Vec<_>>();

        let selected_cells = std::ops::Range {
            start: 0,
            end: columns[0].data.len() + 1,
        }
        .map(|_| {
            std::ops::Range {
                start: 0,
                end: columns.len() + 1,
            }
            .map(|_| false)
            .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

        (
            bound_sql_string,
            CoolTable {
                name: table_name.to_string(),
                // rows,
                // table_schema,
                // column_names,
                rowid_column,
                columns,
                // need to reconcile this to check the columns still exist
                sort_order,
                new_row_inputs2,
                current_cell: (CurrentCellRowid::ColumnHeaders, "rowid".to_string()),
                selected_cells,
            },
        )
    }
}

impl CoolTable {
    fn rows_iter(&self) -> CoolTableIter {
        CoolTableIter {
            table: self,
            index: 0,
        }
    }
    fn column_names(&self) -> Vec<&str> {
        self.columns
            .iter()
            .map(|col| col.name.as_str())
            .collect::<Vec<_>>()
    }
}

struct CoolTableIter<'a> {
    table: &'a CoolTable,
    index: usize,
}

impl<'a> Iterator for CoolTableIter<'a> {
    type Item = Vec<Value>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.table.columns[0].data.len() {
            let mut row = Vec::new();
            self.table
                .columns
                .iter()
                .for_each(|col| row.push(col.data[self.index].clone()));
            self.index += 1;
            Some(row)
        } else {
            None
        }
    }
}

// todo! ast shouldn't be a nested dataself by itself, it should just be extra fields on the actual data, i.e. CoolTable and CoolColumn:
// Db: tables, Table: name, columns, Column: name, is_pk, opt_fk, data_type, col_type, notnull, Fk: table_name, column_name
// Maybe We still want MyAsT as it could be useful to have just the schema in isolation, e.g. for building diagrams? remains to be seen. for now stick to using Vec<Statement>

#[derive(Clone, Debug)]
pub struct MyAstForeignKey {
    pub table_name: String,
    pub column_name: String,
}
// }

trait MyColumnDef {
    fn my_is_pk(&self) -> bool;
    fn my_is_fk(&self) -> bool;
    fn notnull(&self) -> bool;
}

// this will not find cases where the PK is defined as a table constraint
// we don't use it directly, it is used by equivalent table method
impl MyColumnDef for ColumnDef {
    fn my_is_pk(&self) -> bool {
        self.options.iter().any(|option| match option.option {
            ColumnOption::Unique { is_primary } => is_primary,
            _ => false,
        })
    }
    fn my_is_fk(&self) -> bool {
        self.options.iter().any(|option| match option.option {
            ColumnOption::ForeignKey { .. } => true,
            _ => false,
        })
    }
    fn notnull(&self) -> bool {
        self.options
            .iter()
            .filter(|opt| match opt.option {
                ColumnOption::NotNull => true,
                _ => false,
            })
            .count()
            > 0
    }
}

trait MyObjectName {
    fn my_join(&self) -> String;
}

impl MyObjectName for ObjectName {
    fn my_join(&self) -> String {
        self.0
            .iter()
            .map(|ident| ident.value.clone())
            .collect::<Vec<_>>()
            .join(".")
    }
}

// todo eventually should precalculate and define most of this on CoolColumn, since while it is understandable that for sql tables it is necessary to access the whole table schema to determine info about a single column (because of table constraints), this kind of api makes no sense for other non-sql backends
trait MyStatement {
    fn my_table_name(&self) -> String;
    fn my_reference_table_names(&self) -> Vec<String>;
    fn my_get_column_names(&self) -> Vec<String>;
    fn my_get_pk_column_names(&self) -> Vec<String>;
    fn my_get_columns(&self) -> Vec<ColumnDef>;
    fn my_get_column(&self, column_name: String) -> ColumnDef;
    fn my_get_constraints(&self) -> Vec<TableConstraint>;
    fn my_is_pk(&self, column_name: String) -> bool;
    fn my_is_fk(&self, column_name: String) -> bool;
    fn my_get_fk_ast(&self, column_name: String) -> Option<MyAstForeignKey>;
    fn my_editor_column_type(&self, column_name: String) -> EditorColumnType;
    fn my_get_ref_table_name(&self, column_name: String) -> String;
}

impl MyStatement for Statement {
    fn my_table_name(&self) -> String {
        if let Statement::CreateTable { name, .. } = self {
            name.my_join()
        } else {
            panic!("not a create table statement");
        }
    }
    fn my_reference_table_names(&self) -> Vec<String> {
        match self {
            Statement::CreateTable {
                columns,
                constraints,
                ..
            } => {
                let columns_references = columns
                    .iter()
                    .filter(|column| {
                        column
                            .options
                            .iter()
                            .any(|col_option| match col_option.option.clone() {
                                ColumnOption::ForeignKey { .. } => true,
                                _ => false,
                            })
                    })
                    .map(|column_with_fk| {
                        let fk_option = column_with_fk
                            .options
                            .iter()
                            .find(|col_option| match col_option.option.clone() {
                                ColumnOption::ForeignKey { .. } => true,
                                _ => false,
                            })
                            .unwrap()
                            .option
                            .clone();
                        let foreign_table = match fk_option {
                            ColumnOption::ForeignKey { foreign_table, .. } => foreign_table,
                            _ => panic!("not a foreign key"),
                        };
                        foreign_table.my_join()
                    });
                let table_constraint_references = constraints
                    .iter()
                    .filter(|constraint| match constraint {
                        TableConstraint::ForeignKey { .. } => true,
                        _ => false,
                    })
                    .map(|constraint| match constraint {
                        TableConstraint::ForeignKey { foreign_table, .. } => {
                            foreign_table.my_join()
                        }
                        _ => panic!("not a foreign key"),
                    });
                // todo!()
                columns_references
                    .chain(table_constraint_references)
                    .collect::<Vec<_>>()
            }
            _ => {
                panic!("not a create table statement");
            }
        }
    }
    fn my_get_column_names(&self) -> Vec<String> {
        match self {
            Statement::CreateTable { columns, .. } => columns
                .iter()
                .map(|column_def| column_def.name.value.clone())
                .collect::<Vec<_>>(),
            _ => {
                panic!("not a create table statement");
            }
        }
    }
    fn my_get_pk_column_names(&self) -> Vec<String> {
        match self {
            Statement::CreateTable { columns, .. } => columns
                .iter()
                .filter(|column_def| self.my_is_pk(column_def.name.value.clone()))
                .map(|column_def| column_def.name.value.clone())
                .collect::<Vec<_>>(),
            _ => {
                panic!("not a create table statement");
            }
        }
    }
    fn my_get_columns(&self) -> Vec<ColumnDef> {
        match self {
            Statement::CreateTable { columns, .. } => columns.clone(),
            _ => {
                panic!("not a create table statement");
            }
        }
    }
    fn my_get_column(&self, column_name: String) -> ColumnDef {
        self.my_get_columns()
            .iter()
            .find(|col| col.name.value == column_name)
            .unwrap()
            .clone()
    }
    fn my_get_constraints(&self) -> Vec<TableConstraint> {
        match self {
            Statement::CreateTable { constraints, .. } => constraints.clone(),
            _ => {
                panic!("not a create table statement");
            }
        }
    }
    fn my_is_pk(&self, column_name: String) -> bool {
        self.my_get_columns()
            .iter()
            .find(|column| column.name.value == column_name)
            .unwrap()
            .my_is_pk()
            || {
                self.my_get_constraints()
                    .iter()
                    .filter(|constraint| match constraint {
                        TableConstraint::Unique {
                            is_primary,
                            columns,
                            ..
                        } => {
                            *is_primary && columns.iter().any(|column| column.value == column_name)
                        }
                        _ => false,
                    })
                    .collect::<Vec<_>>()
                    .len()
                    > 0
            }
    }
    fn my_is_fk(&self, column_name: String) -> bool {
        self.my_get_columns()
            .iter()
            .find(|column| column.name.value == column_name)
            .unwrap()
            .my_is_fk()
            || {
                self.my_get_constraints()
                    .iter()
                    .filter(|constraint| match constraint {
                        TableConstraint::ForeignKey { columns, .. } => {
                            columns.iter().any(|column| column.value == column_name)
                        }
                        _ => false,
                    })
                    .collect::<Vec<_>>()
                    .len()
                    > 0
            }
    }
    fn my_get_fk_ast(&self, column_name: String) -> Option<MyAstForeignKey> {
        let old_col = self.my_get_column(column_name.clone());
        if old_col.my_is_fk() {
            let fk = &old_col
                .options
                .iter()
                .find(|col_def| match col_def.option {
                    ColumnOption::ForeignKey { .. } => true,
                    _ => false,
                })
                .unwrap()
                .option;
            match fk {
                ColumnOption::ForeignKey {
                    foreign_table,
                    referred_columns,
                    ..
                } => Some(MyAstForeignKey {
                    table_name: foreign_table.my_join(),
                    column_name: referred_columns[0].value.clone(),
                }),
                _ => {
                    panic!("dafds");
                }
            }
        } else if self
            .my_get_constraints()
            .iter()
            .any(|constraint| match constraint {
                TableConstraint::ForeignKey { columns, .. } => columns
                    .iter()
                    .any(|column| column.value == old_col.name.value.clone()),
                _ => false,
            })
        {
            let my_fk = self
                .my_get_constraints()
                .iter()
                .find(|constraint| match constraint {
                    TableConstraint::ForeignKey { columns, .. } => columns
                        .iter()
                        .any(|column| column.value == old_col.name.value.clone()),
                    _ => false,
                })
                .unwrap()
                .clone();

            match my_fk {
                TableConstraint::ForeignKey {
                    foreign_table,
                    referred_columns,
                    ..
                } => Some(MyAstForeignKey {
                    table_name: foreign_table.my_join(),
                    column_name: referred_columns[0].value.clone(),
                }),
                _ => {
                    panic!("dafds");
                }
            }
        } else {
            None
        }
    }
    fn my_editor_column_type(&self, column_name: String) -> EditorColumnType {
        if self.my_is_pk(column_name.clone()) {
            EditorColumnType::Pk
        } else if self.my_is_fk(column_name.clone()) {
            EditorColumnType::Fk
        } else {
            EditorColumnType::Editable
        }
    }
    fn my_get_ref_table_name(&self, column_name: String) -> String {
        self.my_get_constraints()
            .iter()
            .find_map(|constraint| match constraint {
                TableConstraint::ForeignKey {
                    columns,
                    foreign_table,
                    ..
                } => {
                    if columns.iter().any(|column| column.value == column_name) {
                        Some(foreign_table.my_join())
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .unwrap()
    }
}

trait MyVecStatement {
    fn my_table_names(&self) -> Vec<String>;
    fn my_get_reference_table_names(&self, table_name: &str) -> Vec<String>;
    fn my_get_table_schema(&self, table_name: &str) -> Statement;
}

impl MyVecStatement for Vec<Statement> {
    fn my_table_names(&self) -> Vec<String> {
        self.iter()
            .map(|statement| statement.my_table_name())
            .collect::<Vec<_>>()
    }
    fn my_get_reference_table_names(&self, table_name: &str) -> Vec<String> {
        let my_table = self
            .iter()
            .find(|statement| statement.my_table_name() == table_name)
            .unwrap();
        my_table.my_reference_table_names()
    }
    fn my_get_table_schema(&self, table_name: &str) -> Statement {
        self.iter()
            .find(|statement| statement.my_table_name() == table_name)
            .unwrap()
            .clone()
    }
}
// }
