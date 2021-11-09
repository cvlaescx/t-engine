use std::collections::HashMap;
use crate::input::ClientRecord;
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

    fn tr_deposit(&mut self, record: &ClientRecord){
        let amount = record.amount;
        log::debug!("Processing deposit tx={}, amount={}",record.tx, amount);
        self.available = self.available + amount;
        self.total = self.total + amount;
    }

    fn tr_withdrawal(&mut self, record: &ClientRecord)  {
        let amount = record.amount;
        log::debug!("Processing withdrawal tx={}, amount={}",record.tx, record.amount);
        if amount <= self.available {
            self.available = self.available - amount;
            self.total = self.total - amount;
        } else {
            log::warn!("BUSINESS FLAG YELLOW: tx {:?} - Client {:?} tried to withdraw\
                            :  {:?} from available {:?} while held={:?}",
            record.tx, self.client_id, amount, self.available, self.held);
        }
    }

    fn tr_dispute(&mut self,
                  disputed_record: &ClientRecord,
                  client_records:&Vec<ClientRecord>,
                  index: usize) {
        let tx = disputed_record.tx;
        log::debug!("Processing dispute tx={}",tx);

        let mut disputed_records:Vec<i64> = Vec::new();
        for (local_index, local_record) in client_records.iter().enumerate(){
            if local_index > index {
                break;
            }
            if local_record.tx != tx {
                continue
            }
            if local_record.amount < 0  {
                continue
            }
            disputed_records.push(local_record.amount);
        }

        if disputed_records.len() != 1 {
            log::error!("fuzzy disputed transactions for client {} and tx {} --->  {} {:?}",
            self.client_id, tx, disputed_records.len(), disputed_records);
            return;
        }
        if  self.transactions_solved.contains_key(&tx) {
            log::debug!("already solved dispute for client={} tx={:?} ",self.client_id, tx);
            return;
        }
        if  self.transactions_disputed.contains_key(&tx) {
            log::debug!("already requested dispute for client={} tx={:?}",self.client_id, tx);
            return;
        }

        let amount:i64 = disputed_records[0];
        self.available = self.available - amount;
        self.held = self.held + amount;
        self.transactions_disputed.insert(tx,amount);
    }

    fn tr_resolve(&mut self, record: &ClientRecord) {
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

    fn tr_chargeback(&mut self, record: &ClientRecord) {
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

    pub(super) fn dispatch_transactions(&mut self, client_records: &Vec<ClientRecord>) -> String {
        log::debug!("Processing data for client {}", self.client_id);

        for (index,record) in client_records.iter().enumerate() {
            if self.locked && (record.action == ACTION::DEPOSIT|| record.action == ACTION::WITHDRAWAL) {
               log::debug!("account for client {:?} is locked. ignoring action={}, tx={}, amount={:?}"
                   ,self.client_id,record.action, record.tx, record.amount);
            } else {
                match record.action {
                    ACTION::DEPOSIT => self.tr_deposit(record),
                    ACTION::WITHDRAWAL => self.tr_withdrawal(record),
                    ACTION::DISPUTE => self.tr_dispute(record, client_records, index),
                    ACTION::RESOLVE => self.tr_resolve(record),
                    ACTION::CHARGEBACK => self.tr_chargeback(record),
                    _ => log::debug!("Cannot understand action={:?} tx={:?} amount={:?} for client={:?}"
                        ,record.action, record.tx, record.amount,self.client_id),
                }
                log::debug!("{}",self.to_string());
            }
        };
        let account_status=self.to_string();
        println!("{}",account_status);
        account_status
    }

}
