How to replay the shred

1. download <test-ledger>, unzip and put into /tmp/test-ledger

https://drive.google.com/file/d/1IXk39jz6lN02D7Xu3bgKgHyKKzFAW65F/view?usp=share_link

2. download solana-pg.sql

https://drive.google.com/file/d/1m5OC_TvdCbHWLYwW_IQyRIt0-7mh_I4z/view?usp=share_link

then load the .sql file into postgres

```bash
psql -U solana -p 5432 -h 127.0.0.1 -d solana -f /tmp/solana-pg.sql
```

3. run shred_replay process, make sure /tmp/test-ledger exists.

```shell
> shred-replay -c <geyser.json> -l <new-ledger-dir> verify
> shred-replay -c /Users/cairo/solana-dev/settlement/solana-accountsdb-plugin-postgres/scripts/geyser.json -l /tmp/ledger2 verify
genesis hash: D65qTaRvYmu15VUugEEuQt4mfc14zvmYFehXtnoSMwxo
Ok

> solana-ledger-tool -l /tmp/ledger2 verify
run l /tmp/ledger2 verify
```

Here's a snapshot showing blocks being verified success.

![img](https://hypnotic-act-be9.notion.site/image/https%3A%2F%2Fs3-us-west-2.amazonaws.com%2Fsecure.notion-static.com%2Fa5ce4e7d-138e-4a3d-aaa4-95fa218fe4dd%2FScreenshot_2022-12-07_at_04_06_42.jpg?id=6135cfa6-f5b1-4b00-ab24-62d6f1019d11&table=block&spaceId=93e9999d-f293-4042-a26d-86e56f9b5935&width=2000&userId=&cache=v2)



Once the verification fails, the stderr will show “entry hash is different”.