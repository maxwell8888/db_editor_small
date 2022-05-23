// -- todo --
// x add new column
// x add two columns together
// x subset rows
// x subset columns - based on what? just indexes? drop column would be better?
// x sum
// group by sum
// append df
// reorder columns
// summary tables
// join
// tests
// df generator
// print stats about df - types, length, etc
// csv type checking etc
// read csv
// nullable/non nullable columns

// trait SummableColumn {
//     fn col_sum<T>(&self) -> T;
// }

// trait ColumnTrait {
//     fn get_name(&self) -> String;
//     // fn int_col_sum(&self) -> i32 {
//     //     0
//     // }
//     // fn flt_col_sum(&self) -> f32 {
//     //     0.0
//     // }
//     fn get_data<T>(&self) -> &Vec<T>;
// }

#[derive(Clone)]
struct IntColumn {
    name: String,
    data: Vec<i32>,
}

#[derive(Clone)]
struct FltColumn {
    name: String,
    data: Vec<f32>,
}

#[derive(Clone)]
struct StrColumn {
    name: String,
    data: Vec<String>,
}
impl IntColumn {
    fn col_sum(&self) -> i32 {
        self.data.iter().sum()
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_data(&self) -> &Vec<i32> {
        &self.data
    }
}
// impl ColumnTrait for IntColumn {
// }
// impl SummableColumn for IntColumn {
//     fn col_sum<i32>(&self) -> i32 {
//         self.data.iter().sum()
//     }
// }
impl FltColumn {
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn col_sum(&self) -> f32 {
        self.data.iter().sum()
    }
    fn get_data(&self) -> &Vec<f32> {
        &self.data
    }
}
impl StrColumn {
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_data(&self) -> &Vec<String> {
        &self.data
    }
}

#[derive(Clone)]
struct Dataframe {
    int_columns: Vec<IntColumn>,
    flt_columns: Vec<FltColumn>,
    str_columns: Vec<StrColumn>,
}
impl Dataframe {
    fn add_int_column(&mut self, new_column: IntColumn) {
        self.int_columns.push(new_column)
    }
    fn add_two_int_columns(&mut self, col1: usize, col2: usize, name: String) {
        let mut new_data: Vec<i32> = Vec::new();
        for (i, val) in self.int_columns[col1].get_data().iter().enumerate() {
            new_data.push(val + self.int_columns[col2].get_data()[i]);
        }
        let new_column = IntColumn {
            name: name,
            data: new_data,
        };
        self.int_columns.push(new_column)
    }
    fn row_subset(&self, indexes: Vec<usize>) -> Dataframe {
        let mut new = self.clone();

        for (i, col) in self.int_columns.iter().enumerate() {
            new.int_columns[i].data = self.int_columns[i]
                .data
                .iter()
                .enumerate()
                .filter(|x| indexes.contains(&x.0))
                .map(|x| *x.1)
                .collect();
        }
        for (i, col) in self.flt_columns.iter().enumerate() {
            new.flt_columns[i].data = self.flt_columns[i]
                .data
                .iter()
                .enumerate()
                .filter(|x| indexes.contains(&x.0))
                .map(|x| *x.1)
                .collect();
        }
        for (i, col) in self.str_columns.iter().enumerate() {
            new.str_columns[i].data = self.str_columns[i]
                .data
                .iter()
                .enumerate()
                .filter(|x| indexes.contains(&x.0))
                .map(|x| (*x.1).clone())
                .collect();
        }
        new.clone()
    }
    fn get_int_col_by_name(&self, name: String) -> Option<&IntColumn> {
        for column in &self.int_columns {
            if column.name == name {
                return Some(&column);
            }
        }
        None
    }
    fn get_flt_col_by_name(&self, name: String) -> Option<&FltColumn> {
        for column in &self.flt_columns {
            if column.name == name {
                return Some(&column);
            }
        }
        None
    }
    fn get_str_col_by_name(&self, name: String) -> Option<&StrColumn> {
        for column in &self.str_columns {
            if column.name == name {
                return Some(&column);
            }
        }
        None
    }
    fn drop_int_column_by_name(&mut self, name: String) {
        let int_cols = self.int_columns.clone();
        for (i, column) in int_cols.iter().enumerate() {
            if column.name == name {
                self.int_columns.remove(i);
            }
        }
    }
    fn append_df(&mut self, df2: &Dataframe) {
        for (i, col) in self.int_columns.iter().enumerate() {
            if df2.get_int_col_by_name(col.get_name()).is_none() {
                panic!("df2 does not have column: {}", col.get_name());
            }
        }

        for column in self.int_columns.iter_mut() {
            let mut other_data = df2
                .get_int_col_by_name(column.get_name())
                .unwrap()
                .data
                .clone();
            column.data.append(&mut other_data)
        }
        for column in self.flt_columns.iter_mut() {
            let mut other_data = df2
                .get_flt_col_by_name(column.get_name())
                .unwrap()
                .data
                .clone();
            column.data.append(&mut other_data)
        }
        for column in self.str_columns.iter_mut() {
            let mut other_data = df2
                .get_str_col_by_name(column.get_name())
                .unwrap()
                .data
                .clone();
            column.data.append(&mut other_data)
        }
    }
    // fn concat(&mut self, df2: &Dataframe) {

