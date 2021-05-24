use super::runtime::AccountId;
use crate::balances::{AccountStoreExt, BalancesAggregatesStoreExt};
use crate::key::{AccountKey, DevPubKey, DevPubKeyId, PubKeyStore};
use crate::runtime::GensRuntime;
use crate::vesting::VestedStoreExt;
use crate::vesting::VestingStoreExt;
use core::slice::Iter;
use gens_balances::{currency::Currency, BalancesAggregate, SignedBalance};
use serde::ser::SerializeTupleVariant;
use serde::{Serialize, Serializer};
use sp_arithmetic::{FixedI64, FixedPointNumber};
use sp_core::twox_128;
use sp_keyring::{sr25519::Keyring, AccountKeyring};
use sp_runtime::{AccountId32, ModuleId};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::{cell::RefCell, hash::Hash, str::FromStr};
use substrate_subxt::Client;

// Used for selfless traits implementations
thread_local! {
    static SAVED_TD: RefCell<Option<TestData>> = RefCell::new(Option::None)
}

#[derive(Debug)]
pub enum EthereumAddresses {
    Alice,
    Bob,
    Charlie,
    Dave,
    Eve,
    Ferdie,
    One,
    Two,
}

impl EthereumAddresses {
    pub fn iterator() -> Iter<'static, EthereumAddresses> {
        static ETHEREUM_ADDRESSES: [EthereumAddresses; 8] = [
            EthereumAddresses::Alice,
            EthereumAddresses::Bob,
            EthereumAddresses::Charlie,
            EthereumAddresses::Dave,
            EthereumAddresses::Eve,
            EthereumAddresses::Ferdie,
            EthereumAddresses::One,
            EthereumAddresses::Two,
        ];
        ETHEREUM_ADDRESSES.iter()
    }
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum AccountName {
    Id(DevPubKeyId),
    Unknown(AccountId),
}

