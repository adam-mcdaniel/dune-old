use xmachine::*;
use crate::shell::Shell;

#[derive(Debug)]
pub enum Error {}

pub trait Execute {
    fn execute(&self, _: &mut Shell) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Literal {
    String(String),
    Number(f64),
}

impl Execute for Literal {
    fn execute(&self, shell: &mut Shell) -> Result<(), Error> {
        shell.machine.push(match self {
            Self::String(s) => xmachine::Value::string(s),
            Self::Number(n) => xmachine::Value::number(n.clone()),
        });
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct FnCall(pub Box<Value>, pub Vec<Value>);

impl Execute for FnCall {
    fn execute(&self, shell: &mut Shell) -> Result<(), Error> {
        let FnCall(function, mut arguments) = self.clone();
        arguments.reverse();
        for arg in arguments {
            arg.execute(shell)?;
        }
        function.execute(shell)?;
        shell.machine.call();
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Identifier(pub String);

impl Execute for Identifier {
    fn execute(&self, shell: &mut Shell) -> Result<(), Error> {
        let Identifier(name) = self;
        shell.machine.push(xmachine::Value::string(name));
        shell.machine.load();
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Builtin {
    List,
    ChangeDir,
    Move,
    Remove,
    WorkingDir,
    Exit
}

impl Execute for Builtin {
    fn execute(&self, shell: &mut Shell) -> Result<(), Error> {
        match self {
            Self::List => {
                let arg = shell.machine.pop()
                                .map(|v| (*v).clone().to_string());
                shell.ls(arg);
            },
            Self::ChangeDir => {
                let arg = shell.machine.get_arg::<String>();
                shell.cd(&arg);
            },
            Self::Move => {
                let old = shell.machine.get_arg::<String>();
                let new = shell.machine.get_arg::<String>();
                shell.mv(&old, &new);
            },
            Self::WorkingDir => {
                shell.wd();
            },
            Self::Exit => shell.exit(),
            _ => {}
        };

        Ok(())
    }
}


#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Value {
    Name(Name),
    Literal(Literal),
    FnCall(FnCall),
    Builtin(Builtin)
}

impl Execute for Value {
    fn execute(&self, shell: &mut Shell) -> Result<(), Error> {
        match self {
            Self::Name(name) => name.execute(shell)?,
            Self::Literal(literal) => literal.execute(shell)?,
            Self::FnCall(call) => call.execute(shell)?,
            Self::Builtin(call) => call.execute(shell)?,
        };
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Name {
    Name(Identifier),
    DotName(Box<Value>, Vec<Identifier>),
}

impl Execute for Name {
    fn execute(&self, shell: &mut Shell) -> Result<(), Error> {
        match self {
            Self::Name(name) => name.execute(shell)?,
            Self::DotName(head, identifiers) => {
                head.execute(shell)?;
                for ident in identifiers {
                    let Identifier(name) = ident;
                    shell.machine.push(xmachine::Value::string(name));
                    shell.machine.index();
                }
            }
        };
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Expr {
    Assignment(Name, Value),
    WhileLoop(Value, Suite),
    IfThenElse(Value, Suite, Suite),
    Value(Value),
}

impl Execute for Expr {
    fn execute(&self, shell: &mut Shell) -> Result<(), Error> {
        match self {
            Self::Assignment(name, value) => match name {
                Name::Name(ident) => {
                    let Identifier(store) = ident;
                    value.execute(shell)?;
                    shell.machine.push(xmachine::Value::string(store));
                    shell.machine.store();
                }
                dotname => {
                    value.execute(shell)?;
                    dotname.execute(shell)?;
                    shell.machine.assign();
                }
            },
            Self::WhileLoop(value, body) => {
                let ret_val = |shell: &mut Shell| match shell.machine.pop() {
                    Some(v) => bool::from((*v).clone()),
                    _ => false,
                };

                value.execute(shell)?;
                while ret_val(shell) {
                    body.execute(shell)?;
                    value.execute(shell)?;
                }
            }
            Self::IfThenElse(value, then_body, else_body) => {
                let ret_val = |shell: &mut Shell| match shell.machine.pop() {
                    Some(v) => bool::from((*v).clone()),
                    _ => false,
                };

                value.execute(shell)?;
                if ret_val(shell) {
                    then_body.execute(shell)?;
                } else {
                    else_body.execute(shell)?;
                }
            }
            Self::Value(v) => v.execute(shell)?,
        };
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Suite(pub Vec<Expr>);

impl Execute for Suite {
    fn execute(&self, shell: &mut Shell) -> Result<(), Error> {
        let Suite(exprs) = self;
        for expr in exprs {
            expr.execute(shell)?;
        }

        Ok(())
    }
}