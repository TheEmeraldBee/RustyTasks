use std::{collections::BTreeMap, env, fs};

use anyhow::anyhow;
use clap::Parser;
use prettytable::{Row, Table};
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

        /// (OPTIONAL) How deep you want the recursive print of a folder to go (Default 2)
        #[arg(short, default_value_t = 3)]
        depth: u32,
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
        /// (OPTIONAL) sets the status of the task
        #[arg(short, long)]
        status: Option<String>,
    },
}

#[derive(Debug, Default)]
pub struct Folder {
    pub tasks: Vec<Task>,
    pub subfolders: BTreeMap<String, Folder>,
}

pub fn verify_path(path: String) -> anyhow::Result<()> {
    if path.trim() == "" {
        return Ok(());
    }

    let path: Vec<_> = path.split('/').collect();

    for folder in &path {
        if folder.trim() == "" {
            return Err(anyhow!("Empty Folder Name Found\n\nHelp: Ensure there are no trailing slashes or double slashes!"));
        }
    }

    Ok(())
}

impl Folder {
    fn add_task(&mut self, mut task: Task) -> anyhow::Result<()> {
        if task.folder.trim() == "" {
            // This is the correct path.
            self.tasks.push(task);
            return Ok(());
        }

        let mut path: Vec<_> = task.folder.split('/').collect();

        let cur = path.first().expect("Unreachable").to_string();
        path.remove(0);

        if cur.trim() == "" {
            return Err(anyhow!("Empty Folder Name Found\n\nHelp: Ensure there are no trailing slashes or double slashes!"));
        }

        // Get the remaining path
        let mut remaining_path = "".to_string();
        for remaining in &path {
            remaining_path.push_str(&format!("{remaining}/"));
        }

        if !remaining_path.is_empty() {
            remaining_path.remove(remaining_path.len() - 1);
        }

        task.folder = remaining_path.to_string();

        // Ensure the subfolder exists
        if !self.subfolders.contains_key(&cur) {
            self.subfolders.insert(cur.to_string(), Folder::default());
        }

        // Send this path to that subfolder
        self.subfolders.get_mut(&cur).unwrap().add_task(task)?;

        Ok(())
    }
    fn to_table(&self, max_depth: i32) -> Table {
        let mut result = Table::new();
        result.set_format(*consts::FORMAT_NO_BORDER);
        result.set_titles(row![bFg=>"Type", "Contents"]);

        // Show the folders first.
        if max_depth > 1 {
            for folder in &self.subfolders {
                result.add_row(row![
                    &format!("Folder\n{}", folder.0),
                    folder.1.to_table(max_depth - 1)
                ]);
            }
        } else {
            for _folder in &self.subfolders {
                result.add_row(row!["Folder", "------"]);
            }
        }

        // Now display this folder's tasks
        for task in &self.tasks {
            result.add_row(task.to_row());
        }

        result
    }
}

#[derive(FromRow, Debug, Clone)]
pub struct Task {
    pub folder: String,
    pub task: String,
    pub id: i64,
    pub status: String,
}

impl Task {
    pub fn to_row(&self) -> Row {
        let mut task_display = table![[self.id, format_task(&self.task), self.status]];
        task_display.set_titles(row!["ID", "Task", "Status"]);
        task_display.set_format(*consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
        row!["Task", task_display]
    }
}

pub fn format_task(task: &str) -> String {
    let mut result = task.to_string();
    while result.len() < 25 {
        result.push(' ');
    }
    textwrap::fill(&result, 25)
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
        Args::List { folder, depth } => list_tasks(&pool, folder, depth).await?,
        Args::Update {
            id,
            task,
            folder,
            status,
        } => update_task(&pool, id, task, folder, status).await?,
    }

    Ok(())
}

async fn add_task(pool: &SqlitePool, folder: String, task: String) -> anyhow::Result<()> {
    verify_path(folder.clone())?;
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

    let mut table = table!([task.id, task.folder, &task.status, format_task(&task.task)]);
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
        ptable!([bcFg->"Deletion Canceled"]);
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
    status: Option<String>,
) -> anyhow::Result<()> {
    let mut conn = pool.acquire().await?;

    let mut query = "UPDATE tasks SET ".to_string();

    let mut has_task = false;

    if let Some(task) = task {
        query.push_str(&format!("task=\"{}\", ", task));
        has_task = true;
    }
    if let Some(folder) = folder {
        verify_path(folder.clone())?;
        query.push_str(&format!("folder=\"{}\", ", folder));
        has_task = true
    }
    if let Some(status) = status {
        query.push_str(&format!("status=\"{}\",  ", status));
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

async fn list_tasks(pool: &SqlitePool, folder: Option<String>, depth: u32) -> anyhow::Result<()> {
    let mut tasks;
    if let Some(folder) = folder {
        verify_path(folder.clone())?;
        // List the tasks in the folder specified, if empty, tell user it is empty
        let mut query_folder = folder.clone();
        query_folder.push('%');
        tasks = sqlx::query_as!(
            Task,
            "SELECT * FROM tasks WHERE folder LIKE ?1 ORDER BY folder ASC, task ASC",
            query_folder
        )
        .fetch_all(pool)
        .await?;

        for task in &mut tasks {
            let mut replace_string = folder.clone();
            replace_string.push('/');
            let result = task
                .folder
                .to_lowercase()
                .replacen(&replace_string.to_lowercase(), "", 1);

            while task.folder.len() > result.len() {
                task.folder.remove(0);
            }
        }
    } else {
        tasks = sqlx::query_as!(Task, "SELECT * FROM tasks ORDER BY folder ASC, task ASC")
            .fetch_all(pool)
            .await?;
    }

    let mut result = Folder::default();
    for task in tasks {
        result.add_task(task)?;
    }

    let mut print_table = result.to_table(depth as i32);
    print_table.set_format(*consts::FORMAT_DEFAULT);

    if print_table.is_empty() {
        print_table.add_row(row![cH2->"No Tasks Here!"]);
    }

    print_table.print_tty(true)?;

    Ok(())
}
