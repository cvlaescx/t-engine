use serde::{Deserialize, Deserializer};
use csv::Trim::All;
use std::process;
use std::fs::File;
use std::collections::HashMap;
use std::error::Error;

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
pub(crate) struct DFRecord {
    #[serde(rename = "type", deserialize_with = "parse_type")]
    pub(super) action: u8,
    client: u16,
    pub(super) tx: u32,
    #[serde(deserialize_with = "parse_amount")]
    pub(super) amount: Option<i64>,
    pub(super) index: Option<u32>,
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

fn parse_amount<'de, D>(d: D) -> Result<Option<i64>, D::Error> where D: Deserializer<'de> {
    Deserialize::deserialize(d)
        .map(|amount: Option<String>| {
            if amount.is_none() {
                None
            } else {
                let s = amount?;
                let zeros_to_add = "0".repeat(
                    4  - ((s.len() - 1) - s.find('.').unwrap_or(s.len() - 1))
                );

                let big_number = format!("{}{}", s.replace(".", ""), zeros_to_add).parse::<i64>();

                if big_number.is_err() {
                    log::debug!("ERROR parsing {} (default will be zero) --- {:?}", s, big_number);
                    Some(0)
                } else {
                    // assert!(*(big_number.as_ref().unwrap())>=0); // just to validate input correctness
                    big_number.ok()
                }
            }
        })
}

pub(super) fn load_data(file_name: &str) -> Result<HashMap<u16,Vec<Box<DFRecord>>>, Box<dyn Error>> {
    let mut client_ids_map:HashMap<u16,Vec<Box<DFRecord>>> = HashMap::new();

    let file = File::open(file_name)?;
    let mut rdr = csv::ReaderBuilder::new()
        // .flexible(true)
        .comment(Some(b'#'))
        .trim(All)
        .from_reader(file);

    let mut reader_headers = rdr.headers().unwrap().clone();
    reader_headers.push_field("index");
    rdr.set_headers(reader_headers);

    let mut index:u32 = 0;
    for result in rdr.deserialize() {
        if  result.is_err() {
            println!("{:?}", result);
            process::exit(1);
        }
        let mut record: Box<DFRecord> = Box::new(result?);

        index = index + 1;
        record.index = Some(index);

        let client_transactions =client_ids_map.entry(record.client).or_insert(Vec::new());
        client_transactions.push(record);
    }

    Ok(client_ids_map)
}
