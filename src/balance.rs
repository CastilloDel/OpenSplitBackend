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

type GroupName = String;
type UserBalance = HashMap<GroupName, f64>;

pub fn compute_user_balance_by_group(user_nick: UserNick, groups: Vec<Group>) -> UserBalance {
    let mut user_balance = UserBalance::new();
    for group in groups {
        let mut group_balance = 0.0;
        for expense in group.expenses {
            if expense.payer == user_nick {
                group_balance += expense.amount;
            } else if expense.receivers.contains(&user_nick) {
                group_balance -= expense.amount / expense.receivers.len() as f64;
            }
        }
        user_balance.insert(group.name, group_balance);
    }
    user_balance
}
