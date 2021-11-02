use std::error::Error;
use std::fs::File;
use std::process;

use csv::Trim::All;
use csv::WriterBuilder;

use serde::{Serialize, Deserialize, Deserializer};


mod main2;
use crate::main2::read_df;
use crate::main2::split_df;

// use crate::Env;
use env_logger::Env;

#[derive(Deserialize)]
struct InputRecord {
    #[serde(rename = "type")]
    action: String,
    client: u16,
    tx: u32,
    #[serde(deserialize_with="parse_amount")]
    amount: Option<u64>,
}

#[derive(Serialize)]
 struct DFRecord {
    action: String,
    client: u16,
    tx: u32,
    amount: Option<u64>,
 }

fn parse_amount<'de, D>(d: D) -> Result<Option<u64>, D::Error> where D: Deserializer<'de> {
    Deserialize::deserialize(d)
        .map(|x: Option<String>| {
            if x.is_none() {
                None
            } else {
                let s = x?;
                let zeros_nr = 4  + s.find('.').unwrap_or(s.len() - 1) - (s.len() - 1);
                let together = format!("{}{}", s.replace(".", ""), "0".repeat(zeros_nr));

                let aaa = together.parse::<u64>();

                if aaa.is_err() {
                    log::debug!("ERROR parsing {} (default will be zero) --- {:?}", together, aaa);
                    Some(0)
                } else {
                    Some(aaa.ok()?)
                }

            }
        })
}

fn prepare_df_data() -> Result<(), Box<dyn Error>> {
    let file_path = std::env::args().nth(1).expect("no input file given");
    let temp_file = format!("{}{}", file_path, "_bak");
    let file = File::open(file_path)?;
    let mut rdr = csv::ReaderBuilder::new()
        // .flexible(true)
        .comment(Some(b'#'))
        .trim(All)
        .from_reader(file);

    let mut wtr = WriterBuilder::new().from_path(temp_file)?;

    for result in rdr.deserialize() {
        let record: InputRecord = result?;
        wtr.serialize(DFRecord {
         action: record.action,
         client: record.client,
         tx: record.tx,
         amount: record.amount,
     })?;
    }
    wtr.flush()?;

    Ok(())
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();

    if let Err(err) = prepare_df_data() {
        println!("{}", err);
        process::exit(1);
    }
    let df = read_df().unwrap();
    let result = split_df(&df);
    if result.is_err() {
        println!("ERROR is {:?} ", result);
    }

}