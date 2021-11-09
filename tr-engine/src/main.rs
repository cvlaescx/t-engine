mod input;
mod account;
mod tests;

use std::thread;
use std::time::Duration;

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use env_logger::Env;

use crate::account::Account;
use crate::input::load_data;


static GLOBAL_THREAD_COUNT: AtomicUsize = AtomicUsize::new(0);
fn process_clients(file_name:String) {
    let clients_records = load_data(&file_name);
    println!("client, available, held, total, locked");

    for (client_id, client_records) in clients_records {
        GLOBAL_THREAD_COUNT.fetch_add(1, Ordering::SeqCst);
        let _ = thread::spawn(move || {
            let mut account = Box::new(Account::new(client_id));
            account.dispatch_transactions(&client_records);
            GLOBAL_THREAD_COUNT.fetch_sub(1, Ordering::SeqCst);
        });
    }

    //wait for all threads to finish
    while GLOBAL_THREAD_COUNT.load(Ordering::SeqCst) != 0 {
        log::debug!("GLOBAL_THREAD_COUNT ?? {:?}",&GLOBAL_THREAD_COUNT);
        thread::sleep(Duration::from_millis(1));
    }
}

fn main() {
    let file_name = std::env::args().nth(1).expect("no input file given");

    process_clients(file_name);
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init(); // switch these lines to see/hide logs
}
