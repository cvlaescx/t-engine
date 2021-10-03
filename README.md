<!-- ABOUT THE PROJECT -->
## About The Project

There are many challanges out there, hackerrank style, but the beauty of this one is that scope is not fully defined.

Requirements are short:
* Create an engine that will treat few types of bank transactions: deposit, withdrawal, dispute
* Cover as many cases as possible
* Keep it as simple as possible
* There are some implementation constraints, but I'll not focus on them

<!-- PREREQUISITES -->
## Prerequisites

Make sure that you have all bullets ready
- python >= 3.4
- pandas = 0.24.2

or run the command below to install requirements:
  ```sh
  pip install -r requirements.txt
  ```

Feel free to play with a virtual environment if you want to use it

<!-- USAGE EXAMPLES -->
## Usage
Run with:
`python payment_engine.py testing/transactions.csv > accounts.csv`

Check with:
`diff -w <(sort accounts.csv) <(sort testing/expected.csv)`

Additional testing data:
1. `python testing/generator_deposit.py > testing/transactions_deposit_many_users.csv`
2. `python testing/generator_happy_flows.py > testing/transactions_happy_many_users.csv`
3. your testing data

<!-- BEHIND THE SCENE -->
## Behind the scene
1. Input file must be properly formatted. Current implementation can handle some white spaces but not to many. Provided code contains an alternative to read input file, which handles more white spaces but is much slower because it cannot use cython engine
2. I've focused on end-to-end scenarios. Maybe I've missed some cases, but engine can handle 10000 records in less than one minute, and 10000 transactions should be enough for unit testing equivalents.
3. Due to python limitation to handle big numbers, I've used a bash command to validate results against expected data, but this command does not handle white spaces outside expected csv format
4. I've included some generators to produce testing data for checking the engine against large number of clients or large number of transactions. As we can see, generators are few liners and we can play with them to generate different transaction files. Results can be checked from shell, but we can also generate expected.csv-s
5. If you are curious, you can change logging level at the beginnig of the engine file, but be carefull if you set it to DEBUG when you handle a large amount of data. Logging messages will land to stderr and will not impact output, but performance will be affected
<!-- CONTACT -->
## Contact

Cristian Vlaescu  - cvl.escu@gmail.com

Project Link: [https://github.com/cvlaescx/t-engine](https://github.com/cvlaescx/t-engine)