impl Serialize for AccountName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.clone() {
            AccountName::Id(id) => {
                let mut state = serializer.serialize_tuple_variant("AccountName", 0, "Id", 1)?;
                state.serialize_field(&id)?;
                state.end()
            }
            AccountName::Unknown(u) => {
                let mut state =
                    serializer.serialize_tuple_variant("AccountName", 1, "Unknown", 1)?;
                state.serialize_field(&u)?;
                state.end()
            }
        }
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct BalanceData {
    pub account: AccountName,
    pub balances: Vec<(Currency, SignedBalance<u64>)>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct VestingInfo {
    locked: u64,
    per_block: u64,
    starting_block: u64,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct MyTestData {
    pub balances: Vec<BalanceData>,
    pub balance_aggregates: Vec<(Currency, BalancesAggregate<u64>)>,
    pub vesting: Vec<(AccountId, VestingInfo)>,
    pub vested: Vec<(AccountId, u64)>,
    pub total: u64,
}

pub fn register_common_pub_keys(pub_key_store: &mut PubKeyStore) {
    for k in Keyring::iter() {
        let acc_key = AccountKey::from(k);
        pub_key_store.register(acc_key.into());
    }

    let known_modules: Vec<_> = vec![b"ge/trsry", b"ge/prcho", b"ge/vestn"]
        .into_iter()
        .map(|&x| DevPubKey::from(ModuleId(x)))
        .collect();

    for m in known_modules {
        pub_key_store.register(m);
    }

    let known_pub_keys: Vec<_> = vec![(
        "Alice//stash",
        "0xbe5ddb1579b72e84524fc29e78609e3caf42e85aa118ebfe0b0ad404b5bdd25f",
    )]
    .into_iter()
    .map(|(name, acc_id_str)| {
        DevPubKey::well_known(name, AccountId32::from_str(acc_id_str).unwrap())
    })
    .collect();

    for k in known_pub_keys {
        pub_key_store.register(k);
    }
}

pub fn register_external_pub_keys(pub_key_store: &mut PubKeyStore, test_data: &TestData) {
    for acc_id in test_data.balances.keys() {
        pub_key_store.register_external(acc_id.clone());
    }
    for acc_id in test_data.vesting.keys() {
        pub_key_store.register_external(acc_id.clone());
    }

    for acc_id in test_data.vested.keys() {
        pub_key_store.register_external(acc_id.clone());
    }
}

pub fn cmp_currencies(c1: &Currency, c2: &Currency) -> Ordering {
    c1.value().cmp(&c2.value())
}

pub fn humanize_balances(
    balances: &HashMap<AccountId, HashMap<Currency, SignedBalance<u64>>>,
    pub_key_store: &PubKeyStore,
) -> Vec<BalanceData> {
    let mut items: Vec<_> = balances
        .iter()
        .map(|(id, bs)| {
            // let account_option = Keyring::from_account_id(&id);

            let mut blncs: Vec<_> = bs.iter().map(|(c, b)| (*c, b.clone())).collect();
            blncs.sort_by(|(c1, _), (c2, _)| cmp_currencies(&c1, &c2));

            BalanceData {
                account: acc_id_to_acc_name(id, pub_key_store),
                balances: blncs,
            }
        })
        .collect();

    items.sort_by(|b1, b2| b1.account.cmp(&b2.account));

    items
}

pub fn humanize_balance_aggregates(
    aggregates: &HashMap<Currency, BalancesAggregate<u64>>,
) -> Vec<(Currency, BalancesAggregate<u64>)> {
    let mut aggs: Vec<_> = aggregates.iter().map(|(c, a)| (*c, a.clone())).collect();
    aggs.sort_by(|(c1, _), (c2, _)| cmp_currencies(&c1, &c2));

    aggs
}

fn acc_id_to_acc_name(acc_id: &AccountId, pub_key_store: &PubKeyStore) -> AccountName {
    if let Some(id) = pub_key_store.get_id(&acc_id) {
        AccountName::Id(id)
    } else {
        AccountName::Unknown(acc_id.clone())
    }
}

pub fn humanize_vesting(
    vesting: &HashMap<AccountId, gens_vesting::VestingInfo<u64, u64>>,
) -> Vec<(AccountId, VestingInfo)> {
    let vst: Vec<_> = vesting
        .iter()
        .map(|(a, p)| {
            (
                a.clone(),
                VestingInfo {
                    locked: p.locked,
                    per_block: p.per_block,
                    starting_block: p.per_block,
                },
            )
        })
        .collect();
    vst
}

pub fn humanize_vested(vested: &HashMap<AccountId, u64>) -> Vec<(AccountId, u64)> {
    let vst: Vec<_> = vested.iter().map(|(a, p)| (a.clone(), p.clone())).collect();
    // vst.sort_by(|b1, b2| b1.account.cmp(&b2.account));
    vst
}

pub fn humanize_test_data(test_data: &TestData, pub_key_store: &PubKeyStore) -> MyTestData {
    MyTestData {
        balances: humanize_balances(&test_data.balances, pub_key_store),
        balance_aggregates: humanize_balance_aggregates(&test_data.balances_aggregates),
        vesting: humanize_vesting(&test_data.vesting),
        vested: humanize_vested(&test_data.vested),
        total: test_data.total,
    }
}

#[derive(Debug, Clone)]
pub struct TestData {
    pub balances: HashMap<AccountId, HashMap<Currency, SignedBalance<u64>>>,
    pub balances_aggregates: HashMap<Currency, BalancesAggregate<u64>>,
    pub vesting: HashMap<AccountId, gens_vesting::VestingInfo<u64, u64>>,
    pub vested: HashMap<AccountId, u64>,
    pub total: u64,
}

impl TestData {
    pub fn new() -> Self {
        TestData {
            balances: HashMap::new(),
            balances_aggregates: HashMap::new(),
            vesting: HashMap::new(),
            vested: HashMap::new(),
            total: 0,
        }
    }

    pub fn clear(&mut self) {
        self.balances.clear();
        self.balances_aggregates.clear();
    }

    pub fn to_static(&self) {
        SAVED_TD.with(|td_ref| {
            td_ref.replace(Some(self.clone()));
        });
    }

    pub fn get_static() -> Self {
        SAVED_TD.with(|td_ref| {
            let result = td_ref.borrow().clone();
            match result {
                Some(td) => td,
                None => panic!("Error during attempt to access LTS TestData - no data"),
            }
        })
    }
}

pub const EPS: i64 = 1_0;

pub trait EqWithEps {
    fn gens_with_eps(&self, other: &Self) -> bool;
}

impl EqWithEps for u64 {
    fn gens_with_eps(&self, other: &Self) -> bool {
        (*self as i64 - *other as i64).abs() < EPS
    }
}

impl EqWithEps for FixedI64 {
    fn gens_with_eps(&self, other: &Self) -> bool {
        (self.into_inner() - other.into_inner()).abs() < EPS
    }
}

impl EqWithEps for BalancesAggregate<u64> {
    fn gens_with_eps(&self, other: &Self) -> bool {
        self.total_debt.gens_with_eps(&other.total_debt)
            && self.total_issuance.gens_with_eps(&other.total_issuance)
    }
}

impl EqWithEps for SignedBalance<u64> {
    fn gens_with_eps(&self, other: &Self) -> bool {
        match (self, other) {
            (&SignedBalance::Positive(ref a), &SignedBalance::Positive(ref b)) => a.gens_with_eps(b),
            (&SignedBalance::Negative(ref a), &SignedBalance::Negative(ref b)) => a.gens_with_eps(b),
            _ => false,
        }
    }
}

impl<K, V> EqWithEps for HashMap<K, V>
where
    K: Eq + Hash,
    V: EqWithEps,
{
    fn gens_with_eps(&self, other: &Self) -> bool {
        self.len() == other.len()
            && self
                .iter()
                .all(|(k, v)| (*other).get(k).map_or(false, |v2| v.gens_with_eps(v2)))
    }
}

impl PartialEq for TestData {
    fn eq(&self, other: &TestData) -> bool {
        self.balances.gens_with_eps(&other.balances)
            && self
                .balances_aggregates
                .gens_with_eps(&other.balances_aggregates)
            && self.vesting == other.vesting
            && self.vested == other.vested
            && self.total == other.total
    }
}

pub fn compare_test_data_keystore(
    left: &TestData,
    right: &TestData,
    pub_key_store: &PubKeyStore,
) -> Result<(), String> {
    if left.eq(right) {
        Ok(())
    } else {
        let pub_key_store_dump = pub_key_store.dump();
        let pub_key_store_dump = serde_json::to_string_pretty(&pub_key_store_dump).unwrap();

        let left = humanize_test_data(left, &pub_key_store);
        let right = humanize_test_data(right, &pub_key_store);

        let left = serde_json::to_string_pretty(&left).unwrap();
        let right = serde_json::to_string_pretty(&right).unwrap();

        let mut buf = String::from(pub_key_store_dump);
        buf.push_str("\n");
        for diff in diff::lines(&left, &right) {
            match diff {
                diff::Result::Left(l) => buf.push_str(&format!("-{}\n", &l)),
                diff::Result::Both(l, _) => buf.push_str(&format!(" {}\n", &l)),
                diff::Result::Right(r) => buf.push_str(&format!("+{}\n", &r)),
            }
        }

        Err(buf)
    }
}

#[macro_export]
macro_rules! assert_gens_test_data_keystore {
    ($left:expr, $right:expr, $data:expr) => {{
        let result = compare_test_data_keystore($left, $right, $data);
        assert!(
            result.is_ok(),
            "{:?} is not equals to right {:?}:\n{}",
            stringify!($left),
            stringify!($right),
            result.unwrap_err()
        );
    }};
}

pub fn compare_test_data(left: &TestData, right: &TestData) -> Result<(), String> {
    let mut pub_key_store = PubKeyStore::new();
    register_common_pub_keys(&mut pub_key_store);
    register_external_pub_keys(&mut pub_key_store, left);
    register_external_pub_keys(&mut pub_key_store, right);
    compare_test_data_keystore(left, right, &pub_key_store)
}

#[macro_export]
macro_rules! assert_gens_test_data {
    ($left:expr, $right:expr) => {{
        let result = compare_test_data($left, $right);
        assert!(
            result.is_ok(),
            "{:?} is not equals to right {:?}:\n{}",
            stringify!($left),
            stringify!($right),
            result.unwrap_err()
        );
    }};
}

pub struct TestContext {}

impl TestContext {
    pub async fn read_storage(
        client: &Client<GensRuntime>,
    ) -> Result<TestData, Box<dyn std::error::Error>> {
        println!("read_storage:1");
        let mut data = TestData::new();
        let mut prefix = twox_128(b"EqBalances").to_vec();
        prefix.extend(twox_128(b"Account").to_vec());
        let mut processed_accounts: HashSet<AccountId> = HashSet::new();
        let mut processed_currencies: HashMap<AccountId, HashSet<Currency>> = HashMap::new();

        data.total = client.total(Option::None).await.unwrap();

        let start_account_id_i = prefix.len() + 16;
        let end_account_id_i = start_account_id_i + 32;

        println!("read_storage:2");
        let mut iter = client.account_iter(None).await?;
        while let Some((key, signed_balance)) = iter.next().await? {
            // log::info!("key encoded: {:?}", hex::encode(&key.0));
            // log::info!("key encoded: {:?}", key.0);
            let acc_id: Vec<_> = key
                .clone()
                .0
                .drain(start_account_id_i..end_account_id_i)
                .collect();
            let acc_id = hex::encode(&acc_id);
            let acc_id = AccountId::from_str(&acc_id).unwrap();
            let currency = Currency::from(key.0[key.0.len() - 1]);
            let acc_proc_currencies = processed_currencies
                .entry(acc_id.clone())
                .or_insert(HashSet::new());
            acc_proc_currencies.insert(currency);
            // log::info!("acc_id: {:?}", acc_id);
            // log::info!("currency num: {:?}", key.0[key.0.len() - 1]);
            // log::info!("currency: {:?}", currency);
            // log::info!("signed_balance: {:?}\n", signed_balance);
            // balances
            let acc_balance = data
                .balances
                .entry(acc_id.clone())
                .or_insert(HashMap::new());
            acc_balance.entry(currency).or_insert(signed_balance);
            processed_accounts.insert(acc_id.clone());
        }
        println!("read_storage:3");
        // empty balances
        for acc in AccountKeyring::iter() {
            let acc_id = acc.to_account_id();
            if !processed_accounts.contains(&acc_id) {
                processed_currencies.insert(acc_id.clone(), HashSet::new());
                data.balances.insert(acc_id.clone(), HashMap::new());
            }

            //vesting
            let vesting = client.vesting(acc_id.clone(), Option::None).await;
            if let Option::Some(vst) = vesting.unwrap() {
                data.vesting.insert(acc_id.clone(), vst);
            }

            //vested
            let vested = client.vested(acc_id.clone(), Option::None).await;
            if let Option::Some(vst) = vested.unwrap() {
                data.vested.insert(acc_id.clone(), vst);
            }
        }
        println!("read_storage:4");
        for currency in Currency::iterator_with_usd() {
            for (acc_id, proc_currencies) in &processed_currencies {
                if !proc_currencies.contains(currency) {
                    let balances = data.balances.get_mut(&acc_id).unwrap();
                    balances.insert(*currency, SignedBalance::Positive(0));
                }
            }
        }
        println!("read_storage:5");
        // currency aggregates
        for currency in Currency::iterator_with_usd() {
            let balance_aggregate = client.balances_aggregates(*currency, Option::None).await;
            data.balances_aggregates
                .insert(*currency, balance_aggregate.unwrap());
        }

        println!("read_storage:done");
        Ok(data)
    }
}
