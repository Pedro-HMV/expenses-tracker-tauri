// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    sync::Mutex,
};
use tauri::State;

type MutExpenses = Mutex<Option<Expenses>>;

const FILENAME: &str = "expenses.json";

#[derive(Serialize, Deserialize, Debug)]
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
pub struct Expenses {
    expenses: Vec<Expense>,
}

#[tauri::command]
fn update_net_worth(data: String) -> Value {
    let mut json_content = get_json(&data);
    let income = json_content["income"].as_f64().unwrap() as f32;
    let expenses = json_content["expenses"]
        .as_array()
        .unwrap_or(&Vec::<Value>::new())
        .iter()
        .map(|expense| expense["cost"].as_f64().unwrap() as f32)
        .sum::<f32>();
    let net_worth = income - expenses;
    json_content["net_worth"] = json!(net_worth);
    json_content
}

fn sum_expenses(json_content: &Value) -> f32 {
    let expenses = json_content["expenses"]
        .as_array()
        .unwrap_or(&Vec::<Value>::new())
        .iter()
        .map(|expense| Expense {
            name: expense["name"].as_str().unwrap().to_string(),
            cost: expense["cost"].as_f64().unwrap() as f32,
            paid: expense["paid"].as_bool().unwrap(),
            due_date: expense["due_date"].as_u64().unwrap() as u32,
        })
        .collect::<Vec<Expense>>();
    expenses.iter().map(|expense| expense.cost).sum::<f32>()
}

