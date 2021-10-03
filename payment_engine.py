import sys
import traceback
import pandas as pd
from multiprocessing import Pool
import logging
logging.getLogger().setLevel(logging.CRITICAL)

EPSILON = 1e-10

DEPOSIT = 'deposit'
WITHDRAWAL = 'withdrawal'
DISPUTE = 'dispute'
RESOLVE = 'resolve'
CHARGEBACK = 'chargeback'

# input format is (type, client, tx, amount), where
#  type in {"deposit", "withdrawal", "dispute", "resolve", "chargeback"}
#  client  u16
#  tx      u32
#  amount  decimal, with 4 digits precision

# output format is (client, available, held, total, locked)


class Account(object):
    def __init__(self, client_data):
        self.transactions = client_data
        _, t_client_id, _, _ = client_data[0]
        self.client_id = int(t_client_id)
        self.transactions_disputed = []  # [transaction]
        self.transactions_solved = {}  # {tx:transaction}
        self.last_row_index = 0

        self.available = 0.0
        self.held      = 0.0
        self.total     = 0.0
        self.locked    = False

    def printable_balance(self):
        account_balance = "{client_id}, {available:.4f}, {held:.4f}, {total:.4f}, {locked}".format(
            client_id=self.client_id,
            available=self.available,
            held=self.held,
            total=self.total,
            locked=str(self.locked).lower()
        )
        return account_balance

    def calculate_balance(self):
        try:
            for row_index, transaction_data in enumerate(self.transactions, 1):
                self.last_row_index = row_index
                if self.locked:
                    t_type, _, _, _ = transaction_data
                    if t_type in [DEPOSIT, WITHDRAWAL]:
                        logging.info('account for client {client_id} is locked. ignoring {transaction}'.format(
                            client_id=self.client_id,
                            transaction=transaction_data))
                        continue
                self.dispatch_transaction(transaction_data)

        except Exception as e:
            logging.error("{} produced unexpected error: {}".format(transaction_data, e))
            traceback.print_exc()
            raise

        return self.printable_balance()

    def dispatch_transaction(self, transaction_data):
        t_type, _, t_tx, t_amount = transaction_data

        method_name = 'transaction_' + t_type
        method = getattr(self, method_name, lambda a, b: logging.info(
            'ignoring invalid transaction {}'.format(transaction_data)))

        method(t_tx, t_amount)
        logging.debug('processed {} --> <{}> '.format(transaction_data, self.printable_balance()))

    def transaction_deposit(self, _, amount):
        self.available += amount
        self.total     += amount

    def transaction_withdrawal(self, tx, amount):
        if -EPSILON <= self.available - amount:
            self.available -= amount
            self.total     -= amount
        else:
            logging.warning('BUSINESS FLAG YELLOW: tx {tx} - Client {client_id} tried to withdraw'
                            ' {amount:.4f} from available {available:.4f} while held={held:.4f}'.format(
                                tx=tx,
                                client_id=self.client_id,
                                amount=amount,
                                available=self.available,
                                held=self.held))

    def transaction_dispute(self, tx, _):
        if tx in self.transactions_solved.keys():
            logging.warning('ignoring multiple dispute request for transaction {}'.format(
                self.transactions_solved[tx]))
            return

        transactions_disputed = [t for index, t in enumerate(self.transactions, 1)
                                 if (index < self.last_row_index) and (t[2] == tx) and (t[3] is not None)]
        if len(transactions_disputed) != 1:
            logging.error('fuzzy disputed transactions for client {client_id} and tx {tx} '
                          '--->  {size} {transactions} '.format(
                            client_id=self.client_id,
                            tx=tx,
                            size=len(transactions_disputed),
                            transactions=transactions_disputed))
            return

        transaction_disputed = transactions_disputed[0]
        if transaction_disputed in self.transactions_disputed:
            logging.warning('Ignoring dispute request while dispute active for {}'.format(transaction_disputed))
            return

        t_type, _, _, t_amount = transaction_disputed
        if t_type != DEPOSIT:
            logging.warning('TODO: clarify rules for non-{deposit} dispute; '
                            'ignore dispute on {transaction} for the moment'.format(
                                deposit=DEPOSIT,
                                transaction=str(transactions_disputed[0])
                            ))
            return

        self.available -= t_amount
        self.held      += t_amount
        self.transactions_disputed += transactions_disputed

    def get_transaction(self, tx, requested_by='operation'):
        transactions_disputed = [x for x in self.transactions_disputed if x[2] == tx]
        if len(transactions_disputed) != 1:
            logging.info('ignoring {operation} request for client {client_id} and tx {tx} '
                         'due to non-active dispute'.format(
                            operation=requested_by,
                            client_id=self.client_id,
                            tx=tx))
            return None
        else:
            return transactions_disputed[0]

    def transaction_resolve(self, tx, _):
        transaction_disputed = self.get_transaction(tx, requested_by=RESOLVE)
        if transaction_disputed is not None:
            t_type, _, _, t_amount = transaction_disputed
            if t_type == DEPOSIT:
                self.available += t_amount
                self.held      -= t_amount
                self.transactions_disputed.remove(transaction_disputed)
                self.transactions_solved[tx] = transaction_disputed
            else:
                logging.error('Undefined {} operation for {}'.format(RESOLVE, transaction_disputed))

    def transaction_chargeback(self, tx, _):
        transaction_disputed = self.get_transaction(tx, requested_by=CHARGEBACK)
        if transaction_disputed is not None:
            t_type, _, _, t_amount = transaction_disputed
            if t_type == DEPOSIT:
                logging.warning('BUSINESS FLAG RED: Client {client_id} locked due to '
                                '{chargeback} {transaction}'.format(
                                    client_id=self.client_id,
                                    chargeback=CHARGEBACK,
                                    transaction=transaction_disputed))
                self.total -= t_amount
                self.held  -= t_amount
                self.locked = True

                self.transactions_disputed.remove(transaction_disputed)
                self.transactions_solved[tx] = transaction_disputed
            else:
                logging.error('Undefined {chargeback} operation for {transaction}'.format(
                    chargeback=CHARGEBACK,
                    transaction=transaction_disputed))


def print_balance(formatted_balance):
    print(formatted_balance)


def calculate_client_balance(client_data):
    account = Account(client_data)
    return account.calculate_balance()


if __name__ == '__main__':

    input_file = sys.argv[1]

    data = pd.read_csv(input_file, header=0, comment="#")
    # this alternative is much slower but handles whitespaces from input file
    # data = pd.read_csv(input_file, header=0, comment="#", sep=r'\s*,\s*', encoding='ascii', engine='python')

    print('client, available, held, total, locked', flush=True)

    pool = Pool()
    client_ids = data['client'].unique().tolist()

    # start a worker for each client_id
    for _, client_id in enumerate(client_ids):
        client_transactions = data[data['client'] == client_id].replace({pd.NaT: None})
        client_transactions_it = client_transactions.itertuples(index=False, name=None)
        pool.apply_async(calculate_client_balance,
                         args=(list(client_transactions_it),),
                         callback=print_balance)

    pool.close()
    pool.join()

