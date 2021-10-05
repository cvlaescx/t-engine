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
    def __init__(self, client_data_df=None):
        client_id = client_data_df.iloc[0].client
        client_data_df['row_index'] = client_data_df.index

        self.transactions_df = client_data_df
        self.client_id = client_id
        self.transactions_disputed_dict = {}  # {tx:transaction}
        self.transactions_solved_dict = {}  # {tx:transaction}
        self.last_index = 0

        self.available = 0.0
        self.held      = 0.0
        self.total     = 0.0
        self.locked    = False

    def printable_balance(self) -> str:
        account_balance = "{client_id}, {available:.4f}, {held:.4f}, {total:.4f}, {locked}".format(
            client_id=self.client_id,
            available=self.available,
            held=self.held,
            total=self.total,
            locked=str(self.locked).lower()
        )
        return account_balance

    def calculate_balance(self) -> str:
        try:
            for transaction in self.transactions_df.itertuples(index=False):
                self.last_index = transaction.row_index
                if self.locked:
                    if transaction.type in [DEPOSIT, WITHDRAWAL]:
                        logging.info('account for client {client_id} is locked. ignoring {transaction}'.format(
                            client_id=self.client_id,
                            transaction=tuple(transaction)))
                        continue
                self.dispatch_transaction(transaction)

        except Exception as e:
            logging.error("{} produced error: {}".format(tuple(transaction), e))
            traceback.print_exc()
            raise

        return self.printable_balance()

    def dispatch_transaction(self, transaction) -> None:
        method_name = 'transaction_' + transaction.type
        method = getattr(self, method_name, lambda a, b: logging.info(
            'ignoring invalid transaction {}'.format(transaction)))

        method(transaction.tx, transaction.amount)
        logging.debug('processed {} --> <{}> '.format(tuple(transaction), self.printable_balance()))

    def transaction_deposit(self, _, amount) -> None:
        self.available += amount
        self.total     += amount

    def transaction_withdrawal(self, tx, amount) -> None:
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

    def transaction_dispute(self, tx, _) -> None:
        if tx in self.transactions_solved_dict.keys():
            logging.warning('ignoring multiple dispute request for transaction {}'.format(
                self.transactions_solved_dict[tx]))
            return

        if tx in self.transactions_disputed_dict.keys():
            logging.warning('Ignoring dispute request while dispute active for {}'.format(
                self.transactions_disputed_dict[tx]))
            return

        df = self.transactions_df
        transactions_disputed_df = df[(df.row_index <= self.last_index) & (df.tx == tx) & (df.amount.notnull())]
        if len(transactions_disputed_df) != 1:
            logging.error('fuzzy disputed transactions for client {client_id} and tx {tx} '
                          '--->  {size} {transactions} '.format(
                            client_id=self.client_id,
                            tx=tx,
                            size=len(transactions_disputed_df),
                            transactions=list(transactions_disputed_df.itertuples(index=False, name=None))))
            return

        transaction_disputed = transactions_disputed_df.iloc[0]

        if transaction_disputed.type not in [DEPOSIT, WITHDRAWAL]:
            logging.warning('TODO: clarify rules for {undef} dispute; '
                            'ignore dispute on {transaction}'.format(
                                undef=transaction_disputed.type,
                                transaction=str(transaction_disputed)
                            ))
            return

        self.available -= transaction_disputed.amount
        self.held      += transaction_disputed.amount
        self.transactions_disputed_dict[tx] = transaction_disputed

    def get_transaction_disputed(self, tx, requested_by='action'):
        transaction_disputed = self.transactions_disputed_dict.get(tx, None)
        if transaction_disputed is None:
            logging.info('ignoring {action} request for client {client_id} and tx {tx} '
                         'due to non-active dispute'.format(
                            action=requested_by,
                            client_id=self.client_id,
                            tx=tx))
        return transaction_disputed

    def transaction_resolve(self, tx, _) -> None:
        transaction_disputed = self.get_transaction_disputed(tx, requested_by=RESOLVE)
        if transaction_disputed is None:
            return

        self.available += transaction_disputed.amount
        self.held      -= transaction_disputed.amount
        del(self.transactions_disputed_dict[tx])
        self.transactions_solved_dict[tx] = transaction_disputed

    def transaction_chargeback(self, tx, _) -> None:
        transaction_disputed = self.get_transaction_disputed(tx, requested_by=CHARGEBACK)
        if transaction_disputed is None:
            return

        self.total -= transaction_disputed.amount
        self.held  -= transaction_disputed.amount
        self.locked = True

        del self.transactions_disputed_dict[tx]
        self.transactions_solved_dict[tx] = transaction_disputed

        logging.warning('BUSINESS FLAG RED: Client {client_id} locked due to '
                        '{chargeback} {transaction}'.format(
                            client_id=self.client_id,
                            chargeback=CHARGEBACK,
                            transaction=tuple(transaction_disputed)))


class ParallelExecutor:
    def __init__(self):
        self.pool = Pool()

    @staticmethod
    def print_balance(formatted_balance):
        print(formatted_balance)

    @staticmethod
    def calculate_client_balance(client_records_df) -> str:
        try:
            return Account(client_records_df).calculate_balance()
        except Exception as e:
            logging.error("Error: {}".format(e))
            traceback.print_exc()
            raise

    def schedule(self, client_records_list):
        self.pool.apply_async(self.calculate_client_balance,
                              args=client_records_list,
                              callback=self.print_balance)

    def wait(self):
        self.pool.close()
        self.pool.join()


def process_records(records_df):
    print('client, available, held, total, locked', flush=True)
    client_ids = records_df['client'].unique().tolist()
    executor = ParallelExecutor()

    for _, client_id in enumerate(client_ids):
        client_records_df = records_df[records_df['client'] == client_id]
        executor.schedule((client_records_df,))

    executor.wait()


if __name__ == '__main__':

    input_file = sys.argv[1]

    records_df = pd.read_csv(input_file, header=0, comment="#",
                             dtype={'type': str, 'client': int, 'tx': int, 'amount': float})

    # this alternative is much slower but handles whitespaces from input file
    # records_df = pd.read_csv(input_file, header=0, comment="#",
    #                          dtype={'type': str, 'client': int, 'tx':int, 'amount':float},
    #                          sep=r'\s*,\s*', encoding='ascii', engine='python')
    process_records(records_df)
