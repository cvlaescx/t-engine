# python generator_happy_flows.py > transactions_happy_many_users.csv
MAX_U32 = 4294967295
MAX_U16 = 65535

print('type,client,tx,amount')
for client_id in range(1, MAX_U16):
    tx = client_id * 10 + 1
    print('deposit, {client_id}, {tx}, 3.0'.format(client_id=client_id, tx=tx))
    print('withdrawal, {client_id}, {tx}, 1.0'.format(client_id=client_id, tx=tx+1))
    print('dispute, {client_id}, {tx},'.format(client_id=client_id, tx=tx))
    print('withdrawal, {client_id}, {tx}, 0.5'.format(client_id=client_id, tx=tx+1))
    print('resolve, {client_id}, {tx},'.format(client_id=client_id, tx=tx))
