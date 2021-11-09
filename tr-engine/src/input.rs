use serde::{Deserialize, Deserializer};
use csv::Trim::All;
use std::fs::File;
use std::collections::HashMap;

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
    action: u8,
    client: u16,
    tx: u32,
    #[serde(deserialize_with = "parse_amount")]
    amount: i64,
}

#[derive(Debug)]
pub(crate) struct ClientRecord {
    pub(super) action: u8,
    pub(super) tx: u32,
    pub(super) amount: i64,
}

impl ClientRecord {
    pub(super) fn new(input_record:&InputRecord) -> ClientRecord {
        ClientRecord {
            action: input_record.action,
            tx: input_record.tx,
            amount: input_record.amount,
        }
    }
}
fn parse_type<'de, D>(d: D) -> Result<u8, D::Error> where D: Deserializer<'de> {
    Deserialize::deserialize(d)
        .map(|x: String| {
            match x.as_str() {
                "deposit" => ACTION::DEPOSIT,
                "withdrawal" => ACTION::WITHDRAWAL,
                "dispute" => ACTION::DISPUTE,
                "resolve" => ACTION::RESOLVE,
                "chargeback" => ACTION::CHARGEBACK,
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

pub(super) fn load_data(file_name: &str) -> HashMap<u16,Vec<ClientRecord>> {
    let mut client_ids_map:HashMap<u16,Vec<ClientRecord>> = HashMap::new();

    let file = File::open(file_name).unwrap();
    let mut rdr = csv::ReaderBuilder::new()
        .comment(Some(b'#'))
        .trim(All)
        .from_reader(file);

    let mut index:u32 = 0;
    for result in rdr.deserialize() {
        if  result.is_err() {
            panic!("{:?}", result);
        }
        let input_record: InputRecord = result.unwrap();

        index += 1; // this is only for debug purpose
        if index%100000 == 0 {
            log::debug!("Read {} records",index);
        }

        let client_transactions =client_ids_map.entry(input_record.client).or_insert(Vec::new());
        client_transactions.push(ClientRecord::new(&input_record));
    }
    client_ids_map
}
