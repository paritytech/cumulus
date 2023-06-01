use subxt::{Config, SubstrateConfig, config::extrinsic_params::{ExtrinsicParams, Era}};
use codec::{Compact, Encode};
use core::fmt::Debug;

#[derive(Debug)]
pub enum GluttonConfig {}

impl Config for GluttonConfig {
    type Index = <SubstrateConfig as Config>::Index;
    type Hash = <SubstrateConfig as Config>::Hash;
    type Hasher = <SubstrateConfig as Config>::Hasher;
    type Header = <SubstrateConfig as Config>::Header;
    type AccountId = <SubstrateConfig as Config>::AccountId;
    type Address = <SubstrateConfig as Config>::Address;
    type Signature = <SubstrateConfig as Config>::Signature;
	// We need our custom ExtrinsicParams to remove the unexpected `tip` param
    type ExtrinsicParams = GluttonExtrinsicParams<Self>;
}

pub type GluttonExtrinsicParams<T> = BaseExtrinsicParams<T>;

#[derive(Debug)]
pub struct BaseExtrinsicParams<T: Config> {
    era: Era,
    nonce: T::Index,
    spec_version: u32,
    transaction_version: u32,
    genesis_hash: T::Hash,
    mortality_checkpoint: T::Hash,
    marker: std::marker::PhantomData<T>,
}

pub struct BaseExtrinsicParamsBuilder<T: Config> {
    era: Era,
    mortality_checkpoint: Option<T::Hash>,
}

impl<T: Config> Default for BaseExtrinsicParamsBuilder<T> {
    fn default() -> Self {
        Self {
            era: Era::Immortal,
            mortality_checkpoint: None,
        }
    }
}

impl<T: Config + Debug> ExtrinsicParams<T::Index, T::Hash>
    for BaseExtrinsicParams<T>
{
    type OtherParams = BaseExtrinsicParamsBuilder<T>;

    fn new(
        // Provided from subxt client:
        spec_version: u32,
        transaction_version: u32,
        nonce: T::Index,
        genesis_hash: T::Hash,
        // Provided externally:
        other_params: Self::OtherParams,
    ) -> Self {
        BaseExtrinsicParams {
            era: other_params.era,
            mortality_checkpoint: other_params.mortality_checkpoint.unwrap_or(genesis_hash),
            nonce,
            spec_version,
            transaction_version,
            genesis_hash,
            marker: std::marker::PhantomData,
        }
    }

    fn encode_extra_to(&self, v: &mut Vec<u8>) {
        let nonce: u64 = self.nonce.into();
        (self.era, Compact(nonce)).encode_to(v); // `tip` not included
    }

    fn encode_additional_to(&self, v: &mut Vec<u8>) {
        (
            self.spec_version,
            self.transaction_version,
            self.genesis_hash,
            self.mortality_checkpoint,
        )
            .encode_to(v);
    }
}
