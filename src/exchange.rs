use crate::balance::compute_balance_from_group;
use crate::schemas::{Expense, Group, UserNick};
use serde::Serialize;
use std::collections::HashMap;
use std::mem::swap;

#[derive(Clone, Debug)]
pub struct PersonalBalance {
    pub id: UserNick,
    pub balance: f64,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct UserPair {
    pub user1: UserNick,
    pub user2: UserNick,
}

#[derive(Clone, Debug, Serialize)]
pub struct Exchange {
    pub payer: UserNick,
    pub receiver: UserNick,
    pub amount: f64,
}

// The exchanges that will be made if no simplification happens
fn get_naive_exchanges(expenses: Vec<Expense>) -> Vec<Exchange> {
    let mut balances_between_people: HashMap<UserPair, f64> = HashMap::new();

    for expense in expenses {
        let num_receivers = expense.receivers.len() as f64;
        for receiver in expense.receivers {
            let mut pair = UserPair {
                user1: expense.payer.clone(),
                user2: receiver.clone(),
            };
            let mut amount = expense.amount / num_receivers;

            // We use alphabetical order to ensure all the expenses regarding
            // the same users end up stored in the same direction
            if pair.user1 > pair.user2 {
                pair.user1 = receiver;
                pair.user2 = expense.payer.clone();
                amount = -amount;
            }

            balances_between_people
                .entry(pair)
                .and_modify(|balance| *balance += amount)
                .or_insert(amount);
        }
    }

    // Calculate exchanges, now the ones that payed will be the receivers
    let mut exchanges = Vec::new();

    for (people_pair, balance) in balances_between_people {
        let mut payer = people_pair.user2;
        let mut receiver = people_pair.user1;
        // If the balance is smaller than zero we change the direction
        if balance < 0.0 {
            swap(&mut payer, &mut receiver);
        }

        exchanges.push(Exchange {
            payer,
            receiver,
            amount: balance.abs(),
        });
    }

    exchanges
}

// Tries to simplify the number of exchanges
fn get_simplified_balances(
    mut payers: Vec<PersonalBalance>,
    mut receivers: Vec<PersonalBalance>,
) -> Vec<Exchange> {
    payers.sort_by(|a, b| a.balance.partial_cmp(&b.balance).unwrap());
    receivers.sort_by(|a, b| a.balance.partial_cmp(&b.balance).unwrap());

    let mut exchanges: Vec<Exchange> = Vec::new();

    while payers.len() + receivers.len() > 0 {
        let receiver = receivers.last_mut().unwrap();
        let payer = payers.last_mut().unwrap();

        let mut exchange = Exchange {
            payer: payer.id.clone(),
            receiver: receiver.id.clone(),
            amount: 0.0,
        };
        if receiver.balance == payer.balance {
            exchange.amount = payer.balance;
            payers.pop();
            receivers.pop();
        } else if receiver.balance > payer.balance {
            exchange.amount = payer.balance;
            receiver.balance = round_to_2_decimals(receiver.balance - payer.balance);
            payers.pop();
        } else {
            exchange.amount = receiver.balance;
            payer.balance = round_to_2_decimals(payer.balance - receiver.balance);
            receivers.pop();
        }
        exchanges.push(exchange);
    }
    exchanges
}

fn round_to_2_decimals(n: f64) -> f64 {
    (n * 100.0).round() / 100.0
}

pub fn get_exchanges_from_group(group: &Group) -> Vec<Exchange> {
    let people_balances = compute_balance_from_group(group);

    // Divide people into payers and receivers
    let mut payers = Vec::new();
    let mut receivers = Vec::new();

    for (id, balance) in people_balances {
        let person = PersonalBalance {
            id,
            balance: balance.abs(),
        };
        if balance < 0.0 {
            payers.push(person);
        } else {
            receivers.push(person);
        }
    }

    let naive_exchanges = get_naive_exchanges(group.expenses.clone());
    let simplified_exchanges = get_simplified_balances(payers, receivers);

    // We ensure the simplification didn't accidentally end up being
    // more complicated than the naive exchanges
    if simplified_exchanges.len() < naive_exchanges.len() {
        simplified_exchanges
    } else {
        naive_exchanges
    }
}
