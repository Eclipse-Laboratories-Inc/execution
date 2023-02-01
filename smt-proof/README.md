
# Solana verify service with smt proof 

## introduction
This is a service to verify solana ledger with smt proof.
It will read the ledger from blockstore and verify the smt proof with the smt root from postgresql.

## main steps
1. when solana-validator/solana-test-validator starts, it will create a genesis ledger and store it in blockstore.
2. solana/core/validator loaded "Geyser" plugin, which will create a postgresql database and store smt root in it.
Geyser plugin will also create a notifier hook, which will be called when a transaction is processed.
so accounts, slots, blocks and transactions, shreds(encoded entries) can be can be transmitted to PG.
the entry format in pg is like:

|slot| parent_slot|entry_index| is_full_slot|updated_on |
|:------ | :------ | :------ | :------ | :------ |
|1	|0	|0	|1 KB	|t	|2023-01-24 19:50:35.875732|
|1	|0	|1	|1 KB	|t	|2023-01-24 19:50:35.875732|
|1	|0	|2	|1 KB	|t	|2023-01-24 19:50:35.875732|
|1	|0	|3	|1 KB	|t	|2023-01-24 19:50:35.875732|
|2	|1	|0	|1 KB	|t	|2023-01-24 19:50:36.378391|
|2	|1	|1	|1 KB	|t	|2023-01-24 19:50:36.378391|
|2	|1	|2	|1 KB	|t	|2023-01-24 19:50:36.378391|
|2	|1	|3	|1 KB	|t	|2023-01-24 19:50:36.378391|
  
3. when smt-proof starts, it will:
* set flags in ProcessOptions
* load GenesisConfig from <ledger>/genesis.bin
* construct blockstore from shred stored in postgresql table "entry"
* replay the ledger from blockstore
* notify the notifier hook when a transaction is processed
* notify AccountSharedData in Mock "GeyserPlugin", then SequenceClient will spawn a single worker thread to verify it.
 
## debug in IDEA

```shell
run --package smt-proof --bin smt-proof -- -c <settlement_dir>/solana-accountsdb-plugin-postgres/scripts/geyser.json -l /tmp/test-ledger -o /tmp/out-ledger verify --starting_slot 0 --ending_slot 10 
```

## run in CLI
```shell
smt-proof -c <settlement_dir>/solana-accountsdb-plugin-postgres/scripts/geyser.json -l /tmp/test-ledger -o /tmp/out-ledger verify --starting_slot 0 --ending_slot 10 -l /tmp/test-ledger verify --starting_slot 0 --ending_slot 10 
```

### todo:
* store transaction by CAR file