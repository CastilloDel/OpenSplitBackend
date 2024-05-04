use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub expenses: Vec<Expense>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Expense {
    pub name: String,
    pub amount: f64,
    pub payer: String,
    pub receivers: Vec<String>,
}
