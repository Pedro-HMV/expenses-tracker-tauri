// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    sync::Mutex,
};
use tauri::State;

const FILENAME: &str = "expenses.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Expense {
    pub name: String,
    pub cost: f32,
    pub paid: bool,
    pub due_date: u32,
}

impl Expense {
    fn new(name: String, due_date: u32) -> ExpenseBuilder {
        ExpenseBuilder {
            name,
            cost: None,
            paid: false,
            due_date,
        }
    }
}

pub struct ExpenseBuilder {
    name: String,
    cost: Option<f32>,
    paid: bool,
    due_date: u32,
}

impl ExpenseBuilder {
    pub fn cost(&mut self, cost: f32) -> &mut Self {
        self.cost = Some(cost);
        self
    }
    pub fn build(&self) -> Expense {
        Expense {
            name: self.name.clone(),
            cost: self.cost.unwrap_or_default(),
            paid: self.paid,
            due_date: self.due_date,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Content {
    expenses: Mutex<Option<Vec<Expense>>>,
    net_worth: Mutex<f32>,
    income: Mutex<f32>,
}

#[tauri::command]
fn get_content(state: State<Content>) -> Value {
    json!(
        {
            "expenses": *state.expenses.lock().unwrap(),
            "net_worth": *state.net_worth.lock().unwrap(),
            "income": *state.income.lock().unwrap()
        }
    )
}

#[tauri::command]
fn update_net_worth(state: State<Content>) {
    let income = *state.income.lock().unwrap();
    let sum = sum_expenses(&*state);
    *state.net_worth.lock().unwrap() = income - sum;
}

fn sum_expenses(data: &Content) -> f32 {
    if let Some(ref expenses) = *data.expenses.lock().unwrap() {
        return expenses.iter().map(|e| e.cost).sum::<f32>();
    }
    0.0
}

fn check_valid_day(day: u32) -> bool {
    let month = Local::now().month();
    let leap_year = Local::now().year() % 4 == 0;
    let valid = match month {
        _ if day < 1 => false,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => day <= 31,
        4 | 6 | 9 | 11 => day <= 30,
        2 if leap_year => day <= 29,
        2 => day <= 28,
        _ => false,
    };
    valid
}

fn read_file(file: &mut File) -> String {
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read file");
    contents
}

fn get_json(content: &str) -> Value {
    let json_content: Value = serde_json::from_str(content).unwrap_or(json!({
        "income": 0,
        "expenses": [],
        "net_worth": 0
    }));
    json_content
}

#[tauri::command]
fn write_file(state: State<Content>) -> Result<(), String> {
    let mut file = get_file();
    let json_content = get_content(state.clone());
    file.set_len(0).expect("Failed to clear file");
    file.seek(SeekFrom::Start(0))
        .expect("Failed to seek to start");
    file.write_all(
        serde_json::to_string_pretty(&json_content)
            .expect("Failed to serialize JSON in write file")
            .as_bytes(),
    )
    .expect("Failed to write to file");
    Ok(())
}

#[tauri::command]
fn add_expense(state: State<Content>, data: &str) -> Result<(), String> {
    let expense = get_json(data);
    if !check_valid_day(expense["due_date"].as_u64().unwrap() as u32) {
        return Err(format!("Invalid day for {}", Local::now().month()));
    }
    if let Some(ref mut expenses) = *state.expenses.lock().unwrap() {
        if let Some(index) = expenses
            .iter()
            .position(|e| e.name == expense["name"].as_str().unwrap())
        {
            return Err(format!(
                "Expense named {} already exists",
                expenses[index].name
            ));
        }
        expenses.push(
            Expense::new(
                expense["name"].as_str().unwrap().to_string(),
                expense["due_date"].as_u64().unwrap() as u32,
            )
            .cost(expense["cost"].as_f64().unwrap() as f32)
            .build(),
        );
        return Ok(());
    } else {
        *state.expenses.lock().unwrap() = Some(vec![Expense::new(
            expense["name"].as_str().unwrap().to_string(),
            expense["due_date"].as_u64().unwrap() as u32,
        )
        .cost(expense["cost"].as_f64().unwrap() as f32)
        .build()]);

        return Ok(());
    }
}

#[tauri::command]
fn remove_expense(state: State<Content>, title: &str) -> Result<(), String> {
    if let Some(ref mut expenses) = *state.expenses.lock().unwrap() {
        if let Some(index) = expenses.iter().position(|e| e.name == title) {
            expenses.remove(index);
            Ok(())
        } else {
            Err(format!("No expense named {title}"))
        }
    } else {
        Err("There are no expenses to delete".to_string())
    }
}

#[tauri::command]
fn edit_expense(state: State<Content>, data: &str) -> Result<(), String> {
    if let Some(ref mut expenses) = *state.expenses.lock().unwrap() {
        let expense = get_json(data);
        if let Some(index) = expenses
            .iter()
            .position(|e| e.name == expense["name"].as_str().unwrap())
        {
            expenses[index].name = expense["name"].as_str().unwrap().to_string();
            expenses[index].cost = expense["cost"].as_f64().unwrap() as f32;
            expenses[index].due_date = expense["due_date"].as_u64().unwrap() as u32;
            Ok(())
        } else {
            Err(format!("No expense named {}", expense["name"]))
        }
    } else {
        Err("There are no expenses to edit".to_string())
    }
}

#[tauri::command]
fn pay_expense(state: State<Content>, title: &str) -> Result<(), String> {
    if let Some(ref mut expenses) = *state.expenses.lock().unwrap() {
        if let Some(index) = expenses.iter().position(|e| e.name == title) {
            expenses[index].paid = expenses[index].paid;
            Ok(())
        } else {
            Err(format!("No expense named {}", title))
        }
    } else {
        Err("There are no expenses to pay".to_string())
    }
}

#[tauri::command]
fn reset_paid(state: State<Content>) -> Result<(), String> {
    if let Some(ref mut expenses) = *state.expenses.lock().unwrap() {
        for expense in expenses.iter_mut() {
            expense.paid = false;
        }
        Ok(())
    } else {
        Err("An error ocurred".to_string())
    }
}

fn get_file() -> File {
    let exe_path: String = match std::env::current_exe() {
        Ok(path) => path.display().to_string(),
        Err(error) => panic!("Problem getting exe path: {:?}", error),
    };
    let file_path = std::fmt::format(format_args!("{}\\{}", exe_path, FILENAME));
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_path)
        .expect("Failed to open file");
    file
}

fn main() {
    let mut file = get_file();
    let data = get_json(&read_file(&mut file));
    let expenses = data["expenses"]
        .as_array()
        .unwrap_or(&Vec::<Value>::new())
        .iter()
        .map(|expense| Expense {
            name: expense["name"].as_str().expect("aaaaaa").to_string(),
            cost: expense["cost"].as_f64().expect("bbbbbbbb") as f32,
            paid: expense["paid"].as_bool().expect("cccccccc"),
            due_date: expense["due_date"].as_u64().expect("dddddddd") as u32,
        })
        .collect::<Vec<Expense>>();
    let content = {
        let income = data["income"].as_f64().unwrap() as f32;
        let net_worth = data["net_worth"].as_f64().unwrap() as f32;
        Content {
            expenses: Mutex::new(Some(expenses)),
            net_worth: Mutex::new(net_worth),
            income: Mutex::new(income),
        }
    };

    tauri::Builder::default()
        .manage(content)
        .invoke_handler(tauri::generate_handler![
            add_expense,
            remove_expense,
            edit_expense,
            pay_expense,
            reset_paid,
            update_net_worth,
            write_file,
            get_content
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
