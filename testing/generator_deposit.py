# python generator_deposit.py > transactions_deposit_many_users.csv
# max u32 = 4294967295

print('type,client,tx,amount')
for i in range(42949672):
    print('deposit, {}, {}, 1.0'.format(1+i%100,i))
