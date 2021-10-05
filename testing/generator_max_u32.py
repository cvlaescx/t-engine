# python generator_max_u32.py > transactions_max_u32.csv
MAX_U32 = 4294967296
MAX_U16 = 65536

print('type,client,tx,amount')
for client_id in range(1, MAX_U16):
    for client_tx in range(16384):
        tx = client_id * MAX_U16 + client_tx * 4
        print('deposit, {client_id}, {tx}, 3.0'.format(client_id=client_id, tx=tx))
        print('withdrawal, {client_id}, {tx}, 2.0'.format(client_id=client_id, tx=tx+1))
        print('dispute, {client_id}, {tx},'.format(client_id=client_id, tx=tx))
        print('resolve, {client_id}, {tx},'.format(client_id=client_id, tx=tx))
        print('withdrawal, {client_id}, {tx}, .5'.format(client_id=client_id, tx=tx+2))
