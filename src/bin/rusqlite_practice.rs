use rusqlite::{params, Connection, Result};

#[derive(Debug)]
struct Person {
    id: i32,
    name: String,
    data: Option<Vec<u8>>,
}

#[derive(Debug)]
struct Business {
    id: i32,
    name: String,
    companies_house_number: Option<i32>,
}

fn main() -> Result<()> {
    // let conn = Connection::open_in_memory()?;
    let conn = Connection::open("businesses.db")?;

    // conn.execute(
    //     "CREATE TABLE person (
    //               id              INTEGER PRIMARY KEY,
    //               name            TEXT NOT NULL,
    //               data            BLOB
    //               )",
    //     [],
    // )?;
    // let me = Person {
    //     id: 0,
    //     name: "Steven".to_string(),
    //     data: Some(vec![1, 2, 3]),
    // };
    // conn.execute(
    //     "INSERT INTO person (name, data) VALUES (?1, ?2)",
    //     params![me.name, me.data],
    // )?;

    let mut stmt = conn.prepare("SELECT * FROM business")?;
    let business_iter = stmt.query_map([], |row| {
        Ok(Business {
            id: row.get(0)?,
            name: row.get(1)?,
            companies_house_number: row.get(2)?,
        })
    })?;

    let mut business_vec = Vec::new();
    for business in business_iter {
        business_vec.push(business?);
        // println!("Found person {:?}", business.unwrap());
    }
    println!("{:?}", business_vec);
    Ok(())
}
