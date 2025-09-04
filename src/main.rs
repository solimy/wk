use clap::{Parser, Subcommand};
use std::env;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum InfoPeriod {
    Day,
    Week,
    Month,
    Year,
}

#[derive(Subcommand)]
enum Commands {
    Start { name: String },
    Stop {},
    Info { period: Option<InfoPeriod> },
    Add { name: String },
    Remove { name: String },
    List {},
}

fn main() {
    let cli = Cli::parse();

    let home_dir = env::home_dir().unwrap_or(PathBuf::from("."));
    let db_path = home_dir.join("wk").join("db.sqlite");
    std::fs::create_dir_all(home_dir.join("wk")).unwrap();
    let db_path_str = db_path.to_str().unwrap();
    let connection = sqlite::Connection::open(if cfg!(debug_assertions) {
        "dev.sqlite"
    } else {
        db_path_str
    })
    .unwrap();
    init_db(&connection);

    match &cli.command {
        Some(Commands::Start { name }) => {
            start_task(&connection, name);
        }
        Some(Commands::Stop {}) => {
            stop_task(&connection);
        }
        Some(Commands::Info { period }) => match period {
            Some(period) => info_tasks(&connection, period),
            None => info_tasks(&connection, &InfoPeriod::Day),
        },
        Some(Commands::Add { name }) => {
            add_task(&connection, name);
        }
        Some(Commands::Remove { name }) => {
            remove_task(&connection, name);
        }
        Some(Commands::List {}) => {
            list_tasks(&connection);
        }
        None => {}
    }
}

fn init_db(connection: &sqlite::Connection) {
    connection
        .execute("CREATE TABLE IF NOT EXISTS tasks (id INTEGER PRIMARY KEY, name TEXT UNIQUE)")
        .unwrap();
    connection.execute("CREATE TABLE IF NOT EXISTS runs (id INTEGER PRIMARY KEY, task_id INTEGER, start_time TIMESTAMP, end_time TIMESTAMP, FOREIGN KEY(task_id) REFERENCES tasks(id))").unwrap();
}

fn add_task(connection: &sqlite::Connection, name: &str) {
    let mut statement = connection
        .prepare("INSERT INTO tasks (name) VALUES (?)")
        .unwrap();
    statement.bind((1, name)).unwrap();
    statement.next().unwrap();
}

fn list_tasks(connection: &sqlite::Connection) {
    let mut statement = connection.prepare("SELECT id, name FROM tasks").unwrap();
    while let sqlite::State::Row = statement.next().unwrap() {
        let id: i64 = statement.read(0).unwrap();
        let name: String = statement.read(1).unwrap();
        println!("{}: {}", id, name);
    }
}

fn remove_task(connection: &sqlite::Connection, name: &str) {
    let mut statement = connection
        .prepare("DELETE FROM tasks WHERE name = ?")
        .unwrap();
    statement.bind((1, name)).unwrap();
    statement.next().unwrap();
}

fn start_task(connection: &sqlite::Connection, name: &str) {
    stop_task(connection);

    let mut statement = connection
        .prepare("SELECT id FROM tasks WHERE name = ?")
        .unwrap();
    statement.bind((1, name)).unwrap();
    if let sqlite::State::Row = statement.next().unwrap() {
        let task_id: i64 = statement.read(0).unwrap();
        let start_time = std::time::SystemTime::now();
        let mut insert_statement = connection
            .prepare("INSERT INTO runs (task_id, start_time) VALUES (?, ?)")
            .unwrap();
        insert_statement.bind((1, task_id)).unwrap();
        insert_statement
            .bind((
                2,
                start_time
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            ))
            .unwrap();
        insert_statement.next().unwrap();
    } else {
        println!("Task not found: {}", name);
    }
}

fn stop_task(connection: &sqlite::Connection) {
    let mut statement = connection
        .prepare("UPDATE runs SET end_time = ? WHERE end_time IS NULL")
        .unwrap();
    let end_time = std::time::SystemTime::now();
    statement
        .bind((
            1,
            end_time
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        ))
        .unwrap();
    statement.next().unwrap();
}

fn info_tasks(connection: &sqlite::Connection, period: &InfoPeriod) {
    fn run(connection: &sqlite::Connection, label: &str, start_expr: &str) {
        println!("Current {}:", label);

        let query = format!(r#"
            SELECT 
                name,
                (duration / 86400) || 'd ' ||
                ((duration % 86400) / 3600) || 'h ' ||
                ((duration % 3600) / 60) || 'm ' ||
                (duration % 60) || 's' AS human_duration,
                RANK() OVER (ORDER BY duration DESC) AS rank,
                COALESCE(
                    printf('%.0f%%', 100.0 * duration / NULLIF(SUM(duration) OVER (), 0)),
                    '0%%'
                ) AS pct
            FROM (
                SELECT tasks.name,
                    SUM(COALESCE(runs.end_time, strftime('%s','now')) - runs.start_time) AS duration
                FROM tasks
                LEFT JOIN runs ON tasks.id = runs.task_id
                WHERE runs.start_time >= {start_expr}
                GROUP BY tasks.id, tasks.name
            )
            ORDER BY duration DESC;
        "#, start_expr = start_expr);

        let mut statement = connection.prepare(&query).unwrap();
        while let sqlite::State::Row = statement.next().unwrap() {
            let name: String = statement.read(0).unwrap();
            let duration: String = statement.read(1).unwrap();
            let rank: i64 = statement.read(2).unwrap();
            let pct: String = statement.read(3).unwrap();
            println!("\t{}. {}: {} ({})", rank, name, duration, pct);
        }
    }

    match period {
        InfoPeriod::Day => run(connection, "day", "strftime('%s','now','start of day')"),
        InfoPeriod::Week => run(connection, "week", "strftime('%s','now','start of day','-6 days')"),
        InfoPeriod::Month => run(connection, "month", "strftime('%s','now','start of month')"),
        InfoPeriod::Year => run(connection, "year", "strftime('%s','now','start of year')"),
    }
}
