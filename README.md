# SVM Rollup Node

SVM rollup node which replays transactions. This repo uses a local Postgres rather than an actual DA layer. There are no execution layer or DA customizations in this repo.

## Building

For how to build and test code, it is recommanded to see [solana](https://github.com/solana-labs/solana/blob/master/README.md)'s succinct instructions. 

For Apple chip build, there's a [note](./apple_ chip_build.md) that might help.

### Design

The architecture of design as below:

![Architecture](./architecture-diagram.svg)

There are two roles in our Layer 2: __Execution Layer__ and __Settlement Layer__.

* Execution Layer contains:
  
  * Execution Node
    
    Handles all Layer 2 transactions, produce block, push block to DA. 
  
  * Verification Node:
    
    Pull data from DA, reconstruct block and replay.

* Settlement Layer contains:
  
  * Full node:
    
    Check the challenge and push transaction data to DA.
  
  * Light node:
    
    Sync header data.

## Progress

### 1. Execution Layer

```mermaid
graph TD
    A[Execution Node] --> |push block| B[DA]
    B --> |replay data| C[Verification Node]
```

   For now, since Celestia is still unstable, we use PostgreSQL as a DA simulator, here is the execution flow:

#### Execution flow of Execution Layer

##### 1.1 Execution Node

* The execution node produce blocks, we use [accountsdb-plugin-postgres](./solana-accountsdb-plugin-postgres) to save blocks into PostgreSQL database.
  
  Instructions:
  
  a. Build code with submdoule
    
    ```
    git clone https://github.com/Eclipse-Laboratories-Inc/settlement.git
    cd settlement
    cargo build --release
    ```
  
  b. Setup database
    
    &emsp;&emsp;For detailed documents of how to setup database, see [here](./solana-accountsdb-plugin-postgres#database-setup) .
    
    &emsp;&emsp;Suppose we got a database named `solana`, a username `solana` with password `1234`. 
    
    &emsp;&emsp;Then we should create Schema Objects in our solana  database. Our current directory is still `solana-executor`, so here is the command:
    
    ```
    psql -U solana -p 5432 -h localhost -d solana -f solana-accountsdb-plugin-postgres/scripts/create_schema.sql
    ```
    
    &emsp;&emsp;Let's explain the parameters in above command:
    
    ```
    -U -- username
    -p -- port of PostgreSQL server
    -h -- ip address pf PostgreSQL server
    -d -- database name
    -f -- the path of SQL script file we want to execute
    ```
  
  c. Configure plugin settings
    
    &emsp;The plugin configure file is `solana-accountsdb-plugin-postgres/scripts/geyser.json`, we need change some settings in it:
    
    ```json
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
    
    &emsp;The configuration details are:
    ```
    libpath -- Our `libsolana_geyser_plugin_postgres` lib, should be in `target/release/libsolana_geyser_plugin_postgres.dylib
    host -- PostgreSQL server ip address
    user -- Username of database
    password -- Password of database
    dbname -- Database name
    port -- Port of PostgreSQL server, in our case, 5432.
    ```
  
  d. Start execution node
    
    &emsp;For now, we use Test Validator as our execution node, we start it with plugin configure file we just set.
    
    ```shell
    ./target/release/solana-test-validator --geyser-plugin-config ./solana-accountsdb-plugin-postgres/scripts/geyser.json
    ```
    
    &emsp;Now our test validator start producing blocks, and all these data saved in PostgreSQL.

##### 1.2 Verification Node

* Start verification node

&emsp;&emsp;Prerequisites:

```
* Configuration file of geyser plugin.
* 'genesis.bin' directory path, we need to obtain 'genesis.bin' from execution node's ledger directory, and store it in verification node file system.
```

&emsp;&emsp;Replay and verify shred from PG.

```shell
./target/release/shred-replay -c ./solana-accountsdb-plugin-postgres/scripts/geyser.json -l replay-ledger -g genesis_path
```

&emsp;&emsp;The command means:
```
-c, --config <CONFIG>           Configuration file to use [default: config.json]
-g, --genesis <GENESIS_PATH>    Use GENESIS_PATH as genesis path [default: ledger]
-l, --ledger <DIR>              Use DIR as ledger location [default: ledger]
```

* Start verification node's RPC service

```shell
./target/release/solo_rpc -l replay-ledger
```

&emsp;&emsp;`-l` means the verified ledger of verification node.

### 2. Settlement Layer

    TBD


## Designs of under the hood

#### Core designs of Execution Layer

  For Execution Node: 
  * Adding a `entry_notifier` related functions in `geyser-plugin`, which is used for writing entries into PG when validator actives.

  For Verification Node: 
  * `shred-replay-service` querys entries from PG, converts them to shreds, inserts into `blockstore`.
  *  Use `solana-ledger-tool`'s verify function, generating bank hash for whole blockstore.
  
