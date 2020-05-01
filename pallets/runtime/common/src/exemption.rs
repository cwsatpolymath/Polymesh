use pallet_identity as identity;
use polymesh_common_utilities::{
    asset::Trait as AssetTrait, balances::Trait as BalancesTrait,
    exemption::Trait as ExemptionTrait, identity::Trait as IdentityTrait, Context,
};
use polymesh_primitives::{AccountKey, IdentityId, Signatory, Ticker};

use codec::Encode;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    weights::SimpleDispatchInfo,
};
use frame_system::{self as system, ensure_signed};
use sp_std::{convert::TryFrom, prelude::*};

/// The module's configuration trait.
pub trait Trait: frame_system::Trait + BalancesTrait + IdentityTrait {
    /// The overarching event type.
    type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;
    type Asset: AssetTrait<Self::Balance, Self::AccountId>;
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as exemption {
        // Mapping -> ExemptionList[ticker][TM][DID] = true/false
        ExemptionList get(fn exemption_list): map hasher(blake2_128_concat) (Ticker, u16, IdentityId) => bool;
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// The sender must be a signing key for the DID.
        SenderMustBeSigningKeyForDid,
        /// The sender is not a token owner.
        NotAnOwner,
        /// No change in the state.
        NoChange
    }
}

type Identity<T> = identity::Module<T>;

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event() = default;

        #[weight = SimpleDispatchInfo::FixedNormal(200_000)]
        fn modify_exemption_list(origin, ticker: Ticker, _tm: u16, asset_holder_did: IdentityId, exempted: bool) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            let sender = Signatory::AccountKey(sender_key);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &sender),
                Error::<T>::SenderMustBeSigningKeyForDid
            );

            ensure!(Self::is_owner(&ticker, did), Error::<T>::NotAnOwner);
            let ticker_asset_holder_did = (ticker, _tm, asset_holder_did);
            let is_exempted = Self::exemption_list(&ticker_asset_holder_did);
            ensure!(is_exempted != exempted, Error::<T>::NoChange);

            <ExemptionList>::insert(&ticker_asset_holder_did, exempted);
            Self::deposit_event(Event::ModifyExemptionList(ticker, _tm, asset_holder_did, exempted));

            Ok(())
        }
    }
}

decl_event!(
    pub enum Event {
        ModifyExemptionList(Ticker, u16, IdentityId, bool),
    }
);

impl<T: Trait> Module<T> {
    pub fn is_owner(ticker: &Ticker, sender_did: IdentityId) -> bool {
        T::Asset::is_owner(ticker, sender_did)
    }
}

impl<T: Trait> ExemptionTrait for Module<T> {
    fn is_exempted(ticker: &Ticker, tm: u16, did: IdentityId) -> bool {
        Self::exemption_list((*ticker, tm, did))
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    // use super::*;

    // use substrate_primitives::{Blake2Hasher, H256};
    // use sp_io::with_externalities;
    // use sp_runtime::{
    //     testing::{Digest, DigestItem, Header},
    //     traits::{BlakeTwo256, IdentityLookup},
    //     BuildStorage,
    // };
    // use frame_support::{assert_ok, impl_outer_origin};

    // impl_outer_origin! {
    //     pub enum Origin for Test {}
    // }

    // // For testing the module, we construct most of a mock runtime. This means
    // // first constructing a configuration type (`Test`) which `impl`s each of the
    // // configuration traits of modules we want to use.
    // #[derive(Clone, Eq, PartialEq)]
    // pub struct Test;
    // impl frame_system::Trait for Test {
    //     type Origin = Origin;
    //     type Index = u64;
    //     type BlockNumber = u64;
    //     type Hash = H256;
    //     type Hashing = BlakeTwo256;
    //     type Digest = H256;
    //     type AccountId = u64;
    //     type Lookup = IdentityLookup<Self::AccountId>;
    //     type Header = Header;
    //     type Event = ();
    //     type Log = DigestItem;
    // }
    // impl Trait for Test {
    //     type Event = ();
    // }
    // type exemption = Module<Test>;

    // // This function basically just builds a genesis storage key/value store according to
    // // our desired mockup.
    // fn new_test_ext() -> sp_io::TestExternalities<Blake2Hasher> {
    //     frame_system::GenesisConfig::default()
    //         .build_storage()
    //         .unwrap()
    //         .0
    //         .into()
    // }

    // #[test]
    // fn it_works_for_default_value() {
    //     with_externalities(&mut new_test_ext(), || {
    //         // Just a dummy test for the dummy funtion `do_something`
    //         // calling the `do_something` function with a value 42
    //         assert_ok!(exemption::do_something(Origin::signed(1), 42));
    //         // asserting that the stored value is equal to what we stored
    //         assert_eq!(exemption::something(), Some(42));
    //     });
    // }
}
