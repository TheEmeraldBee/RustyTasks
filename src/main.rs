use std::{collections::HashMap, env, fs};

use clap::{Parser, ValueEnum};
use prettytable::Table;
use rand::Rng;
use sqlx::{FromRow, SqlitePool};

#[macro_use]
extern crate prettytable;
use prettytable::format::consts;

#[derive(Debug, Clone, Parser, PartialEq)]
#[command(name = "Rusty Tasks")]
#[command(version, author)]
enum Args {
    /// Add A New Task
    Add {
        /// The group of tasks to be stored in
        #[arg(short, long)]
        folder: String,
        /// What the task is you want to complete
        #[arg(short, long)]
        task: String,
    },
    /// Remove A Task
    Delete {
        /// The Unique id of the task
        #[arg(short, long)]
        id: u8,
    },
    /// List The Tasks
    List {
        /// (OPTIONAL) The folder that you want to list
        #[arg(short, long)]
        folder: Option<String>,
    },
    /// Update a task, changing the task, its folder, or its status.
    Update {
        /// The unique id of the task
        #[arg(short, long)]
        id: u8,
        /// (OPTIONAL) the new task
        #[arg(short, long)]
        task: Option<String>,
        /// (OPTIONAL) the new folder for the task
        #[arg(short, long)]
        folder: Option<String>,
        /// (OPTIONAL) the new status for the task,
        #[arg(value_enum, short, long)]
        status: Option<Status>,
        /// (OPTIONAL) sets a custom status instead
        #[arg(short, long)]
        custom_status: Option<String>,
    },
}

#[derive(Clone, Debug, Default, ValueEnum, PartialEq)]
pub enum Status {
    #[default]
    Incomplete,
    InProgress,
    Complete,
}

impl Status {
    pub fn to_string(&self) -> &'static str {
        match self {
            Status::Incomplete => "Incomplete",
            Status::InProgress => "In Progress",
            Status::Complete => "Complete",
        }
    }
}

#[derive(FromRow, Debug)]
pub struct Task {
    pub folder: String,
    pub task: String,
    pub id: i64,
    pub status: String,
}

pub fn format_task(task: &str) -> String {
    textwrap::fill(task, 25)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let directory = directories::UserDirs::new().expect("You don't have a home directory?");
    let directory = directory.home_dir();
    let mut directory = directory
        .to_str()
        .expect("Your directory isn't a valid string.")
        .to_string();
    directory.push_str("/.tasks");

    fs::create_dir_all(&directory)?;
    env::set_current_dir(&directory)?;

    let args = Args::parse();

    let db_url = "sqlite://tasks.db?mode=rwc";

    let pool = SqlitePool::connect(db_url).await?;

    {
        let mut conn = pool.acquire().await?;

        if sqlx::query(
            "CREATE TABLE tasks 
(
folder VARCHAR(255) NOT NULL,
task LONGTEXT NOT NULL,
id INT NOT NULL,
status VARCHAR(255) NOT NULL
);",
        )
        .execute(&mut conn)
        .await
        .is_ok()
        {
            println!("Created Database!");
        }
    }

    match args {
        Args::Add { folder, task } => add_task(&pool, folder, task).await?,
        Args::Delete { id } => remove_task(&pool, id).await?,
        Args::List { folder } => list_tasks(&pool, folder).await?,
        Args::Update {
            id,
            task,
            folder,
            status,
            custom_status,
        } => update_task(&pool, id, task, folder, status, custom_status).await?,
    }

    Ok(())
}

async fn add_task(pool: &SqlitePool, folder: String, task: String) -> anyhow::Result<()> {
    let mut conn = pool.acquire().await?;
    let mut rng = rand::thread_rng();

    let mut value;

    // Make Sure we get a ticket id that doesn't exist
    loop {
        value = rng.gen_range(0..u8::MAX);

        let values = sqlx::query!("SELECT * FROM tasks WHERE id = ?1", value)
            .fetch_all(pool)
            .await?;

        if values.is_empty() {
            break;
        }
    }

    // Create The Task
    sqlx::query!(
        "INSERT INTO tasks VALUES (?1, ?2, ?3, ?4)",
        folder,
        task,
        value,
        "Incomplete"
    )
    .execute(&mut conn)
    .await?;

    let task = sqlx::query!("SELECT * FROM TASKS WHERE id = ?1", value)
        .fetch_one(pool)
        .await?;

    let mut table = table!([task.id, task.folder, task.status, format_task(&task.task)]);
    table.set_titles(row![bFg=>"Task\n\nID", "Created!\n\nFolder", "\n\nStatus", "\n\nTask"]);
    table.set_format(*consts::FORMAT_BORDERS_ONLY);
    table.print_tty(true)?;

    Ok(())
}

