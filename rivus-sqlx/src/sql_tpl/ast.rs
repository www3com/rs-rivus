use crate::sql_tpl::value::{SqlParam, Value};

#[derive(Debug, Clone)]
pub enum AstNode {
    Text(String),
    Var(String),
    Include { refid: String },
    If { test: String, body: Vec<AstNode> },
    For { item: String, collection: String, open: String, sep: String, close: String, body: Vec<AstNode> },
}

pub struct RenderBuffer {
    pub sql: String,
    pub params: Vec<SqlParam>,
}

pub struct Context<'a> {
    root: &'a Value,
    locals: Vec<(String, &'a Value)>,
}

impl<'a> Context<'a> {
    pub fn new(root: &'a Value) -> Self {
        Self {
            root,
            locals: Vec::new(),
        }
    }

    pub fn push(&mut self, key: &str, value: &'a Value) {
        self.locals.push((key.to_string(), value));
    }

    pub fn pop(&mut self) {
        self.locals.pop();
    }

    pub fn lookup(&self, key: &str) -> &'a Value {
        // First check locals (stack) in reverse order
        for (k, v) in self.locals.iter().rev() {
            if k == key {
                return v;
            }
        }
        
        // Then check root
        match self.root {
            Value::Map(m) => m.get(key).unwrap_or(&Value::Null),
            _ => &Value::Null,
        }
    }
}
