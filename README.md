<!-- ABOUT THE PROJECT -->
## About The Project

There are many challenges out there, HackerRank style, but the beauty of this one is that scope is not fully defined.

Requirements are short:
* Create an engine that will treat few types of bank transactions: deposit, withdrawal, dispute
* Cover as many cases as possible
* Keep it as simple as possible
* Engine can handle large amount of data?

<!-- SOLUTION -->
## Solution

<details>
<summary>Python version</summary>
<!-- PREREQUISITES -->

##Prerequisites

Make sure that you have all bullets ready
- python >= 3.4
- pandas = 0.24.2

or run the command below to install requirements:
  ```sh
  pip install -r requirements.txt
  ```

Feel free to play with a virtual environment if you want to use it

<!-- PYTHON USAGE EXAMPLES -->
## Usage

Run with:
`python payment_engine.py testing/transactions.csv > accounts.csv`

Check with:
`diff -w <(sort accounts.csv) <(sort testing/expected.csv)`

Additional testing data:
1. `python testing/generator_deposit.py > testing/transactions_deposit_many_users.csv`
2. `python testing/generator_happy_flows.py > testing/transactions_happy_many_users.csv`
3. `python testing/generator_max_u32.py > testing/transactions_max_u32.csv`
4. Your testing data. Please make sure that first line of input file is `type,client,tx,amount` - without any white space

<!-- PYTHON BEHIND THE SCENE -->
## Behind the scene
1. Input file must be properly formatted. Current implementation can handle some white spaces but it doesn't cover all white spaces. Provided code contains an alternative to read input file, which handles more white spaces but is much slower because it cannot use cython engine for Pandas
2. I've focused on end-to-end scenarios. Maybe I've missed some cases, but engine can handle 10000 records in less than one minute, and 10000 transactions should be enough for unit testing equivalents.
3. Due to python limitation to handle big numbers, I've used a bash command to validate results against expected data. Drawback is that this command does not handle white spaces outside expected csv format
4. I've included some generators to produce testing data for checking the engine against large number of clients or large number of transactions. As we can see, generators are few liners and we can play with them to generate different transaction files. Results can be checked from shell, but we can also generate expected.csv-s
5. If you are curious, you can change logging level at the beginnig of the engine file, but be careful if you set it to DEBUG when you handle a large amount of data. Logging messages will land to stderr and will not impact output, but performance will be affected
6. If you've checked results following steps from [Usage](#usage), then you've discovered that some tests are failed. That's because scope is not defined yet for big numbers.
7. I've played with generators, but I have a lazy processor and I didn't try the engine with transaction files generated by generators, as they are in repo. If you try them, please share some feedback.
8. **BE AWARE !** Engine will try to eat as many resources as possible. At some point, your machine may freeze. Keep the faith and let the engine run. Or put a tail on results and check if there is something new after 1-2 minutes.

</details>


<details>
<summary>Rust version</summary>
<!-- RUST_PREREQUISITES -->

##PREREQUISITES
You'll need to go into Rust project: `cd tr-engine`

Make sure that you have an active internet connection. Cargo will take care about dependencies.

<!-- PYTHON USAGE EXAMPLES -->
## Usage

Run with:
`cargo run -- ../testing/transactions.csv > accounts.csv`

Check with:
`cargo test tests::tests` - tests 50, 67, 68 and 73 will fail due to numbers bigger than expected

<!-- RUST BEHIND THE SCENE -->
## Behind the scene
1. Rust version is still in development. It's my first Rust project and I'll use it to explore some limits.
2. Currently, on my same lazy processor, for some datasets, Rust version performs 50 times quicker than Python version
3. This comes also with the cost of machine freeze for some periods.
4. Still, I plan to see if it can do better. Adding some benchmarks will help to keep trace
5. For a pretty large csv, after some time, process is killed by OS. I'll have to make it work with pretty larger csv-s and also stop gracefully. Maybe my friend Valgrind will have a word to say
</details>


<!-- CONTACT -->
## Contact

Cristian Vlaescu  - cvl.escu@gmail.com

Project Link: [https://github.com/cvlaescx/t-engine](https://github.com/cvlaescx/t-engine)
