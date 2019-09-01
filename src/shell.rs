use xmachine::{Machine, Value};
use read_input::prelude::*;
use crate::parser::program;
use crate::tokens::Execute;


use std::fs::{read_dir, rename};
use dirs::home_dir;
use std::path::PathBuf;
use std::process::exit;
use crossterm::{terminal, ClearType};


fn to_string(path: &PathBuf) -> String {
    path.to_str().unwrap().to_string()
}


pub struct Shell {
    pub directory: PathBuf,
    pub machine: Machine
}

impl Shell {
    pub fn new() -> Self {
        Self {
            directory: home_dir().unwrap(),
            machine: machine()
        }
    }

    pub fn run(&mut self) {
        loop {
            print!("{}$ ", to_string(&self.directory));
            let command: String = input().get();

            match program().parse(&command) {
                Ok(v) => {
                    v.execute(self);
                    self.clear_stack();
                },
                Err(e) => println!("Error: {:?}", e)
            };
        }
    }

    pub fn clear_stack(&mut self) {
        while let Some(_) = self.machine.pop() {}
    }

    pub fn wd(&self) {
        println!("{}", to_string(&self.directory));
    }

    pub fn mv(&self, old: &str, new: &str) {
        let mut old_dir = self.directory.clone();
        old_dir.push(old);
        let mut new_dir = self.directory.clone();
        new_dir.push(new);
        rename(old_dir, new_dir);
    }

    pub fn ls(&self, dir: Option<String>) {
        match dir {
            Some(d) => {
                let mut result_dir = self.directory.clone();
                result_dir.push(d);

                for name in read_dir(result_dir) {
                    println!("{:?}", name);
                }
            },
            None => {
                for name in read_dir(self.directory.clone()).unwrap() {
                    println!("{:?}", name);
                }
            }
        }
    }

    pub fn cd(&mut self, dir: &str) {
        self.directory.push(dir);
        self.directory = self.directory.canonicalize().unwrap();
    }

    pub fn exit(&mut self) { exit(0); }
}

fn add_fn(m: &mut Machine, function: fn(&mut Machine) -> (), name: &str) {
    m.push(Value::function(function, &m));
    m.push(Value::string(name));
    m.store();
}

fn machine() -> Machine {
    let m = &mut Machine::new();
    add_fn(m, |m| {println!("{}", match m.pop() {
        Some(v) => v,
        None => Value::string("")
    });}, "echo");
    add_fn(m, |_| {
        let mut terminal = terminal();
        terminal.clear(ClearType::All);
    }, "clear");
    add_fn(m, |m| { m.push(Value::tree()); }, "dict");

    m.clone()
}