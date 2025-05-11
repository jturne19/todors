#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui;
use egui_extras::{TableBuilder, Column};
use chrono::prelude::*;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::rc::Rc;
use std::cell::RefCell;

// create public struct for a TODO item
#[derive(Clone, Default)]
pub struct TodoStruct {
    text: String,
    date_added: String,
    completed: bool,
    date_completed: String
}

impl TodoStruct {
    // implement a clear function to reset a TODO item
    pub fn clear(&mut self) {
        self.text.clear();
        self.date_added.clear();
        self.completed = false;
        self.date_completed.clear();
    }

    // marks an item as completed and adds the current Date
    pub fn completed(&mut self) {
        self.completed = true;
        self.date_completed = Utc::now().date_naive().to_string();
    }

    // marks an item as not completed and clears out the date_completed field
    pub fn not_completed(&mut self) {
        self.completed = false;
        self.date_completed.clear();
    }
}
// set static variables for the markdown file names
static TODOS_FILENAME: &str = "todos.md";
static DONES_FILENAME: &str = "done_todos.md";

// Save the todo and done lists to their Markdown files
fn save_todos_to_file(todo_list: &Vec<TodoStruct>, done_list: &Vec<TodoStruct>, filename: &str, filename_done: &str) -> std::io::Result<()> {
    let mut file = fs::File::create(filename)?;
    let header = "# TODOs\n";
    file.write(header.as_bytes())?;

    let mut file2 = fs::File::create(filename_done)?;
    let header2 = "# DONEs\n";
    file2.write(header2.as_bytes())?;

    for todo in todo_list {
        let line = format!(
            "- ({}) {}\n",
            todo.date_added, todo.text
        );
        file.write_all(line.as_bytes())?;
    }
    for done in done_list {
        let line = format!(
            "- DONE (Completed {}, Added {}) {}\n",
            done.date_completed, done.date_added, done.text
        );
        file2.write(line.as_bytes())?;
    }
    Ok(())
}

// load the TODOs from the todo Markdown file
// expects "# TODOs" in the file to know it has TODOs to read in
// expect each todo line in the file to look like "- (<DATE>) My TODO text here"
// E.g, "- (2025-05-11) Need to do something important"
fn load_todos_from_file(filename: &str) -> std::io::Result<Vec<TodoStruct>> {
    let file = match fs::File::open(filename) {
        Ok(file) => file,
        Err(_) => return Ok(Vec::new()),
    };
    let reader = BufReader::new(file);
    let mut todos = Vec::new();
    let mut reading_todos = false;

    for line_result in reader.lines() {
        let line = line_result?;
        let trimmed_line = line.trim();

        if trimmed_line == "# TODOs" {
            reading_todos = true;
            continue;
        }

        if reading_todos && trimmed_line.starts_with("- (") && trimmed_line.contains(") "){
            if let Some(start_paren) = trimmed_line.find("(") {
                if let Some(end_paren) = trimmed_line.find(")") {
                    if start_paren < end_paren && start_paren == 2 && trimmed_line.starts_with("- "){
                        let date_added = trimmed_line[(start_paren + 1)..end_paren].trim().to_string();
                        let text_start = end_paren + 2;
                        if text_start < trimmed_line.len() {
                            let text = trimmed_line[text_start..].trim().to_string();
                            todos.push(TodoStruct {
                                text,
                                date_added,
                                completed: false,
                                date_completed: String::new(),
                            })
                        }
                    }
                }
            }
        }

    }
    Ok(todos)
}

