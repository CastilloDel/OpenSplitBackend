use serde::Serialize;

use crate::schemas::{Group, UserNick};
use std::collections::HashMap;

type GroupBalance = HashMap<UserNick, f64>;

pub fn compute_balance_from_group(group: &Group) -> GroupBalance {
    let mut balance = GroupBalance::new();
    for expense in &group.expenses {
        let amount = expense.amount;
        balance
            .entry(expense.payer.clone())
            .and_modify(|v| *v += amount)
            .or_insert(expense.amount);
        let amount_per_receiver = amount / expense.receivers.len() as f64;
        for receiver in &expense.receivers {
            balance
                .entry(receiver.clone())
                .and_modify(|v| *v -= amount_per_receiver)
                .or_insert(-amount_per_receiver);
        }
    }
    balance
}

#[derive(Serialize)]
pub struct UserGroupBalance {
    group_id: String,
    group_name: String,
    amount: f64,
}

pub fn compute_user_balance_by_group(
    user_nick: UserNick,
    groups: Vec<Group>,
) -> Vec<UserGroupBalance> {
    let mut balances = Vec::new();
    for group in groups {
        let mut balance = UserGroupBalance {
            group_id: group.id,
            group_name: group.name,
            amount: 0.0,
        };
        for expense in group.expenses {
            if expense.payer == user_nick {
                balance.amount += expense.amount;
            } else if expense.receivers.contains(&user_nick) {
                balance.amount -= expense.amount / expense.receivers.len() as f64;
            }
        }
        balances.push(balance);
    }
    balances
}
