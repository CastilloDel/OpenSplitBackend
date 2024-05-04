use std::collections::HashMap;

use crate::schemas::Group;

type Balance = HashMap<String, f64>;

pub fn compute_balance_from_group(group: Group) -> Balance {
    let mut balance = Balance::new();
    for expense in group.expenses {
        let amount = expense.amount;
        balance
            .entry(expense.payer)
            .and_modify(|v| *v += amount)
            .or_insert(expense.amount);
        let amount_per_receiver = amount / expense.receivers.len() as f64;
        for receiver in expense.receivers {
            balance
                .entry(receiver)
                .and_modify(|v| *v -= amount_per_receiver)
                .or_insert(-amount_per_receiver);
        }
    }
    balance
}
