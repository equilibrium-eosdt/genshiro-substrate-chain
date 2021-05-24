

## Genshiro

## Run

### Single node development chain

Purge any existing developer chain state:

```bash
curl https://getsubstrate.io -sSf | bash -s -- --fast

./scripts/init.sh
```

```bash
cargo build --release
```

cargo  tests

```bash
cargo check --tests
```

Launch all tests

```bash
cargo t
```

```bash
./target/release/gens-node purge-chain --dev
```

Start a development chain with:

```bash
./target/release/gens-node --dev
```

Launch all integration tests
(with dev node launched and listening on localhost:9944)
```bash
cargo r -p gens-integration-testing
```

https://polkadot.js.org/apps/#/explorer

Select network:
Local Node (Own, 127.0.0.1:9944)

Change settings for custom types:
Settings -> Developer ->

```json
{
    "Address": "AccountId",
    "LookupSource": "AccountId",
   "Keys":"SessionKeys2",
   "Balance":"u64",
   "FixedI64":"i64",
   "SignedBalance":{
      "_enum":{
         "Positive":"Balance",
         "Negative":"Balance"
      }
   },
   "Currency":{
      "_enum":[
         "Unknown",
         "Usd",
         "Gens",
         "Eth",
         "Btc",
         "Eos"
      ]
   },
   "BalancesAggregate":{
      "total_issuance":"Balance",
      "total_debt":"Balance"
   }
}
```
### Benchmarks
#### Build
```bash
cd ./node
cargo build -p gens-node --release --features runtime-benchmarks
```
#### Run benchmark
```bash
cd ./palllets/<pallet-name>/src
../../../target/release/gens-node benchmark --chain dev --execution=wasm --wasm-execution=compiled --pallet <pallet_name> --extrinsic "*" --raw --steps 10 --repeat 5 --output
```
You will get as result benachmarks.rs file with WeightInfo for pallet's extrinsics.

## License

Genshiro substrate is [GPL 3.0 licensed].