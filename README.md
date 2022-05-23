# thesis

Three common approaches to working with data are:
1. Spreadsheets like MS Excel
2. Programming languages like R and Python
3. Full/professional data engineering stacks which are often backed up by some kind of database, use a dedicated dashboarding tool for data visualisation, use dedicated tools for ETL, make use of cloud computing for flexibility in resources, make use of tools like Apache Spark, Big Query, Apache Kafka.

Each approach has pros and cons. This project aims to provide a tool which combines the pros of each approach without any of the cons. A tool which is:
1. Easy to use, intuitive, and accessible like Excel.
2. Flexible, composable and open with a large ecosystem full of reusable code, like R and Python
3. Robust and powerful like a professional data engineering stack.


# usage

requires nightly. cd into the repo and run the below.
```bash
rustup toolchain install nightly
rustup override set nightly
```

run `cargo run --bin sqlite_editor -- -s load_businesses.sql` to compile and run the sqlite editor and load in some example data.