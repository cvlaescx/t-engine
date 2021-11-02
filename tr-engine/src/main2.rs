use polars::prelude::*;

use core::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::fs::File;
use serde::{Deserialize};
use std::thread;
// use std::panic;
use core::time::Duration;
use std::collections::HashMap;

#[derive(Deserialize)]
struct ClientRecord {
    action: String,
    tx: u32,
    amount: Option<i64>,
}

pub(crate)  fn read_df() -> Result<DataFrame> {
    let schema =     Schema::new(vec![
        Field::new("action", DataType::Utf8),
        Field::new("client", DataType::UInt32),
        Field::new("tx", DataType::UInt32),
        Field::new("amount", DataType::UInt64),
    ]);
    CsvReader::from_path("tr1.csv_bak")?
        .with_dtypes(Some(&schema))
            .has_header(true)
            .finish()
}


pub(crate)   fn split_df(df: &DataFrame) -> Result<()> {
    let clients = df.column("client")?.u32()?;

    for (group_first, group_indexes) in df.groupby("client")?.get_groups() {
        let client_id = clients.get(*group_first as usize);
        let idx = UInt32Chunked::new_from_slice("idx", group_indexes);
        let sub_df = df.take(&idx)?.select(&["action", "tx", "amount"]);

        GLOBAL_THREAD_COUNT.fetch_add(1, Ordering::SeqCst);
        // log::debug!("GLOBAL_THREAD_COUNT ++ {:?}",&GLOBAL_THREAD_COUNT);
        thread::spawn(move|| {
            let result = thread::spawn(move || {
                // log::debug!("client {:?} thread reporting", &client_id);
                if let Err(err) = process_client(client_id, &sub_df.unwrap()) {
                    println!("Cannot process client {:?}", &client_id);
                    println!("{:?}", err);
                }
                GLOBAL_THREAD_COUNT.fetch_sub(1, Ordering::SeqCst);
            });
            // We need to catch panics to reliably signal exit of a thread
            // process errors
            match result {
                // Err(err) => println!("{:?}", err),
                _ => {}
            }
            // signal thread exit


        });

    };

    // Wait for all threads to finish.
    while GLOBAL_THREAD_COUNT.load(Ordering::SeqCst) != 0 {
        log::debug!("GLOBAL_THREAD_COUNT ?? {:?}",&GLOBAL_THREAD_COUNT);
        thread::sleep(Duration::from_millis(1));
    }

    Ok(())
}

static GLOBAL_THREAD_COUNT: AtomicUsize = AtomicUsize::new(0);

fn process_client(client: Option<u32>, df: &DataFrame) -> Result<()> {
    log::debug!("Processing data for client {}", client.unwrap_or(0));
    let filename = format!("myfile_{:?}.txt", client);
    let mut file = File::create(&filename).expect("could not create file");

    CsvWriter::new(&mut file)
        .has_header(true)
        .with_delimiter(b',')
        .finish(df)
        .expect("cannot open file for writing");

    let file2 = File::open(&filename)?;
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(file2);

    let mut account = Account::new(client.unwrap(), df);

    for result in rdr.deserialize() {
        let record: ClientRecord =  result.unwrap_or(ClientRecord{action:"err".to_string(),tx:0,amount:Some(0)});

        if account.locked && (record.action == "deposit" || record.action == "withdrawal") {
           log::debug!("account for client {:?} is locked. ignoring action={}, tx={}, amount={:?}"
               ,client,record.action, record.tx, record.amount);
        } else {
            match &*record.action {
                "deposit" => account.tr_deposit(record),
                "withdrawal" => account.tr_withdrawal(record),
                "dispute" => account.tr_dispute(record),
                "resolve" => account.tr_resolve(record),
                "chargeback" => account.tr_chargeback(record),
                _ => log::debug!("Cannot understand action={:?} tx={:?} amount={:?} for client={:?}",record.action, record.tx, record.amount,client),
            }
            log::debug!("{}",account.to_string());
        }
    };
    account.println();
    Ok(())
}

struct Account<'a> {
    client_id: u32,
    df: &'a DataFrame,
    transactions_disputed : HashMap<u32, i64>, //  <tx, amount>
    transactions_solved : HashMap<u32, i64>, //  <tx, amount>

    available : i64,
    held      : i64,
    total     : i64,
    locked    : bool,
}

