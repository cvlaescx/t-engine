use serde::{Deserialize, Deserializer};
use csv::Trim::All;
use std::fs::File;
use std::collections::HashMap;

use std::error::Error;
// use std::io;
// use std::process;

#[non_exhaustive]
pub(crate) struct ACTION;

impl ACTION {
    pub const UNKNOWN: u8 = 0;
    pub const DEPOSIT: u8 = 1;
    pub const WITHDRAWAL: u8 = 2;
    pub const DISPUTE: u8 = 3;
    pub const RESOLVE: u8 = 4;
    pub const CHARGEBACK: u8 = 5;
}

#[derive(Deserialize, Debug)]
pub(crate) struct InputRecord {
    #[serde(rename = "type", deserialize_with = "parse_type")]
    // #[serde(rename = "type")]
    pub(super) action: u8,
    pub(super) client: u16,
    pub(super) tx: u32,
    #[serde(deserialize_with = "parse_amount")]
    pub(super) amount: i64,
}

#[derive(Debug)]
pub(crate) struct ClientRecord {
    pub(super) action: u8,
    pub(super) tx: u32,
    pub(super) amount: i64,
}

fn parse_type<'de, D>(d: D) -> Result<u8, D::Error> where D: Deserializer<'de> {
    Deserialize::deserialize(d)
        .map(|x: &[u8]| {
            match x {
                b"deposit" => ACTION::DEPOSIT,
                b"withdrawal" => ACTION::WITHDRAWAL,
                b"dispute" => ACTION::DISPUTE,
                b"resolve" => ACTION::RESOLVE,
                b"chargeback" => ACTION::CHARGEBACK,
                _ => ACTION::UNKNOWN,
            }
        })
}

fn parse_amount<'de, D>(d: D) -> Result<i64, D::Error> where D: Deserializer<'de> {
    Deserialize::deserialize(d)
        .map(|amount: Option<&str>| {
            if amount.is_none() {
                -1 as i64
            } else {
                let s = amount.unwrap().to_owned()+"0000";
                let pos = s.find('.').unwrap_or(s.len() - 5);
                let result = s[..pos+5].replace(".", "").parse::<i64>().unwrap_or(0);
                assert!(result>=0);
                result
            }
        })
}

pub(super) fn load_data(file_name: String) -> Result<HashMap<u16,Vec<ClientRecord>>,Box<dyn Error>> {
    let mut client_ids_map:HashMap<u16,Vec<ClientRecord>> = HashMap::new();

    let file = File::open(file_name).unwrap();
    let mut rdr = csv::ReaderBuilder::new()
        .comment(Some(b'#'))
        .trim(All)
        .from_reader(file);
    let mut raw_record = csv::ByteRecord::new();
    let headers = rdr.byte_headers().unwrap().clone();

    let mut index:u32 = 0;

    while rdr.read_byte_record(&mut raw_record)? {
        let record: InputRecord = raw_record.deserialize(Some(&headers)).unwrap();

        index += 1; // this is only for debug purpose
        if index%100000 == 0 {
            println!("Read {} records",index);
        }
        let client_transactions =client_ids_map.entry(record.client).or_insert(Vec::new());
        client_transactions.push(ClientRecord{action:record.action, tx:record.tx, amount:record.amount});
    }

    Ok(client_ids_map)
}
