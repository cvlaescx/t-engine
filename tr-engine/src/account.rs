use std::collections::HashMap;
use crate::input::DFRecord;
use crate::input::ACTION;


pub(super) struct Account {
    client_id: u16,
    transactions_disputed : HashMap<u32, i64>, //  <tx, amount>
    transactions_solved : HashMap<u32, i64>, //  <tx, amount>

    available : i64,
    held      : i64,
    total     : i64,
    locked    : bool,
}

impl Account {
    pub(super) fn new(client_id: u16) -> Account {
        Account{
            client_id,
            transactions_disputed : HashMap::new(),
            transactions_solved : HashMap::new(),
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

    fn tr_deposit(&mut self, record: &DFRecord){
        let amount = record.amount.unwrap();
        log::debug!("Processing deposit tx={}, amount={}",record.tx, amount);
        self.available = self.available + amount;
        self.total = self.total + amount;
    }

    fn tr_withdrawal(&mut self, record: &DFRecord)  {
        let amount = record.amount.unwrap();
        log::debug!("Processing withdrawal tx={}, amount={}",record.tx, record.amount.unwrap());
        if amount <= self.available {
            self.available = self.available - amount;
            self.total = self.total - amount;
        } else {
            log::warn!("BUSINESS FLAG YELLOW: tx {:?} - Client {:?} tried to withdraw\
                            :  {:?} from available {:?} while held={:?}",
            record.tx, self.client_id, amount, self.available, self.held);
        }
    }

    fn tr_dispute(&mut self, disputed_record: &Box<DFRecord>, client_records:&Vec<Box<DFRecord>> ) {
        let tx = disputed_record.tx;
        let mut disputed_records:Vec<&Box<DFRecord>> = Vec::new();
        for rec in client_records{
            if rec.index> disputed_record.index {
                break;
            }
            if rec.tx != tx {
                continue
            }
            if rec.amount.is_none()  {
                continue
            }
            disputed_records.push(rec);
        }

        if disputed_records.len() != 1 {
            log::error!("fuzzy disputed transactions for client {} and tx {} --->  {} {:?}",
            self.client_id, tx, disputed_records.len(), disputed_records);
            return;
        }

        log::debug!("Processing dispute tx={}",tx);

        if  self.transactions_solved.contains_key(&tx) {
            log::debug!("already solved for client={} tx={:?} ",self.client_id, tx);
            return;
        }
        if  self.transactions_disputed.contains_key(&tx) {
            log::debug!("already requested dispute for client={} tx={:?}",self.client_id, tx);
            return;
        }


        let amount:i64 = disputed_records[0].amount.unwrap();
        self.available = self.available - amount;
        self.held = self.held + amount;
        self.transactions_disputed.insert(tx,amount);
    }

    fn tr_resolve(&mut self, record: &DFRecord) {
        let tx = record.tx;
        log::debug!("Processing resolve tx={}",record.tx);
        let transaction_disputed = self.transactions_disputed.get(&tx);
        if !transaction_disputed.is_none() {
            let amount = *transaction_disputed.unwrap();
            self.available = self.available + amount;
            self.held = self.held - amount;
            self.transactions_disputed.remove(&tx);
            self.transactions_solved.insert(tx,amount);
        }
    }

    fn tr_chargeback(&mut self, record: &DFRecord) {
        let tx = record.tx;
        log::debug!("Processing chargeback tx={}",record.tx);
        let transaction_disputed = self.transactions_disputed.get(&tx);
        if !transaction_disputed.is_none() {
            let amount = *transaction_disputed.unwrap();
            self.total = self.total - amount;
            self.held = self.held - amount;
            self.locked = true;
            self.transactions_disputed.remove(&tx);
            self.transactions_solved.insert(tx,amount);
            log::warn!("BUSINESS FLAG RED: Client {} locked due to chargeback  \
                    tx={}, amount={}",self.client_id, tx, amount);
        };
    }

    pub(super) fn dispatch_transactions(&mut self, client_records: &Vec<Box<DFRecord>>) -> String {
        log::debug!("Processing data for client {}", self.client_id);

        for record in client_records {
            if self.locked && (record.action == ACTION::DEPOSIT|| record.action == ACTION::WITHDRAWAL) {
               log::debug!("account for client {:?} is locked. ignoring action={}, tx={}, amount={:?}"
                   ,self.client_id,record.action, record.tx, record.amount);
            } else {
                match record.action {
                    ACTION::DEPOSIT => self.tr_deposit(&record),
                    ACTION::WITHDRAWAL => self.tr_withdrawal(&record),
                    ACTION::DISPUTE => self.tr_dispute(&record, &client_records),
                    ACTION::RESOLVE => self.tr_resolve(&record),
                    ACTION::CHARGEBACK => self.tr_chargeback(&record),
                    _ => log::debug!("Cannot understand action={:?} tx={:?} amount={:?} for client={:?}",record.action, record.tx, record.amount,self.client_id),
                }
                log::debug!("{}",self.to_string());
            }
        };
        self.println();
        self.to_string()
    }

}