impl Account <'_> {
    fn new(client_id: u32, df : &DataFrame) -> Account {
        Account{
            df,
            transactions_disputed : HashMap::new(),
            transactions_solved : HashMap::new(),
            client_id,
            available: 0,
            held: 0,
            total: 0,
            locked: false
        }
    }

    fn to_string(&self) -> String {
        fn decimal (val: i64) -> String {
            if val<0 {
                format!( "-{big}.{small:0>width$}", big=-val/10000, small=-val%10000, width=4)
            } else {
                format!( "{big}.{small:0>width$}", big=val/10000, small=val%10000, width=4)
            }
        }
        format!( "{}, {}, {}, {}, {}",
                self.client_id,
                 decimal(self.available),
                 decimal(self.held),
                 decimal(self. total),
                 self.locked)
    }

    fn println(&self) {
        println!("{}", self.to_string());
    }

    fn tr_deposit(&mut self, record: ClientRecord){
        let amount = record.amount.unwrap();
        log::debug!("Processing deposit tx={}, amount={}",record.tx, amount);
        self.available = self.available + amount;
        self.total = self.total + amount;
    }

    fn tr_withdrawal(&mut self, record: ClientRecord)  {
        let amount = record.amount.unwrap();
        log::debug!("Processing withdrawal tx={}, amount={}",record.tx, record.amount.unwrap());
        if amount <= self.available {
            self.available = self.available - amount;
            self.total = self.total - amount;
        } else {
            log::warn!("BUSINESS FLAG YELLOW: tx {:?} - Client {:?} tried to withdraw'
                            ' {:?} from available {:?} while held={:?}",
            record.tx, self.client_id, amount, self.available, self.held);
        }
    }

    fn tr_dispute(&mut self, record: ClientRecord) {
        let tx = record.tx;
        log::debug!("Processing dispute tx={}",tx);

        let x1= match self.transactions_solved.get(&tx) {
            Some(amount) => {
                log::debug!("already solved for client={} tx={:?} amount={}",self.client_id, tx, amount);
                Some(amount)
            }
            None => None
        };
        let x2= match self.transactions_disputed.get(&tx) {
            Some(amount) => {
                log::debug!("already requested dispute for client={} tx={:?} amount={}",self.client_id, tx, amount);
                Some(amount)
            }
            None => None
        };
        if x1.is_none() && x2.is_none() {
            log::debug!("will need to handle dispute for client={} tx={}", self.client_id, tx);
            let mask1 = self.df.column("tx").unwrap().eq(tx);
            let filtered_df1 = self.df.filter(&mask1).unwrap();
            let mask2 = filtered_df1.column("amount").unwrap().is_not_null();
            let filtered_df2 = filtered_df1.filter(&mask2).unwrap();

            if filtered_df2.width() != 1 {
                log::error!("fuzzy disputed transactions for client {} and tx {} --->  {} {:?}",
                self.client_id, tx, filtered_df2.width(), filtered_df2);
            } else {
                let amount:i64 = filtered_df2.select_at_idx(2).unwrap().min().unwrap_or(0);
                self.available = self.available - amount;
                self.held = self.held + amount;
                self.transactions_disputed.insert(tx,amount);
            }
        }

    }

    fn get_transaction_disputed(&self, tx:u32) -> Option<i64> {
        let mm = match self.transactions_disputed.get(&tx) {
            None => None,
            Some(amount) => {
                log::debug!("found requested dispute for client={} tx={:?} amount={}",self.client_id, tx, amount);
                Some(*amount)
            }
        };
        mm
    }
    fn tr_resolve(&mut self, record: ClientRecord) {
        let tx = record.tx;
        log::debug!("Processing resolve tx={}",record.tx);
        let result = self.get_transaction_disputed(tx);
        if !result.is_none() {
            let amount = result.unwrap();
            self.available = self.available + amount;
            self.held = self.held - amount;
            self.transactions_disputed.remove(&tx);
            self.transactions_solved.insert(tx,amount);
        }
    }

    fn tr_chargeback(&mut self, record: ClientRecord) {
        let tx = record.tx;
        log::debug!("Processing chargeback tx={}",record.tx);
        let result = self.get_transaction_disputed(tx);
        if !result.is_none() {
            let amount = result.unwrap();
            self.total = self.total - amount;
            self.held = self.held - amount;
            self.locked = true;
            self.transactions_disputed.remove(&tx);
            self.transactions_solved.insert(tx,amount);
            log::warn!("BUSINESS FLAG RED: Client {} locked due to chargeback  \
                    tx={}, amount={}",self.client_id, tx, amount);
        };
    }
}

