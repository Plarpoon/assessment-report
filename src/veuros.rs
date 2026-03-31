use std::io::{self, Write};

use crate::toml::parser::Config;

const TOTAL: u32 = 50;

pub struct Assignment {
    pub name: String,
    pub amount: u32,
}

pub fn run(config: &Config) -> Vec<Assignment> {
    let my_name = config.general.my_name.trim();

    let peers: Vec<&str> = config
        .members
        .students
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.eq_ignore_ascii_case(my_name))
        .collect();

    loop {
        let assignments = collect_round(&peers);
        let total: u32 = assignments.iter().map(|a| a.amount).sum();

        if total < TOTAL {
            println!(
                "\n{} vEuro(s) left unassigned — all 50 must be distributed. Starting over.\n",
                TOTAL - total
            );
            continue;
        }

        if confirm("All 50 vEuros assigned. Are you sure? [y/N] ") {
            return assignments;
        }

        println!("Restarting...\n");
    }
}

fn collect_round(peers: &[&str]) -> Vec<Assignment> {
    let mut remaining = TOTAL;

    println!("Assign 50 vEuros across your group members:\n");

    peers
        .iter()
        .map(|&name| {
            let amount = ask_amount(name, remaining);
            remaining -= amount;
            Assignment {
                name: name.to_string(),
                amount,
            }
        })
        .collect()
}

fn ask_amount(name: &str, remaining: u32) -> u32 {
    loop {
        println!("  vEuros remaining: {remaining}");
        print!("  Assign to {name}: ");
        io::stdout().flush().unwrap();

        match read_line().trim().parse::<u32>() {
            Ok(n) if n <= remaining => return n,
            Ok(_) => println!("  Cannot assign more than {remaining} vEuros.\n"),
            Err(_) => println!("  Please enter a whole number.\n"),
        }
    }
}

fn confirm(prompt: &str) -> bool {
    print!("{prompt}");
    io::stdout().flush().unwrap();
    read_line().trim().eq_ignore_ascii_case("y")
}

fn read_line() -> String {
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    buf
}
