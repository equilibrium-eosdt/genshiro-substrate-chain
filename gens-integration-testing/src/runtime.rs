use super::{balances, rate, timestamp, vesting, whitelists};
use crate::balances::BalancesEventsDecoder;
use crate::rate::GensRateEventsDecoder;
use crate::timestamp::TimestampEventsDecoder;
use crate::whitelists::WhitelistsEventsDecoder;
use sp_runtime::traits::{BlakeTwo256, IdentifyAccount, Verify};
use sp_runtime::{generic, MultiSignature, OpaqueExtrinsic};
use substrate_subxt::{
    balances::Balances, // needed for sign extra
    extrinsic::DefaultExtra,
    sudo::Sudo,
    system::System,
    EventsDecoder,
    Metadata,
    Runtime,
};

#[derive(Clone, PartialEq, Debug)]
pub struct GensRuntime;

impl GensRuntime {
    pub fn create_decoder(metadata: Metadata) -> EventsDecoder<GensRuntime> {
        let mut decoder = EventsDecoder::<GensRuntime>::new(metadata);
        decoder.with_balances();
        decoder.with_gens_rate();
        decoder.with_timestamp();
        decoder.with_whitelists();

        decoder
    }
}

impl Eq for GensRuntime {}

type Signature = MultiSignature;

impl Runtime for GensRuntime {
    type Signature = Signature;
    type Extra = DefaultExtra<Self>;
}

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

pub type Address = AccountId;

impl System for GensRuntime {
    type Index = u32;
    type BlockNumber = u64;
    type Hash = sp_core::H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Address = Self::AccountId;
    type Header = generic::Header<Self::BlockNumber, BlakeTwo256>;
    type Extrinsic = OpaqueExtrinsic;
    type AccountData = ();
}

pub type Balance = u64;

impl Balances for GensRuntime {
    type Balance = Balance;
}

impl balances::Balances for GensRuntime {
    type Balance = Balance;
    type Currency = gens_balances::currency::Currency;
}

impl timestamp::Timestamp for GensRuntime {
    type Moment = u64;
}

impl Sudo for GensRuntime {}

impl whitelists::Whitelists for GensRuntime {}

impl rate::GensRate for GensRuntime {}

impl vesting::EqVesting for GensRuntime {
    type Balance = Balance;
}