    // }
    // fn join(&self, df2: Dataframe, df1_name: String, df2_name: String) -> Dataframe {
    //     for i in &self.get_int_col_by_name(df1_name).unwrap().data {
    //         for j in df2.get_int_col_by_name(df2_name).unwrap().data {
    //             if i == j {

    //             }
    //         }
    //     }
    //     self.clone()
    // }

    fn print_df(&self) {
        for col in &self.int_columns {
            print!("{}\t", col.get_name());
        }
        for col in &self.flt_columns {
            print!("{}\t", col.get_name());
        }
        for col in &self.str_columns {
            print!("{}\t", col.get_name());
        }
        println!();
        for (i, val) in self.int_columns[0].get_data().iter().enumerate() {
            for column in &self.int_columns {
                print!("{}\t", column.get_data()[i]);
            }
            for column in &self.flt_columns {
                print!("{}\t", column.get_data()[i]);
            }
            for column in &self.str_columns {
                print!("\"{}\"\t", column.get_data()[i]);
            }
            println!();
        }
    }
    // fn print_col_names(&self) {
    //     for col in &self.columns {
    //         println!("{}", col.get_name());
    //     }
    // }
}
// impl Copy for Dataframe {

// }

trait IndexWhere<T> {
    fn index_where<F>(&self, f: F) -> Vec<usize>
    where
        F: Fn(&(usize, &T)) -> bool;
}
impl<T> IndexWhere<T> for Vec<T> {
    fn index_where<F>(&self, f: F) -> Vec<usize>
    where
        F: Fn(&(usize, &T)) -> bool,
    {
        self.iter().enumerate().filter(f).map(|x| x.0).collect()
    }
}

fn main() {
    let tup = (500, 6.4, 1);
    println!("creat a df");
    let mut df = Dataframe {
        int_columns: vec![IntColumn {
            name: String::from("col1"),
            data: vec![1, 2, 3],
        }],
        flt_columns: vec![FltColumn {
            name: String::from("col3"),
            data: vec![1.1, 2.1, 3.1],
        }],
        str_columns: vec![StrColumn {
            name: String::from("col2"),
            data: vec![String::from("1"), String::from("2"), String::from("3")],
        }],
    };
    df.print_df();

    println!("\nappend df");
    let mut df_append = Dataframe {
        int_columns: vec![IntColumn {
            name: String::from("col1"),
            data: vec![4, 5],
        }],
        flt_columns: vec![FltColumn {
            name: String::from("col3"),
            data: vec![4.1, 5.1],
        }],
        str_columns: vec![StrColumn {
            name: String::from("col2"),
            data: vec![String::from("4"), String::from("5")],
        }],
    };
    df_append.print_df();
    df.append_df(&df_append);
    df.print_df();
    df_append.print_df();

    println!("\nadd column");
    df.add_int_column(IntColumn {
        name: String::from("col4"),
        data: vec![1, 2, 3, 4, 5],
    });
    df.print_df();

    println!("\nsum column");
    let col1 = &df.get_int_col_by_name(String::from("col1")).unwrap();
    println!("col1: {}", col1.col_sum());

    println!("\nadd two columns");
    df.add_two_int_columns(0, 1, String::from("col5"));
    df.print_df();

    let col5 = &df.get_int_col_by_name(String::from("col5")).unwrap();
    println!("\ncol5 sum: {}", col5.col_sum());

    println!("\nsubset columns with vec");
    let df2 = df.row_subset(vec![0, 2]);
    df2.print_df();

    println!("\nsubset columns with calculated vec");
    let mut df3 = df.row_subset(
        df.get_int_col_by_name(String::from("col1"))
            .unwrap()
            .data
            .index_where(|x| *x.1 > 1),
    );
    df3.print_df();

    println!("\ndrop column");
    df3.drop_int_column_by_name(String::from("col4"));
    df3.print_df();
}
