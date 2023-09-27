use std::{error::Error, path::Path};

use chrono::{NaiveDateTime, TimeZone};
use chrono_tz::Europe::Berlin;
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::base::{Amount, Transaction, Operation};

#[derive(Debug, Clone, Deserialize)]
enum TransferType {
    #[serde(rename = "Sent to")]
    SentTo,
    #[serde(rename = "Received with")]
    ReceivedWith,
    Generated,
}

#[derive(Debug, Deserialize)]
struct BitcoinCoreAction<'a> {
    // #[serde(rename = "Confirmed")]
    // confirmed: bool,
    #[serde(rename = "Date")]
    date: NaiveDateTime,
    #[serde(rename = "Type")]
    type_: TransferType,
    #[serde(rename = "Label")]
    label: &'a str,
    // #[serde(rename = "Address")]
    // address: String,
    #[serde(rename = "Amount (BTC)", alias = "Amount (PPC)", alias = "Amount (RDD)")]
    amount: Decimal,
    #[serde(rename = "ID")]
    id: &'a str,
}

impl<'a> BitcoinCoreAction<'a> {
    // todo: translate address?
    fn to_tx(self, currency: &str) -> Transaction {
        let utc_time = Berlin.from_local_datetime(&self.date).unwrap().naive_utc();
        let mut tx = match self.type_ {
            TransferType::SentTo => {
                Transaction::send(utc_time, Amount::new(-self.amount, currency.to_owned()))
            },
            TransferType::ReceivedWith => {
                Transaction::receive(utc_time, Amount::new(self.amount, currency.to_owned()))
            },
            TransferType::Generated => {
                Transaction::new(utc_time, Operation::Staking(Amount::new(self.amount, currency.to_owned())))
            },
        };
        tx.description = if self.label.is_empty() { None } else { Some(self.label.to_owned()) };
        tx.tx_hash = Some(self.id.trim_end_matches("-000").to_owned());
        tx.blockchain = Some(currency.to_owned());
        tx
    }
}

// loads a Bitcoin Core CSV file into a list of unified transactions
fn load_transactions(input_path: &Path, currency: &str) -> Result<Vec<Transaction>, Box<dyn Error>> {
    let mut transactions = Vec::new();

    let mut rdr = csv::ReaderBuilder::new()
        .from_path(input_path)?;
    let mut raw_record = csv::StringRecord::new();
    let headers = rdr.headers()?.clone();

    while rdr.read_record(&mut raw_record)? {
        let record: BitcoinCoreAction = raw_record.deserialize(Some(&headers))?;
        transactions.push(record.to_tx(currency));
    }

    Ok(transactions)
}

// loads a Bitcoin Core CSV file into a list of unified transactions
pub(crate) fn load_bitcoin_core_csv(input_path: &Path) -> Result<Vec<Transaction>, Box<dyn Error>> {
    load_transactions(input_path, "BTC")
}

// loads a Peercoin CSV file into a list of unified transactions
pub(crate) fn load_peercoin_csv(input_path: &Path) -> Result<Vec<Transaction>, Box<dyn Error>> {
    load_transactions(input_path, "PPC")
}

// loads a Reddcoin Core CSV file into a list of unified transactions
pub(crate) fn load_reddcoin_core_csv(input_path: &Path) -> Result<Vec<Transaction>, Box<dyn Error>> {
    load_transactions(input_path, "RDD")
}