// load the DONEs from the done Markdown file
// expects "# DONEs" in the file to know it has TODOs to read in
// expect each todo line in the file to look like "- DONE (Completed <DATE>, Added <DATE>) My TODO text here"
// E.g, "- DONE (Completed 2025-05-11, Added 2025-05-10) pee pee poo poo"
fn load_dones_from_file(filename: &str) -> std::io::Result<Vec<TodoStruct>> {
    let file = match fs::File::open(filename) {
        Ok(file) => file,
        Err(_) => return Ok(Vec::new()),
    };
    let reader = BufReader::new(file);
    let mut dones = Vec::new();
    let mut reading_dones = false;

    for line_result in reader.lines() {
        let line = line_result?;
        let trimmed_line = line.trim();

        if trimmed_line == "# DONEs" {
            reading_dones = true;
            continue;
        }

        if reading_dones && trimmed_line.starts_with("- DONE (Completed ") && trimmed_line.contains(", Added ") {
            // "- DONE (Completed 2025-05-10, Added 2025-05-09) my task to do something"
            let parts: Vec<&str> = trimmed_line.splitn(2, ") ").collect();
            if parts.len() == 2 {
                let metadata_part = parts[0];
                let text = parts[1].trim().to_string();

                if metadata_part.starts_with("- DONE (Completed ") && metadata_part.contains(", Added ") {
                    let completed_start = "- DONE (Completed ".len();
                    if let Some(comma_index) = metadata_part[completed_start..].find(',') {
                        let date_completed = metadata_part[completed_start..(completed_start + comma_index)].trim().to_string();

                        let added_start_offset = ", Added ".len();
                        if let Some(added_comma_index) = metadata_part.find(", Added ") {
                            let added_start = added_comma_index + added_start_offset;
                            let added_end = metadata_part.len() - 1; // Remove the trailing ')'
                            if added_start < added_end {
                                let date_added = metadata_part[added_start..added_end].trim().to_string();
                                dones.push(TodoStruct {
                                    text,
                                    date_added,
                                    completed: true,
                                    date_completed,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(dones)
}

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    // set the default window size
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    // initialize string for the user-input todo text
    let mut new_todo_text = String::new();
    // the RefCell stuff allows for mutable shared ownership of the todo_list and done_list
    let todo_list = Rc::new(RefCell::new(Vec::<TodoStruct>::new()));
    let todo_list_clone = todo_list.clone();
    let done_list = Rc::new(RefCell::new(Vec::<TodoStruct>::new()));
    let done_list_clone = done_list.clone();
    // initialize an empty TodoStruct item which will be filled in with the user-nput
    let mut new_todo = TodoStruct {text: "".to_string(), date_added: "".to_string(), completed: false, date_completed: "NA".to_string()};

    // Load TODOs from file
    {
        match load_todos_from_file(TODOS_FILENAME) {
            Ok(loaded_todos) => {
                *todo_list_clone.borrow_mut() = loaded_todos;
            }
            Err(e) => eprintln!("Error loading todos: {}; skipping", e)
        }
    }
    // Load DONEs from file
    {
        match load_dones_from_file(DONES_FILENAME) {
            Ok(loaded_dones) => {
                *done_list_clone.borrow_mut() = loaded_dones;
            }
            Err(e) => eprintln!("Error done list: {}; skipping", e)
        }
    }

    // Run the egui app
    eframe::run_simple_native("todors", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let response = ui.add(egui::TextEdit::singleline(&mut new_todo_text).hint_text("Add new TODO here"));
            let button_response = ui.button("Add TODO").clicked();
            // if user presses enter or if the button is pressed, then add the new TODO
            if (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) || button_response {
                new_todo.text = new_todo_text.trim().to_string();
                new_todo.date_added = Utc::now().date_naive().to_string();
                todo_list_clone.borrow_mut().insert(0, new_todo.clone());
                new_todo.clear();
                new_todo_text.clear();

                // save both lists to file
                let current_todo_list = todo_list_clone.borrow();
                let current_done_list = done_list_clone.borrow();
                match save_todos_to_file(&current_todo_list, &current_done_list, TODOS_FILENAME, DONES_FILENAME) {
                    Ok(_) => (),
                    Err(e) => eprintln!("Error saving todo list: {}", e)
                }
            }
            ui.separator();
            ui.label("Current TODOs:");
            // create a scrollable area for displaying the current TODOs
            egui::ScrollArea::vertical().show(ui, |ui| {
                // create a Table for displaying the current TODOs
                TableBuilder::new(ui)
                    .striped(true)
                    .column(Column::auto())// column for the checkbox
                    .column(Column::remainder()) // column for the todo text
                    .body(|mut body| {
                        let todo_list_data = {
                            let todo_list = todo_list_clone.borrow();
                            todo_list.clone()
                        };

                        // borrow mutable todo list and done list
                        let mut todo_list_mut = todo_list_clone.borrow_mut();
                        let mut done_list_mut = done_list_clone.borrow_mut();
                        // loop through all the todos in the todo list to show them in the table
                        for (row_index, mut todo_item) in todo_list_data.into_iter().enumerate() {
                            body.row(20.0, |mut row| {
                                // first column is the checkbox for if the item is completed or not
                                row.col(|ui| {
                                    let mut checked = todo_item.completed;
                                    if ui.checkbox(&mut checked, "").changed() {
                                        if checked {
                                            // move it to the done list
                                            let mut moved_item = todo_list_mut.remove(row_index);
                                            moved_item.completed();
                                            done_list_mut.insert(0, moved_item);
                                        } else{
                                            // otherwise leave it as not completed. probably not strictly necessary
                                            todo_item.not_completed();
                                        }
                                        // save the lists on change
                                        match save_todos_to_file(&todo_list_mut, &done_list_mut, TODOS_FILENAME, DONES_FILENAME) {
                                            Ok(_) => (),
                                            Err(e) => eprintln!("Error saving todo list: {}", e)
                                        }
                                    }
                                });
                                // second column is the todo text simply printed out
                                row.col(|ui| {
                                    ui.label(format!("{}", todo_item.text));
                                });
                            });
                        }
                    });
            });
            ui.separator();
            // create a collapsing dialog for the DONEs
            ui.collapsing("DONEs", |ui| {
                // create a scrollable area for displaying the DONEs
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // create a table for the DONEs similar to the TODOs
                    TableBuilder::new(ui)
                        .striped(true)
                        .column(Column::auto())
                        .column(Column::remainder())
                        .body(|mut body| {
                            let done_list_data = {
                                let done_list = done_list_clone.borrow();
                                done_list.clone()
                            };

                            let mut done_list_mut = done_list_clone.borrow_mut();
                            let mut todo_list_mut = todo_list_clone.borrow_mut();

                            for (row_index, mut done_item) in done_list_data.into_iter().enumerate() {
                                body.row(20.0, |mut row| {
                                    row.col(|ui| {
                                        let mut checked = done_item.completed;
                                        if ui.checkbox(&mut checked, "").changed() {
                                            if !checked {
                                                // gotta move it back to the todo list
                                                let mut moved_item = done_list_mut.remove(row_index);
                                                moved_item.not_completed();
                                                todo_list_mut.insert(0, moved_item);
                                                // save the lists on change
                                                match save_todos_to_file(&todo_list_mut, &done_list_mut, TODOS_FILENAME, DONES_FILENAME) {
                                                    Ok(_) => (),
                                                    Err(e) => eprintln!("Error saving todo list: {}", e)
                                                }
                                            } else {
                                                done_item.completed();
                                            }
                                        }
                                    });
                                    row.col(|ui| {
                                        ui.label(format!("{}", done_item.text));
                                    });
                                });
                            }
                        });
                });
            });
        });
    })
}