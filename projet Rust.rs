use clap::{Parser, Subcommand};
use dialoguer::{Input, Select};
use rusqlite::{params, Connection, Result};

#[derive(Parser)]
#[command(name = "budget_manager")]
#[command(about = "A simple budget manager CLI application", long_about = None)]
struct Cli {
    command: Commands,
}
enum Commands {
    AddBudget { name: String, amount: f64 },
    RemoveBudget { id: i32 },
    EditBudget { id: i32, name: Option<String>, amount: Option<f64> },
    AddTransaction { budget_id: i32, amount: f64 },
    RemoveTransaction { id: i32 },
    EditTransaction { id: i32, amount: f64 },
    ShowBudgets,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let conn = Connection::open("budget_manager.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS budgets (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            amount REAL NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS transactions (
            id INTEGER PRIMARY KEY,
            budget_id INTEGER NOT NULL,
            amount REAL NOT NULL,
            FOREIGN KEY(budget_id) REFERENCES budgets(id)
        )",
        [],
    )?;

    match &cli.command {
        Commands::AddBudget { name, amount } => {
            conn.execute("INSERT INTO budgets (name, amount) VALUES (?1, ?2)", params![name, amount])?;
            println!("Budget added");
        }
        Commands::RemoveBudget { id } => {
            conn.execute("DELETE FROM budgets WHERE id = ?1", params![id])?;
            conn.execute("DELETE FROM transactions WHERE budget_id = ?1", params![id])?;
            println!("Budget removed");
        }
        Commands::EditBudget { id, name, amount } => {
            if let Some(name) = name {
                conn.execute("UPDATE budgets SET name = ?1 WHERE id = ?2", params![name, id])?;
            }
            if let Some(amount) = amount {
                conn.execute("UPDATE budgets SET amount = ?1 WHERE id = ?2", params![amount, id])?;
            }
            println!("Budget updated");
        }
        Commands::AddTransaction { budget_id, amount } => {
            conn.execute("INSERT INTO transactions (budget_id, amount) VALUES (?1, ?2)", params![budget_id, amount])?;
            println!("Transaction added");
        }
        Commands::RemoveTransaction { id } => {
            conn.execute("DELETE FROM transactions WHERE id = ?1", params![id])?;
            println!("Transaction removed");
        }
        Commands::EditTransaction { id, amount } => {
            conn.execute("UPDATE transactions SET amount = ?1 WHERE id = ?2", params![amount, id])?;
            println!("Transaction updated");
        }
        Commands::ShowBudgets => {
            let mut stmt = conn.prepare("SELECT id, name, amount FROM budgets")?;
            let budget_iter = stmt.query_map([], |row| {
                Ok(Budget {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    amount: row.get(2)?,
                })
            })?;

            println!("Budgets:");
            for budget in budget_iter {
                let budget = budget?;
                let remaining = calculate_remaining(&conn, budget.id)?;
                println!("ID: {}, Name: {}, Amount: {}, Remaining: {}", budget.id, budget.name, budget.amount, remaining);
            }
        }
    }
    Ok(())
}

struct Budget {
    id: i32,
    name: String,
    amount: f64,
}

fn calculate_remaining(conn: &Connection, budget_id: i32) -> Result<f64> {
    let mut stmt = conn.prepare("SELECT SUM(amount) FROM transactions WHERE budget_id = ?1")?;
    let total: f64 = stmt.query_row(params![budget_id], |row| row.get(0)).unwrap_or(0.0);
    let budget_amount: f64 = conn.query_row("SELECT amount FROM budgets WHERE id = ?1", params![budget_id], |row| row.get(0))?;
    Ok(budget_amount - total)
}