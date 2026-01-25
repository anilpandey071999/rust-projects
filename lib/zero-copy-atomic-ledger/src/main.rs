use std::{
    collections::HashMap,
    f64::consts::E,
    str::FromStr,
    time::{self, Duration, SystemTime},
};

use thiserror::Error;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

pub type Ids = [u8; 16];
pub type AmountType = u128;
pub const BASE_UNIT_OF_AMOUNT: u32 = 1_00_000_000;

#[derive(Error, Debug)]
pub enum LedgerError {
    #[error("Insufficient Balance")]
    InsufficientBalance,
    #[error("Invalid Account ID: {0}")]
    InvalidAccount(&'static str),
    #[error("Negative Value")]
    NegativeValue,
    #[error("Account Already Exists")]
    AccountAlreadyExists,
    #[error("Account Not Found")]
    AccountNotFound,
    #[error("Invalid Command")]
    InvalidCommand,
    #[error("Failed to parse command {0}")]
    InvalidParsing(&'static str),
    #[error("Different Currency ")]
    DifferentCurrency,
    #[error("Double-Spend")]
    DoubleSpend,
    #[error("LedgerError Timeout")]
    Timeout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    CreateAccount {
        account_id: Ids,
        currency: Currency,
        starting_balance: AmountType,
    },
    Transfer {
        transaction_id: Ids,
        from: Ids,
        to: Ids,
        amount: AmountType,
    },
    Shutdown,
}

impl FromStr for Command {
    type Err = LedgerError;

    fn from_str(s: &str) -> Result<Self, LedgerError> {
        if s.is_empty() {
            return Err(LedgerError::InvalidCommand);
        }
        let s = s.trim().split_ascii_whitespace().collect::<Vec<&str>>();
        if s.len() != 4 {
            return Err(LedgerError::InvalidCommand);
        }
        match s[0] {
            "create account" | "Create Account" => {
                let currency = s[2].parse::<Currency>().map_err(|err| {
                    println!("Failed to create account due to {err}");
                    LedgerError::InvalidCommand
                })?;
                let balance = s[3].parse::<u128>().map_err(|err| {
                    println!("Failed to create account due to {err}");
                    LedgerError::InvalidCommand
                })?;
                Ok(Command::CreateAccount {
                    account_id: Uuid::new_v4().as_bytes().clone(),
                    currency: currency,
                    starting_balance: balance,
                })
            }
            "transfer" | "Transfer" => {
                let from = s[1]
                    .parse::<Uuid>()
                    .map_err(|err| {
                        println!("Failed to transfer due to {err}");
                        LedgerError::InvalidCommand
                    })?
                    .as_bytes()
                    .clone();
                let to = s[2]
                    .parse::<Uuid>()
                    .map_err(|err| {
                        println!("Failed to transfer due to {err}");
                        LedgerError::InvalidCommand
                    })?
                    .as_bytes()
                    .clone();
                let amount = s[3].parse::<u128>().map_err(|err| {
                    println!("Failed to transfer due to {err}");
                    LedgerError::InvalidCommand
                })?;
                Ok(Command::Transfer {
                    transaction_id: Uuid::new_v4().as_bytes().clone(),
                    from,
                    to,
                    amount,
                })
            }
            _ => Err(LedgerError::InvalidCommand),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    NewAccountAdded {
        id: Ids,
    },
    MoneyTransfered {
        from: Ids,
        to: Ids,
        amount: AmountType,
    },
    Shutdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Currency {
    USD,
    EUR,
    BTC,
}

impl FromStr for Currency {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        match s {
            "usd" | "USD" => Ok(Currency::USD),
            "eur" | "EUR" => Ok(Currency::EUR),
            "btc" | "BTC" => Ok(Currency::BTC),
            _ => Err("Not supported currency"),
        }
    }
}

#[derive(Debug)]
pub struct UserAccount {
    account_id: Ids,
    currency: Currency,
    balance: AmountType,
}

#[derive(Debug)]
pub struct Transaction {
    id: Ids,
    sender: Ids,
    receiver: Ids,
    amount: AmountType,
    currency: Currency,
}

#[derive(Debug)]
pub struct Ledger {
    ledger_tx: HashMap<Ids, (Transaction, SystemTime)>,
    ledger_balance: HashMap<Ids, UserAccount>,
}

impl Ledger {
    pub fn new() -> Self {
        Self {
            ledger_tx: HashMap::new(),
            ledger_balance: HashMap::new(),
        }
    }

    pub fn process_command(&mut self, cmd: Command) -> Result<Event, LedgerError> {
        match cmd {
            Command::CreateAccount {
                account_id,
                currency,
                starting_balance,
            } => {
                let new_user_account = UserAccount {
                    account_id,
                    currency,
                    balance: starting_balance,
                };
                if self.ledger_balance.contains_key(&account_id) {
                    return Err(LedgerError::AccountAlreadyExists);
                }
                self.ledger_balance.insert(account_id, new_user_account);
                Ok(Event::NewAccountAdded { id: account_id })
            }
            Command::Transfer {
                transaction_id,
                from,
                to,
                amount,
            } => {
                if self.ledger_tx.contains_key(&transaction_id) {
                    return Err(LedgerError::DoubleSpend);
                }
                let sender_currency = if let Some(sender) = self.ledger_balance.get(&from) {
                    if sender.balance < amount {
                        return Err(LedgerError::InsufficientBalance);
                    }
                    sender.currency.clone()
                } else {
                    return Err(LedgerError::AccountNotFound);
                };
                let receiver_currency = if let Some(receiver) = self.ledger_balance.get(&to) {
                    receiver.currency.clone()
                } else {
                    return Err(LedgerError::AccountNotFound);
                };

                if sender_currency != receiver_currency {
                    return Err(LedgerError::DifferentCurrency);
                }
                if let Some(sender_account) = self.ledger_balance.get_mut(&from) {
                    sender_account.balance -= amount;
                    let tx = Transaction {
                        id: transaction_id,
                        sender: from,
                        receiver: to,
                        amount,
                        currency: sender_account.currency.clone(),
                    };
                    let timestamp = time::SystemTime::now();
                    self.ledger_tx.insert(transaction_id, (tx, timestamp));
                }

                if let Some(receiver) = self.ledger_balance.get_mut(&to) {
                    receiver.balance += amount;
                    return Ok(Event::MoneyTransfered { from, to, amount });
                } else {
                    return Err(LedgerError::AccountNotFound);
                }
            }
            Command::Shutdown => Ok(Event::Shutdown),
        }
    }
}

pub struct Bank {
    pub sender: mpsc::Sender<LedgerMessage>,
}

impl Bank {
    pub fn new(tx: mpsc::Sender<LedgerMessage>) -> Self {
        Self { sender: tx }
    }

    pub async fn submit(&self, cmd: Command) -> Result<Event, LedgerError> {
        let (tx, rx) = oneshot::channel::<Result<Event, LedgerError>>();
        let ledger_message = LedgerMessage {
            command: cmd,
            responder: tx,
        };
        if let Err(_) = self.sender.send(ledger_message).await {
            return Err(LedgerError::InvalidCommand);
        }
        match tokio::time::timeout(Duration::from_secs(1), rx).await {
            Ok(event) => {
                if let Ok(event) = event {
                    return event;
                } else {
                    Err(LedgerError::Timeout)
                }
            }
            Err(_) => Err(LedgerError::Timeout),
        }
    }
}

pub struct LedgerMessage {
    command: Command,
    responder: tokio::sync::oneshot::Sender<Result<Event, LedgerError>>,
}
async fn run_ledger(mut rx: mpsc::Receiver<LedgerMessage>) {
    let mut ledger = Ledger::new();
    let mut processing_id = 0;
    while let Some(ledger_message) = rx.recv().await {
        processing_id += 1;
        let resp = ledger.process_command(ledger_message.command);
        let is_shutdown = matches!(resp, Ok(Event::Shutdown));

        match ledger_message.responder.send(resp) {
            Ok(_) => {
                println!("Successfully 🎉 processed No: {processing_id}")
            }
            Err(_) => {
                println!(
                    "Failed 💀 while processing {processing_id} {:?}",
                    ledger.ledger_balance
                );
            }
        }

        if is_shutdown {
            println!("Ledger shutting down gracefully.");
            break;
        }
    }
}

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel(100);
    tokio::spawn(run_ledger(rx));

    // Create the client
    let bank = Bank::new(tx);

    // NOW you create users and transfer
    let user1_id = Uuid::new_v4().as_bytes().clone();
    let user2_id = Uuid::new_v4().as_bytes().clone();

    // Look how clean this is compared to your previous code:
    bank.submit(Command::CreateAccount {
        account_id: user1_id,
        currency: Currency::USD,
        starting_balance: 1000,
    })
    .await
    .expect("Failed to create");
    bank.submit(Command::CreateAccount {
        account_id: user2_id,
        currency: Currency::USD,
        starting_balance: 1000,
    })
    .await
    .expect("Failed to create");

    let transaction_id = Uuid::new_v4().as_bytes().clone();

    bank.submit(Command::Transfer {
        transaction_id,
        from: user1_id,
        to: user2_id,
        amount: 111,
    })
    .await
    .expect("Failed to transfer");

    bank.submit(Command::Shutdown).await.expect("msg");
}