async fn remove_task(pool: &SqlitePool, id: u8) -> anyhow::Result<()> {
    // Verify that the task exists
    let task = match sqlx::query_as!(Task, "SELECT * FROM tasks WHERE id = ?1", id)
        .fetch_one(pool)
        .await
    {
        Ok(task) => task,
        Err(_) => {
            ptable!(["    ", Fr->"No Task with that ID", "    "]);
            return Ok(());
        }
    };

    let mut table = table!([task.id, task.folder, format_task(&task.task), task.status]);
    table.set_titles(row![bFg=>"ID", "Folder", "Task", "Status"]);
    table.print_tty(true)?;

    ptable!([cH2Fr=>"Are You Sure?"], [c=>"Y", "N"]);

    let mut confirmation = String::new();
    std::io::stdin().read_line(&mut confirmation)?;
    confirmation = confirmation.to_uppercase().trim().to_string();

    if confirmation != "Y" && confirmation != "YES" {
        ptable!([bcFg->"Deletion Cancled"]);
        return Ok(());
    }

    let mut conn = pool.acquire().await?;

    let rows = sqlx::query!("DELETE FROM tasks WHERE id = ?1", id)
        .execute(&mut conn)
        .await?;

    let mut table = Table::new();
    table.set_titles(row!["", bFg->"Result", ""]);

    if rows.rows_affected() == 0 {
        table.add_row(row!["   ", Fr->"No Task with That Name", "   "]);
    } else {
        table.add_row(row!["   ", Fb->"Successfully Deleted\nTask", Fb->id]);
    }

    table.print_tty(true)?;

    Ok(())
}

async fn update_task(
    pool: &SqlitePool,
    id: u8,
    task: Option<String>,
    folder: Option<String>,
    status: Option<Status>,
    custom_status: Option<String>,
) -> anyhow::Result<()> {
    let mut conn = pool.acquire().await?;

    let mut query = "UPDATE tasks SET ".to_string();

    let mut has_task = false;

    if let Some(task) = task {
        query.push_str(&format!("task=\"{}\", ", task));
        has_task = true;
    }
    if let Some(folder) = folder {
        query.push_str(&format!("folder=\"{}\", ", folder));
        has_task = true
    }
    if let Some(custom_status) = custom_status {
        query.push_str(&format!("status=\"{}\",  ", custom_status));
        has_task = true;
    } else if let Some(status) = status {
        query.push_str(&format!("status=\"{}\",  ", status.to_string()));
        has_task = true;
    }

    if !has_task {
        ptable!([bFr=>"ERROR!", "Please Use an Update Flag!"]);
        return Ok(());
    }

    query = query.trim().to_string();
    query.remove(query.len() - 1);

    query.push_str(&format!("WHERE id = {}", id as i64));

    sqlx::query(&query).execute(&mut conn).await?;

    if let Ok(task) = sqlx::query_as!(Task, "SELECT * FROM tasks WHERE id=?1", id)
        .fetch_one(pool)
        .await
    {
        let mut table = table!([task.id, task.folder, task.status, format_task(&task.task)]);
        table.set_titles(row![bFg=>"Task\n\nID", "Updated!\n\nFolder", "\n\nStatus", "\n\nTask"]);
        table.set_format(*consts::FORMAT_BORDERS_ONLY);
        table.print_tty(true)?;
    } else {
        table!([bFr->"ERROR\nTask Doesn't Exist", Fb->format!("ID\n{}", id)]).print_tty(true)?;
    }

    Ok(())
}

async fn list_tasks(pool: &SqlitePool, folder: Option<String>) -> anyhow::Result<()> {
    if let Some(folder) = folder {
        // List the tasks in the folder specified, if empty, tell user it is empty
        let tasks = sqlx::query_as!(
            Task,
            "SELECT * FROM tasks WHERE folder=?1 ORDER BY folder ASC, task ASC",
            folder
        )
        .fetch_all(pool)
        .await?;

        let mut table = Table::new();
        table.set_format(*consts::FORMAT_NO_LINESEP_WITH_TITLE);
        table.set_titles(row![bFg->"Folder\nID", bFg->format!("{folder}\nTask"), bFg->"\nStatus"]);

        if tasks.is_empty() {
            table.add_row(row!["No Tasks Here!"]);
        }
        for task in tasks {
            table.add_row(row![task.id, task.task, task.status]);
        }

        table.print_tty(true)?;
    } else {
        let mut table = Table::new();
        table.set_format(*consts::FORMAT_DEFAULT);

        table.set_titles(row![bFg->"Folder", bFg->"Tasks"]);

        let mut task_tables: HashMap<String, Table> = HashMap::new();

        // List All Tasks Here In a Heirarchy view
        let tasks = sqlx::query_as!(Task, "SELECT * FROM tasks ORDER BY folder ASC, task ASC")
            .fetch_all(pool)
            .await?;

        if tasks.is_empty() {
            table.add_row(row!["No Tasks Here!"]);
        }

        for task in tasks {
            if let Some(task_table) = task_tables.get_mut(&task.folder) {
                task_table.add_row(row![task.id, format_task(&task.task), task.status]);
            } else {
                let mut task_table = Table::new();
                task_table.set_format(*consts::FORMAT_NO_BORDER);
                task_table.set_titles(row![bFb->"ID", bFb->"Task", bFb->"Status"]);
                task_table.add_row(row![task.id, format_task(&task.task), task.status]);
                task_tables.insert(task.folder, task_table);
            }
        }

        for task_table in task_tables.iter() {
            table.add_row(row![task_table.0, task_table.1]);
        }

        table.print_tty(true)?;
    }

    Ok(())
}
