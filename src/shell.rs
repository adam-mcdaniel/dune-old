use crate::parser::program;
use crate::tokens::Execute;
use crate::{LOGO, INFO};
use read_input::prelude::*;
use xmachine::{Machine, Ref, Value};

use dirs::home_dir;
use std::process::{Command, Stdio};
use std::fs::{create_dir_all, read_dir, remove_dir_all, remove_file, rename, write};
use std::path::PathBuf;

fn to_string(path: &PathBuf) -> String {
    path.to_str().unwrap().to_string()
}

#[derive(Clone)]
pub struct Shell {
    pub directory: PathBuf,
    pub machine: Machine,
    pub is_done: bool
}

impl Shell {
    pub fn new() -> Self {
        Self {
            directory: home_dir().unwrap(),
            machine: machine(),
            is_done: false
        }
    }

    pub fn run(&mut self) {
        while !self.is_done {
            print!("{}$ ", to_string(&self.directory));
            let mut command = String::from("");
            let mut user_input = input::<String>().get();
            command += &user_input;
            while !program().parse(&command).is_ok() && !(user_input.trim() == "") {
                user_input = input()
                    .msg(" ".repeat(to_string(&self.directory).len()) + "> ")
                    .get();
                command += &user_input;
            }

            match program().parse(&command) {
                Ok(v) => {
                    match v.execute(self) {
                        _ => {}
                    };
                    self.print_stack();
                    self.clear_stack();
                }
                Err(e) => println!("Error: {:?}", e),
            };
        }
    }

    pub fn print_stack(&mut self) {
        while let Some(value) = self.machine.pop() {
            println!("{}", value);
        }
    }

    pub fn clear_stack(&mut self) {
        while let Some(_) = self.machine.pop() {}
    }

    pub fn wd(&mut self) {
        self.machine.push(Value::string(to_string(&self.directory)));
    }

    pub fn mv(&self, old: &str, new: &str) {
        let mut old_dir = self.directory.clone();
        old_dir.push(old);
        let mut new_dir = self.directory.clone();
        new_dir.push(new);
        match rename(old_dir, new_dir) {
            _ => {}
        };
    }

    pub fn rm(&self, path: &str) {
        if path == "" {
            return;
        }
        let directory = {
            let mut result = self.directory.clone();
            result.push(path);
            result
        };

        match remove_dir_all(directory.clone()) {
            _ => {}
        };
        match remove_file(directory) {
            _ => {}
        };
    }

    pub fn mkdir(&self, path: &str) {
        if path == "" {
            return;
        }
        let directory = {
            let mut result = self.directory.clone();
            result.push(path);
            result
        };

        match create_dir_all(directory) {
            _ => {}
        };
    }

    pub fn mkf(&self, path: &str) {
        if path == "" {
            return;
        }
        let directory = {
            let mut result = self.directory.clone();
            result.push(path);
            result
        };

        match write(directory, "") {
            _ => {}
        };
    }

    pub fn ls(&mut self, dir: Option<String>) {
        let directory = match dir {
            Some(d) => {
                let mut result_dir = self.directory.clone();
                result_dir.push(d);
                result_dir
            }
            None => self.directory.clone(),
        };

        let mut result = vec![];
        match read_dir(directory) {
            Ok(dir) => {
                for name in dir {
                    result.push(Value::string(
                        name.unwrap().path().file_name().unwrap().to_str().unwrap(),
                    ));
                }
            }
            _ => {}
        }

        self.machine.push(Ref::new(Value::List(result)));
    }

    pub fn cd(&mut self, dir: &str) {
        let mut result = self.directory.clone();
        result.push(dir);
        self.directory = match result.canonicalize() {
            Ok(dir) => dir,
            _ => self.directory.clone(),
        };
    }

    pub fn sh(&mut self, cmd: &str) {
        let components = cmd.split_whitespace().collect::<Vec<&str>>();
        if !components.is_empty() {
            match Command::new(components[0])
                .args(components[1..].iter())
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .output() { _ => {} };
        }
    }

    pub fn clear(&mut self) {
        println!("{}", "\n".repeat(200));
    }

    pub fn exit(&mut self) {
        self.is_done = true;
    }
}

fn add_fn(m: &mut Machine, function: fn(&mut Machine) -> (), name: &str) {
    m.push(Value::function(function, &m));
    m.push(Value::string(name));
    m.store();
}

fn add_const(m: &mut Machine, value: impl Into<Value>, name: &str) {
    m.push(Ref::new(value.into()));
    m.push(Value::string(name));
    m.store();
}