fn check_valid_day(day: u32) -> bool {
    day > 0 && day <= 31
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
fn write_file(data: String) {
    let mut file = get_file();
    let mut json_content = get_json(&data);
    file.set_len(0).expect("Failed to clear file");
    file.seek(SeekFrom::Start(0))
        .expect("Failed to seek to start");
    file.write_all(
        serde_json::to_string_pretty(&json_content)
            .expect("Failed to serialize JSON in write file")
            .as_bytes(),
    )
    .expect("Failed to write to file");
}

#[tauri::command]
fn add_expense(data: String) -> Value {
    let mut json_content = get_json(&data);
    let mut input = String::new();
    let mut expenses = json_content["expenses"]
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
    println!("Digite o título da despesa: ");
    match std::io::stdin().read_line(&mut input) {
        Ok(_) => {
            let trimmed = input.trim();
            let name = trimmed.to_string();
            loop {
                input.clear();
                println!("Digite o dia de vencimento a cada mês");
                match std::io::stdin().read_line(&mut input) {
                    Ok(_) => {
                        let trimmed = input.trim();
                        if trimmed.len() > 2 {
                            println!("O dia deve conter no máximo 2 dígitos!");
                            continue;
                        } else {
                            match trimmed.parse::<u32>() {
                                Ok(value) if !check_valid_day(value) => {
                                    println!("Digite um dia válido!");
                                    continue;
                                }
                                Ok(value) => {
                                    let mut expense = Expense::new(name.clone(), value);
                                    input.clear();
                                    println!("Digite o valor da despesa (vazio = 0): ");
                                    match std::io::stdin().read_line(&mut input) {
                                        Ok(_) => {
                                            let trimmed = input.trim();
                                            if trimmed == "" {
                                                expenses.push(expense.build());
                                                input.clear()
                                            } else {
                                                match trimmed.parse::<f32>() {
                                                    Ok(value) => {
                                                        expense.cost(value);
                                                        expenses.push(expense.build());
                                                        input.clear();
                                                    }
                                                    Err(..) => {
                                                        println!("Digite um número válido!");
                                                        input.clear();
                                                    }
                                                }
                                            }
                                        }
                                        Err(error) => println!("error: {}", error),
                                    };
                                    break;
                                }
                                Err(..) => {
                                    println!("Digite um número válido!");
                                    input.clear();
                                }
                            };
                        }
                    }
                    Err(..) => println!("Digite um número válido!"),
                };
            }
        }
        Err(error) => println!("error: {}", error),
    }
    json_content["expenses"] = json!(expenses);
    json_content["net_worth"] = json!(update_net_worth(&json_content)["net_worth"]);
    json_content
}

#[tauri::command]
fn remove_expense(data: String) -> Value {
    let mut json_content = get_json(&data);
    let mut input = String::new();
    println!("Digite o título da despesa que deseja remover: ");
    match std::io::stdin().read_line(&mut input) {
        Ok(_) => {
            let trimmed = input.trim();
            let expenses = json_content["expenses"].as_array_mut().unwrap();
            let index = expenses
                .iter()
                .position(|expense| expense["name"] == trimmed);
            match index {
                Some(index) => {
                    expenses.remove(index);
                    input.clear();
                }
                None => println!("Despesa não encontrada!"),
            }
        }
        Err(error) => println!("error: {}", error),
    }
    json_content["net_worth"] = json!(update_net_worth(json_content)["net_worth"]);
    json_content
}

#[tauri::command]
fn edit_expense(data: String) -> Value {
    let mut json_content = get_json(&data);
    let mut input = String::new();
    println!("Digite o título da despesa que deseja editar: ");
    match std::io::stdin().read_line(&mut input) {
        Ok(_) => {
            let trimmed = input.trim();
            let expenses = json_content["expenses"].as_array_mut().unwrap();
            let index = expenses
                .iter()
                .position(|expense| expense["name"] == trimmed);
            match index {
                Some(index) => {
                    input.clear();
                    println!("Digite o novo título da despesa (vazio = manter): ");
                    match std::io::stdin().read_line(&mut input) {
                        Ok(_) => {
                            let trimmed = input.trim();
                            if trimmed != "" {
                                expenses[index]["name"] = json!(trimmed);
                            }
                            input.clear();
                        }
                        Err(error) => println!("error: {}", error),
                    }
                    println!("Digite o novo valor da despesa (vazio = manter): ");
                    match std::io::stdin().read_line(&mut input) {
                        Ok(_) => {
                            let trimmed = input.trim();
                            if trimmed != "" {
                                match trimmed.parse::<f32>() {
                                    Ok(value) => {
                                        expenses[index]["cost"] = json!(value);
                                        input.clear();
                                    }
                                    Err(..) => println!("Digite um número válido!"),
                                }
                            }
                        }
                        Err(error) => println!("error: {}", error),
                    }
                    loop {
                        println!("Digite o novo dia de vencimento (vazio = manter): ");
                        match std::io::stdin().read_line(&mut input) {
                            Ok(_) => {
                                let trimmed = input.trim();
                                if trimmed != "" {
                                    match trimmed.parse::<u32>() {
                                        Ok(value) if check_valid_day(value) => {
                                            expenses[index]["due_date"] = json!(value);
                                            input.clear();
                                            break;
                                        }
                                        Ok(_) => {
                                            println!("Digite um dia válido!");
                                            input.clear();
                                            continue;
                                        }
                                        Err(..) => println!("Digite um número válido!"),
                                    }
                                }
                            }
                            Err(error) => println!("error: {}", error),
                        }
                    }
                }
                None => println!("Despesa não encontrada!"),
            }
        }
        Err(error) => println!("error: {}", error),
    }
    json_content["net_worth"] = json!(update_net_worth(json_content)["net_worth"]);
    json_content
}

#[tauri::command]
fn pay_expense(data: String) -> Value {
    let mut json_content = get_json(&data);
    let mut input = String::new();
    println!("Digite o título da despesa que deseja pagar: ");
    match std::io::stdin().read_line(&mut input) {
        Ok(_) => {
            let trimmed = input.trim();
            let expenses = json_content["expenses"].as_array_mut().unwrap();
            let index = expenses
                .iter()
                .position(|expense| expense["name"] == trimmed);
            match index {
                Some(index) => {
                    expenses[index]["paid"] = json!(true);
                    input.clear();
                }
                None => println!("Despesa não encontrada!"),
            }
        }
        Err(error) => println!("error: {}", error),
    }
    json_content
}

#[tauri::command]
fn reset_paid(data: String) -> Value {
    let mut json_content = get_json(&data);
    let expenses = json_content["expenses"].as_array_mut().unwrap();
    for expense in expenses {
        expense["paid"] = json!(false);
    }
    json_content
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn get_file() -> File {
    let exe_path: String = match std::env::current_exe() {
        Ok(path) => path.display().to_string(),
        Err(error) => panic!("Problem getting exe path: {:?}", error),
    };
    let file_path = std::fmt::format(format_args!("{}\\{}", exe_path, FILENAME));
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_path)
        .expect("Failed to open file");
    file
}

fn main() {
    let mut file = get_file();
    let mut json_content = read_file(&mut file);
    let mut expenses = json_content["expenses"]
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

    tauri::Builder::default()
        .manage(Mutex::new(None::<Expenses>))
        .invoke_handler(tauri::generate_handler![
            greet,
            add_expense,
            remove_expense,
            edit_expense,
            pay_expense,
            reset_paid,
            update_net_worth,
            write_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
