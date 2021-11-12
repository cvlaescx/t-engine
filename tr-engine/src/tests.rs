#[cfg(test)]
pub fn client_transactions(client_id:u16) -> String {
    use crate::account::Account;
    use crate::input::load_data;
    let clients_records = load_data("../testing/transactions.csv".to_string()).unwrap();

    let mut account = Account::new(client_id);
    account.dispatch_transactions(&clients_records.get(&client_id).unwrap())
    }

#[cfg(test)]
mod tests {
    use crate::tests::client_transactions;

    macro_rules! client_test {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (input, expected) = $value;
                assert_eq!(expected, client_transactions(input));
            }
        )*
        }
    }

    client_test! {
        client_transactions_1: (1, "1, 0.0000, 0.0000, 0.0000, false"),
        client_transactions_2: (2, "2, -1.0000, 0.0000, -1.0000, true"),
        client_transactions_3: (3, "3, 0.5000, 0.0000, 0.5000, false"),
        client_transactions_4: (4, "4, 4.0000, 0.0000, 4.0000, false"),
        client_transactions_5: (5, "5, 0.0000, 0.0000, 0.0000, false"),
        client_transactions_6: (6, "6, 0.0000, 0.0000, 0.0000, false"),
        client_transactions_7: (7, "7, 0.0000, 0.0000, 0.0000, false"),
        client_transactions_20: (20, "20, 0.0001, 0.0000, 0.0001, false"),
        client_transactions_21: (21, "21, 0.9999, 0.0000, 0.9999, false"),
        client_transactions_22: (22, "22, 0.0000, 0.0000, 0.0000, true"),
        client_transactions_23: (23, "23, 0.0000, 0.0000, 0.0000, true"),
        client_transactions_24: (24, "24, 0.9999, 0.0000, 0.9999, false"),
        client_transactions_30: (30, "30, -3.0000, 3.0000, 0.0000, false"),
        client_transactions_31: (31, "31, 0.0000, 0.0000, 0.0000, false"),
        client_transactions_32: (32, "32, 0.0000, 0.0000, 0.0000, false"),
        client_transactions_33: (33, "33, -3.0000, 0.0000, -3.0000, true"),
        client_transactions_34: (34, "34, -2.0000, 0.0000, -2.0000, true"),
        client_transactions_35: (35, "35, -1.0000, 0.0000, -1.0000, true"),
        client_transactions_36: (36, "36, -1.0000, 0.0000, -1.0000, true"),
        client_transactions_37: (37, "37, -2.0000, 0.0000, -2.0000, true"),
        client_transactions_50: (50, "50, 1234567890123457.9999, 0.0000, 1234567890123457.9999, false"),
        client_transactions_51: (51, "51, 2.0000, 1.0000, 3.0000, false"),
        client_transactions_52: (52, "52, 3.0000, 0.0000, 3.0000, false"),
        client_transactions_53: (53, "53, 2.0000, 0.0000, 2.0000, true"),
        client_transactions_60: (60, "60, 2.4176, 0.0000, 2.4176, false"),
        client_transactions_62: (62, "62, 0.9999, 1.0000, 1.9999, false"),
        client_transactions_63: (63, "63, 0.0000, 1.0000, 1.0000, false"),
        client_transactions_64: (64, "64, 0.9999, 0.0000, 0.9999, false"),
        client_transactions_65: (65, "65, 0.0000, 0.0000, 0.0000, true"),
        client_transactions_66: (66, "66, -0.3000, 1.0000, 0.7000, false"),
        client_transactions_67: (67, "67, 1234567890123455.9999, 0.0000, 1234567890123456.9999, false"),
        client_transactions_68: (68, "68, 0.0001, 0.0000, 0.0001, false"),
        client_transactions_69: (69, "69, 1.0000, 0.0000, 1.0000, false"),
        client_transactions_70: (70, "70, 0.0000, 1.0000, 1.0000, false"),
        client_transactions_71: (71, "71, 0.0000, 3.0000, 3.0000, false"),
        client_transactions_72: (72, "72, 0.0000, 3.0000, 3.0000, false"),
        client_transactions_73: (73, "73, -1234567890123456.0001, 1234567890123456.0001, 0.0000, false"),
        client_transactions_74: (74, "74, -123456789012345.0001, 123456789012345.0001, 0.0000, false"),
        client_transactions_80: (80, "80, 0.7000, 0.5000, 1.2000, false"),
        client_transactions_81: (81, "81, -1.1000, 0.0000, -1.1000, true"),
        client_transactions_82: (82, "82, -1.1000, 0.0000, -1.1000, true"),
        client_transactions_90: (90, "90, 1.8000, 0.0000, 1.8000, false"),
        client_transactions_91: (91, "91, 0.5000, 0.0000, 0.5000, false"),
        client_transactions_93: (93, "93, 1.8000, 0.0000, 1.8000, false"),
        client_transactions_94: (94, "94, -0.8000, 0.9000, 0.1000, false"),
        client_transactions_95: (95, "95, -0.2000, 0.0000, -0.2000, true"),
        client_transactions_101: (101, "101, 0.0000, 0.0000, 0.0000, false"),
        client_transactions_102: (102, "102, -123.0000, 0.0000, -123.0000, true"),
        client_transactions_109: (109, "109, 0.0200, 0.0000, 0.0200, true"),
        client_transactions_110: (110, "110, -0.0001, 0.0000, -0.0001, true"),
        client_transactions_111: (111, "111, 124.0200, 0.0000, 124.0200, false"),
        client_transactions_112: (112, "112, 123.9999, 0.0000, 123.9999, false"),
    }
}