fn machine() -> Machine {
    let m = &mut Machine::new();
    add_const(m, 1, "true");
    add_const(m, 0, "false");
    add_fn(
        m,
        |m| {
            print!(
                "{}",
                match m.pop() {
                    Some(v) => v,
                    None => Value::string(""),
                }
            );
        },
        "print",
    );
    add_fn(
        m,
        |m| {
            println!(
                "{}",
                match m.pop() {
                    Some(v) => v,
                    None => Value::string(""),
                }
            );
        },
        "println",
    );
    add_fn(
        m,
        |m| {
            m.push(Value::tree());
        },
        "dict",
    );
    add_fn(
        m,
        |m| {
            let function = match m.pop() {
                Some(f) => f,
                None => Value::function(|_| {}, &m),
            };

            let list = m.get_arg::<Vec<Ref<xmachine::Value>>>();

            for item in list {
                m.push(item);
                m.push(function.clone());
                m.call();
            }
        },
        "map",
    );
    add_fn(
        m,
        |m| {
            let a = match m.pop() {
                Some(v) => (*v).clone(),
                _ => return,
            };
            m.return_value(!a);
        },
        "not",
    );
    add_fn(
        m,
        |m| {
            let a = m.pop();
            let b = m.pop();
            m.push(Value::number((a == b) as i32))
        },
        "eq",
    );
    add_fn(
        m,
        |m| {
            let a = m.pop();
            let b = m.pop();
            m.push(Value::number((a != b) as i32))
        },
        "neq",
    );
    add_fn(
        m,
        |m| {
            let a = m.get_arg::<f64>();
            let b = m.get_arg::<f64>();
            m.push(Value::number((a > b) as i32))
        },
        "gt",
    );
    add_fn(
        m,
        |m| {
            let a = m.get_arg::<f64>();
            let b = m.get_arg::<f64>();
            m.push(Value::number((a < b) as i32))
        },
        "lt",
    );
    add_fn(
        m,
        |m| {
            let a = m.get_arg::<f64>();
            let b = m.get_arg::<f64>();
            m.push(Value::number((a <= b) as i32))
        },
        "le",
    );
    add_fn(
        m,
        |m| {
            let a = m.get_arg::<f64>();
            let b = m.get_arg::<f64>();
            m.push(Value::number((a >= b) as i32))
        },
        "ge",
    );
    add_fn(
        m,
        |m| {
            let a = match m.pop() {
                Some(v) => (*v).clone(),
                _ => return,
            };
            let b = match m.pop() {
                Some(v) => (*v).clone(),
                _ => return,
            };
            m.return_value(a + b)
        },
        "add",
    );
    add_fn(
        m,
        |m| {
            let a = match m.pop() {
                Some(v) => (*v).clone(),
                _ => return,
            };
            let b = match m.pop() {
                Some(v) => (*v).clone(),
                _ => return,
            };
            m.return_value(a - b)
        },
        "sub",
    );
    add_fn(
        m,
        |m| {
            let a = match m.pop() {
                Some(v) => (*v).clone(),
                _ => return,
            };
            let b = match m.pop() {
                Some(v) => (*v).clone(),
                _ => return,
            };
            m.return_value(a * b)
        },
        "mul",
    );
    add_fn(
        m,
        |m| {
            let a = match m.pop() {
                Some(v) => (*v).clone(),
                _ => return,
            };
            let b = match m.pop() {
                Some(v) => (*v).clone(),
                _ => return,
            };
            m.return_value(a / b)
        },
        "div",
    );
    add_fn(
        m,
        |m| {
            let a = match m.pop() {
                Some(v) => (*v).clone(),
                _ => return,
            };
            let b = match m.pop() {
                Some(v) => (*v).clone(),
                _ => return,
            };
            m.return_value(a % b)
        },
        "rem",
    );
    add_fn(
        m,
        |m| {
            print!(
                "{}",
                match m.pop() {
                    Some(v) => v,
                    None => Value::string(""),
                }
            );

            m.push(Value::string(input::<String>().get().trim()));
        },
        "input",
    );
    add_fn(
        m,
        |m| {
            let command = match m.pop() {
                Some(v) => (*v).clone(),
                _ => return,
            };

            match program().parse(&format!("{}", command)) {
                Ok(v) => {
                    let shell = &mut Shell::new();
                    match v.execute(shell) {
                        _ => {}
                    };
                    shell.print_stack();
                    shell.clear_stack();
                }
                Err(e) => println!("Error: {:?}", e),
            };
        },
        "eval",
    );
    add_fn(
        m,
        |m| {
            println!("{}", INFO);
            println!("{}", m);
        },
        "help",
    );
    add_fn(
        m,
        |m| {
            println!("{}", INFO);
            println!("{}", m);
        },
        "debug",
    );
    add_fn(
        m,
        |m| {
            println!("{}", INFO);
            println!("{}", m);
        },
        "info",
    );
    add_fn(
        m,
        |_| {
            println!("{}", INFO);
            println!("{}", LOGO);
        },
        "logo",
    );

    m.clone()
}
