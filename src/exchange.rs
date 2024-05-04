use crate::schemas::{Group, Expense};
use crate::balance::compute_balance_from_group;

use serde::{Serialize};

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct PersonBalance {
    pub id: String,
    pub balance: f64
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct PeoplePair {
    pub person1: String,
    pub person2: String
}

#[derive(Clone, Debug, Serialize)]
pub struct Exchange {
    pub payer: String,
    pub receiver: String,
    pub amount: f64
}

fn get_reverse_expenses(expenses: Vec<Expense>) -> Vec<Exchange> {

    let mut balances_between_people: HashMap<PeoplePair, f64> = HashMap::new();
    
    // Simplify balance between couple of people
    for expense in expenses {
        let num_receivers = expense.receivers.len() as f64;
        for receiver in expense.receivers {
        
            let person1;
            let person2;
            let amount;
           
            if expense.payer < receiver {
                person1 = expense.payer.clone(); 
                person2 = receiver; 
                amount = expense.amount/num_receivers;
            } else {
                person1 = receiver; 
                person2 = expense.payer.clone();
                amount = -expense.amount/num_receivers;
            }
            
            let people_pair = PeoplePair{person1: person1, person2: person2 };
            balances_between_people.entry(people_pair).and_modify(|balance| *balance += amount).or_insert(amount);
        }
    }
    
    // Calculate exchanges
    let mut exchanges = Vec::new();
    
    for (people_pair, mut balance) in balances_between_people {
        let payer;
        let receiver;
        if balance > 0.0 {
          payer = people_pair.person2;
          receiver = people_pair.person1;
        } else {
          payer = people_pair.person1;
          receiver = people_pair.person2;
          balance = -balance;
        }
        
        exchanges.push(Exchange {payer: payer, receiver: receiver, amount: balance});
    }
    
    exchanges
}

fn get_simplified_balances(mut payers: Vec<PersonBalance>, mut receivers: Vec<PersonBalance>) -> Vec<Exchange> {
    let mut num_settled = 0;
    let people_to_be_settled = payers.len() + receivers.len();

    // order ascendent by balance
    payers.sort_by(|a, b| a.balance.partial_cmp(&b.balance).unwrap());
    receivers.sort_by(|a, b| a.balance.partial_cmp(&b.balance).unwrap());
    
    let mut exchanges: Vec<Exchange> = Vec::new();
    
    while num_settled < people_to_be_settled {
        let receiver = receivers.last_mut().unwrap();
        let payer = payers.last_mut().unwrap();
        
        let payer_name = payer.id.clone();
        let receiver_name = receiver.id.clone();

        if receiver.balance > payer.balance {
            exchanges.push(Exchange {payer: payer_name, receiver: receiver_name, amount: payer.balance});
            receiver.balance -= payer.balance;
            payers.pop();
            num_settled += 1;
        } else if receiver.balance < payer.balance {
            exchanges.push(Exchange {payer: payer_name, receiver: receiver_name, amount: receiver.balance});
            payer.balance -= receiver.balance;
            receivers.pop();
            num_settled += 1;
        } else {
            exchanges.push(Exchange {payer: payer_name, receiver: receiver_name, amount: payer.balance});
            payers.pop();
            receivers.pop();
            num_settled += 2;
        }
            
    }
    
    exchanges
}

pub fn get_exchanges_from_group(group: &Group) -> Vec<Exchange> {
    // Get how many amount of money debt or owe people
    let people_balances = compute_balance_from_group(group);
    
    // Divide people into payer and receiver
    let mut payers: Vec<PersonBalance> = Vec::new();
    let mut receivers: Vec<PersonBalance> = Vec::new();
    
    for (id, balance) in people_balances {
        if balance < 0.0 {
            payers.push(PersonBalance {id: id, balance: -balance});
        } else if balance > 0.0 {
            receivers.push(PersonBalance {id: id, balance: balance});
        }
    }

    let not_simplified_exchanges = get_reverse_expenses(group.expenses.clone());
    let simplified_exchanges = get_simplified_balances(payers, receivers);
    
    let exchanges;
    if simplified_exchanges.len() < not_simplified_exchanges.len() {
        exchanges = simplified_exchanges;
    } else {
        exchanges = not_simplified_exchanges;
    }
    
    exchanges
}