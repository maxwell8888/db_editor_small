#![feature(slice_partition_dedup)]

use rusqlite::types::ToSqlOutput;
use rusqlite::{params, types::Value, Connection, ToSql};

use chrono::prelude::*;
use chrono::Utc;
use db_editor_small::personal_finance::*;
use std::fs;
use std::fs::DirEntry;
use std::fs::File;

fn get_dir_entries(path: &str, name: &str) -> Vec<DirEntry> {
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
    dir_entries
}

#[derive(Debug, Clone)]
pub struct SqlAccount {
    pub name: String,
    pub account_type: i64,
    pub account_open_date: Option<NaiveDate>,
}
impl SqlAccount {
    fn get_accounts_from_db(conn: &Connection) -> Vec<(i64, SqlAccount)> {
        let mut stmt = conn.prepare("select * from account;").unwrap();
        let account_iter = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    SqlAccount {
                        name: row.get(1)?,
                        account_type: row.get(2)?,
                        account_open_date: None,
                    },
                ))
            })
            .unwrap();
        let mut account_vec = Vec::new();
        for payee in account_iter {
            account_vec.push(payee.unwrap());
        }
        account_vec
    }
}

#[derive(Debug, Clone)]
pub struct SqlPayee {
    pub raw_payee_name: String,
    pub business_id: Option<i64>,
}
impl SqlPayee {
    fn get_payees_from_db(conn: &Connection) -> Vec<(i64, SqlPayee)> {
        let mut stmt = conn.prepare("select * from payees;").unwrap();
        let payee_iter = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    SqlPayee {
                        raw_payee_name: row.get(1)?,
                        business_id: row.get(2)?,
                    },
                ))
            })
            .unwrap();
        let mut payee_vec = Vec::new();
        for payee in payee_iter {
            payee_vec.push(payee.unwrap());
        }
        payee_vec
    }
    fn get_dedup_payees_from_qif_dir(path: &str, name: &str) -> Vec<SqlPayee> {
        let dir_entries = get_dir_entries(path, name);
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

        let mut payees = qifs
            .iter()
            .flat_map(|qif| {
                // transactions.reverse();
                qif.transactions.clone()
            })
            // initial balance seems to only be included when the balance doesn't start at 0 (i.e. is a new account) we simply drop it here but should be used to verify data and indentify missing transactions or other inconsistencies
            .filter(|transaction| transaction.payee != "INITIAL BALANCE")
            .map(|transaction| SqlPayee {
                raw_payee_name: transaction.payee.to_string(),
                business_id: None,
            })
            .collect::<Vec<_>>();
        payees.sort_by_key(|payee| payee.raw_payee_name.clone());
        payees.dedup_by_key(|payee| payee.raw_payee_name.clone());
        payees
    }
    // use placeholders...
    fn into_sql_insert(&self) -> String {
        format!("(NULL, ?, ?)")
    }
    fn write_payees_to_db(payees: &Vec<SqlPayee>, conn: &Connection) -> rusqlite::Result<usize> {
        if payees.len() > 0 {
            let values = payees
                .iter()
                .map(|_| format!("(NULL, ?, ?)"))
                .collect::<Vec<_>>();
            let values = values.join(",\n");
            let mut stmt = conn.prepare(&format!("INSERT INTO \"payees\" VALUES {};", values))?;
            let bad_params3 = payees
                .iter()
                .flat_map(|payee| {
                    vec![
                        Some(payee.raw_payee_name.clone()),
                        payee.business_id.map(|val| val.to_string()),
                    ]
                })
                .collect::<Vec<_>>();
            let params2 = bad_params3
                .iter()
                .map(|x| x as &dyn ToSql)
                .collect::<Vec<_>>();
            let bound_sql_string2 = stmt.expanded_sql().unwrap();
            let n = stmt.execute(&params2[..])?;
            let bound_sql_string = stmt.expanded_sql().unwrap();
            Ok(n)
        } else {
            Ok(0)
        }
    }
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
impl SqlTransaction {
    fn get_transactions_from_db(conn: &Connection) -> Vec<(usize, SqlTransaction)> {
        let mut stmt = conn.prepare("select * from \"transaction\";").unwrap();
        let payee_iter = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    SqlTransaction {
                        account_id: row.get(1)?,
                        date: row.get(2)?,
                        amount: row.get(3)?,
                        starting_balance: row.get(4)?,
                        closing_balance: row.get(5)?,
                        payee_id: row.get(6)?,
                        category_id: row.get(7)?,
                    },
                ))
            })
            .unwrap();
        let mut payee_vec = Vec::new();
        for payee in payee_iter {
            payee_vec.push(payee.unwrap());
        }
        payee_vec
    }
    fn get_transactions_from_qif_dir(
        path: &str,
        account_name: &str,
        account_type: i64,
        account_open_date: Option<NaiveDate>,
        payees: &Vec<(i64, SqlPayee)>,
        accounts: &Vec<(i64, SqlAccount)>,
    ) -> Vec<SqlTransaction> {
        let dir_entries = get_dir_entries(path, account_name);
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

        let mut transactions = qifs
            .iter()
            .flat_map(|qif| {
                // transactions.reverse();
                qif.transactions.clone()
            })
            // initial balance seems to only be included when the balance doesn't start at 0 (i.e. is a new account) we simply drop it here but should be used to verify data and indentify missing transactions or other inconsistencies
            .filter(|transaction| transaction.payee != "INITIAL BALANCE")
            .map(|transaction| {
                let payee_id = transaction.payee.to_string();

                SqlTransaction {
                    account_id: accounts
                        .iter()
                        .find_map(|account| {
                            if &account.1.name == account_name
                                && account.1.account_type == account_type
                                && account.1.account_open_date == account_open_date
                            {
                                Some(account.0 as i64)
                            } else {
                                None
                            }
                        })
                        .unwrap(),
                    date: NaiveDate::parse_from_str(&transaction.date, "%Y-%m-%d").unwrap(),
                    amount: transaction.amount,
                    starting_balance: 0,
                    closing_balance: 0,
                    payee_id: payees
                        .iter()
                        .find_map(|payee| {
                            if payee.1.raw_payee_name == transaction.payee {
                                Some(payee.0 as i64)
                            } else {
                                None
                            }
                        })
                        .unwrap(),
                    category_id: None,
                }
            })
            .collect::<Vec<_>>();
        // todo use proper starting_balance
        // let mut balance = starting_balance;
        let mut balance = 0;
        transactions.iter_mut().for_each(|transaction| {
            transaction.starting_balance = balance;
            balance = balance + transaction.amount;
            transaction.closing_balance = balance;
        });
        transactions
    }
    // use placeholders...
    fn into_sql_vec(&self) -> Vec<ToSqlOutput> {
        let SqlTransaction {
            account_id,
            date,
            amount,
            starting_balance,
            closing_balance,
            payee_id,
            category_id,
        } = self;
        vec![
            account_id.to_sql().unwrap(),
            ToSqlOutput::from(date.format("%Y-%m-%d").to_string()),
            amount.to_sql().unwrap(),
            starting_balance.to_sql().unwrap(),
            closing_balance.to_sql().unwrap(),
            payee_id.to_sql().unwrap(),
            category_id.to_sql().unwrap(),
        ]
    }
    fn into_sql_insert(&self) -> String {
        format!("(NULL, ?, ?, ?, ?, ?, ?, ?)")
    }
    fn write_transactions_to_db(
        transactions: &Vec<SqlTransaction>,
        conn: &Connection,
    ) -> rusqlite::Result<usize> {
        let values = transactions
            .iter()
            .map(|_| format!("(NULL, ?, ?, ?, ?, ?, ?, ?)"))
            .collect::<Vec<_>>();
        let values = values.join(",\n");
        let mut stmt = conn.prepare(&format!("INSERT INTO \"transaction\" VALUES {};", values))?;
        let params1 = transactions
            .iter()
            .flat_map(|transaction| transaction.into_sql_vec())
            .collect::<Vec<_>>();
        let params2 = params1.iter().map(|x| x as &dyn ToSql).collect::<Vec<_>>();
        let n = stmt.execute(&params2[..])?;
        Ok(n)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("businesses.db").unwrap();

    // add new deduped payees to payees table
    // add new transactions now that we can link against payees

    // set qif paths
    let santander_path = "data/santander_credit_card";
    let monzo_path = "data/monzo_current_account";

    // get qif payees
    let mut santander_payees = SqlPayee::get_dedup_payees_from_qif_dir(santander_path, "santander");
    let mut monzo_payees = SqlPayee::get_dedup_payees_from_qif_dir(monzo_path, "monzo");
    santander_payees.append(&mut monzo_payees);
    let mut all_qif_transactions = santander_payees;
    all_qif_transactions.sort_by(|p1, p2| p1.raw_payee_name.cmp(&p2.raw_payee_name));

    // get db payees
    let mut all_db_payees = SqlPayee::get_payees_from_db(&conn);
    all_db_payees.sort_by(|p1, p2| p1.1.raw_payee_name.cmp(&p2.1.raw_payee_name));

    // subset to only new payees

    // ASSUMPTION there are no removals - db is a strict subset of qifs
    // ASSUMPTION raw_payee_name's are unique. they are because they have been deduped, the real assumption is where we are deduping them that duplicates are in fact the same payee

    // todo can try using HashSets to avoid writing our own algo:
    // https://users.rust-lang.org/t/idiomatic-way-to-get-difference-between-two-vecs/48396/10
    // https://stackoverflow.com/questions/63557089/is-there-a-built-in-function-to-compute-the-difference-of-two-sets
    // let item_set: HashSet<_> = previous_items.iter().collect();
    // let difference: Vec<_> = new_items
    //     .into_iter()
    //     .filter(|item| !item_set.contains(item))
    //     .collect();
    // or btree set: https://www.reddit.com/r/learnrust/comments/fswfq7/comparing_2_vectors/

    let mut new_payees = Vec::new();
    let mut db_iter = all_db_payees.iter();
    let mut db_payee = db_iter.next();
    for payee in all_qif_transactions {
        match &db_payee {
            Some(db_payee2) => {
                // only increment db iter if there is not a new value
                if payee.raw_payee_name != db_payee2.1.raw_payee_name {
                    new_payees.push(payee.clone());
                } else {
                    db_payee = db_iter.next();
                }
            }

            None => {
                // if the db iter is finished add all remaining payees
                // it doesn't really matter whether we increment the iterator since it should just keep returning None when empty, but probably best not too because some iterator implementations could start again from the beginning once they reach the end
                new_payees.push(payee.clone());
            }
        }
    }

    // insert new payees into db
    SqlPayee::write_payees_to_db(&new_payees, &conn)?;

    // get transactions for qifs
    // get all payees from db again, now that we might have added new ones
    let all_db_transactions = SqlPayee::get_payees_from_db(&conn);
    let all_db_accounts = SqlAccount::get_accounts_from_db(&conn);

    let mut santander_transactions = SqlTransaction::get_transactions_from_qif_dir(
        santander_path,
        "Santander",
        1,
        None,
        &all_db_transactions,
        &all_db_accounts,
    );
    let mut monzo_transactions = SqlTransaction::get_transactions_from_qif_dir(
        monzo_path,
        "Monzo",
        2,
        None,
        &all_db_transactions,
        &all_db_accounts,
    );
    santander_transactions.append(&mut monzo_transactions);
    let mut all_qif_transcations = santander_transactions;

    // todo the below sorting is only necessary because
    // define correct sequence of sorting to be able to dedup - I think I might be going crazy here, the order of the fields sorted surely doesn't matter. a duplicate is a duplicate. even if we sorted again by amount at the end, any duplicates would still be next to each other
    all_qif_transcations.sort_by(|t1, t2| t1.amount.cmp(&t2.amount));
    all_qif_transcations.sort_by(|t1, t2| t1.date.cmp(&t2.date));
    all_qif_transcations.sort_by(|t1, t2| t1.payee_id.cmp(&t2.payee_id));
    all_qif_transcations.sort_by(|t1, t2| t1.account_id.cmp(&t2.account_id));

    // check for duplicates
    let (dedup, duplicates) = all_qif_transcations.partition_dedup_by(|t1, t2| {
        t1.amount == t2.amount
            && t1.date == t2.date
            && t1.payee_id == t2.payee_id
            && t1.account_id == t2.account_id
    });

    // get db transactions
    let all_db_transactions = SqlTransaction::get_transactions_from_db(&conn);
    dbg!(&all_db_transactions.len());
    // all_db_transactions.sort_by(|p1, p2| p1.1.raw_payee_name.cmp(&p2.1.raw_payee_name));

    // subset to only new transactions
    // first dedupe (nope, don't need to dedupe) db transactions then step through both, identifying new cases as ones that appear in qif but not the db. need to query payees db tables to get payid to add to transaction
    // IMPORTANT will need to recalcuate accumlative balances in db if transactions have been removed (unless they are only removed from end of table) or if new transactions have been added (again, unless they are only added to the end of the table)
    // best/safest/cleanest approach is probably to just add/remove transactions wherever you want in the db table, then sort the table according to some fixed criteria, then iterate through the entire table updating the balance columns, which is very scalable since we don't need to read the entire table into memory or do a fancy sql statement (it is not straightfoward to do update based on value in different rows), we can just read one row at a time, generating single row update statements.
    // balance show only be updated per account per day. having it change per transaction makes no sense. where to store this data though? I think it's also best to just calculate it on demand because sometimes you might want it for total of all accounts - can easily just add the separate account balances. also the problem with calculating it on demand is that you need the whole table

    let mut new_transactions = Vec::new();
    let mut db_iter = all_db_transactions.iter();
    let mut t2x = db_iter.next();
    for t1 in all_qif_transcations {
        // println!("new iter");
        match &t2x {
            Some(t2) => {
                let (_, t2) = t2;
                // dbg!(&payee.raw_payee_name);
                // dbg!(&db_payee.1.raw_payee_name);

                // only increment db iter if there is not a new value
                if t1.amount == t2.amount
                    && t1.date == t2.date
                    && t1.payee_id == t2.payee_id
                    && t1.account_id == t2.account_id
                {
                    new_transactions.push(t1.clone());
                } else {
                    t2x = db_iter.next();
                }
            }

            None => {
                // if the db iter is finished add all remaining payees
                // it doesn't really matter whether we increment the iterator since it should just keep returning None when empty, but probably best not too because some iterator implementations could start again from the beginning once they reach the end
                new_transactions.push(t1.clone());
            }
        }
    }
    dbg!(&new_transactions.len());

    // insert new transactions into db
    SqlTransaction::write_transactions_to_db(&new_transactions, &conn)?;

    // update balances (per account, per day (so all transactions on the same day will show the same balance)) don't need to read whole table into memory, can just sort by rowid (because it is unique and doesn't change!!), date, account_id, in the select statement, then (assuming rusqlite allows iterating over the results rather than reading them all into memory. this is all I could find but doesn't actually say anything useful: https://www.sqlite.org/c3ref/step.html) simply iterate over table generating update statements, running them, then throwing them away.

    Ok(())
}
