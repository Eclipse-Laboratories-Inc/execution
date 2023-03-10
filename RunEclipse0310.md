## Install Rust

```
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env
```



## Build code

```
git clone git@github.com:Eclipse-Laboratories-Inc/execution.git
cd execution
cargo build --release
```



## Install PostgreSQL Server

###  Ubuntu

Please follow [PostgreSQL Ubuntu Installation](https://www.postgresql.org/download/linux/ubuntu/) on instructions to install the PostgreSQL database server. For example, to install postgresql-14,

```
sudo sh -c 'echo "deb http://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list'
wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | sudo apt-key add -
sudo apt-get update
sudo apt-get -y install postgresql-14
```



### MacOS

We can use homebrew to install PostgreSQL:

```
brew install postgresql -v
```



### Configuration database

Next to initialize the database where to store:

```
initdb $HOME/postgres -E utf8
```

Start PostgreSQL:

```
pg_ctl -D $HOME/postgres -l $HOME/postgres/server.log start
```

If you want to stop PostgreSQL:

```
pg_ctl -D $HOME/postgres stop -s -m fast
```

Create a user 'solana' with password '1234':

```
createuser username -P
#Enter password for new role:
#Enter it again:
```

Create database 'solana' owned by user 'solana':

```
createdb solana -O solana -E UTF8 -e
```

Suppose we got a database named `solana`, a username `solana` with password `1234`.

Then we should create Schema Objects in our solana database. Our current directory is still `execution`, so here is the command:

```
psql -U solana -p 5432 -h localhost -d solana -f solana-accountsdb-plugin-postgres/scripts/create_schema.sql
```

If you want to drop all the schema:

```
psql -U solana -p 5432 -h localhost -d solana -f solana-accountsdb-plugin-postgres/scripts/drop_schema.sql
```

Let's explain the parameters in above command:

```
-U -- username
-p -- port of PostgreSQL server
-h -- ip address pf PostgreSQL server
-d -- database name
-f -- the path of SQL script file we want to execute
```

## Configuration plugin

The plugin configure file is `solana-accountsdb-plugin-postgres/scripts/geyser.json`, we need change some settings in it:

```
{
  "libpath": "../../target/release/libsolana_geyser_plugin_postgres.dylib",
  "host": "127.0.0.1",
  "user": "solana",
  "password": "1234",
  "dbname": "solana",
  "port": 5432,
  "threads": 20,
  "batch_size": 20,
  "panic_on_db_errors": true,
  "accounts_selector" : {
      "accounts" : ["*"]
  },
  "transaction_selector" : {
      "mentions" : ["*"]
  },
  "entry_selector" : true
}
```

The configuration details are:

```
libpath -- Our `libsolana_geyser_plugin_postgres` lib, should be in `target/release/libsolana_geyser_plugin_postgres.dylib
host -- PostgreSQL server ip address
user -- Username of database
password -- Paddword of database
dbname -- Database name
port -- Port of PostgreSQL server, in our case, 5432.
```

## Start execution node

For now, we use Test Validator as our execution node, we start it with plugin configure file we just set

```
./target/release/solana-test-validator --geyser-plugin-config ./solana-accountsdb-plugin-postgres/scripts/geyser.json
```

it will store ledge in floder ``./test-ledge``.

## Start verify node

Verify node use the same plugin configure file with exectution node.

And we need to obtain 'genesis.bin' from execution node's ledger directory ``(./test-ledge)``, and store it in verification node file system ``genesis_path``.

```
./target/release/shred-replay -c ./solana-accountsdb-plugin-postgres/scripts/geyser.json -l replay-ledger -g genesis_path
```

The command means:

```
-c, --config <CONFIG>           Configuration file to use [default: config.json]
-g, --genesis <GENESIS_PATH>    Use GENESIS_PATH as genesis path [default: ledger]
-l, --ledger <DIR>              Use DIR as ledger location [default: ledger]
```



## Start RPC service on verify node

```
./target/release/solo_rpc -l replay-ledger
```

It will start the rpc service at `127.0.0.1:9988`

You can test it like this

```
curl http://127.0.0.1:9988 -X POST -H "Content-Type: application/json" -d '{ "jsonrpc":"2.0","id":1, "method":"getBlockHeight"}  '
```

or

```
curl http://127.0.0.1:9988 -X POST -H "Content-Type: application/json" -d '{ "jsonrpc":"2.0","id":1, "method":"getBalance","params":["DFdNGtmFM2irnCf7cu5GTDENdZrksVNMJepms7aPYdxB"]}'
```

