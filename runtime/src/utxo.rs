use parity_codec::{Decode, Encode};
use primitives::{H256, H512};
use runtime_io::sr25519_verify;
use runtime_primitives::traits::{BlakeTwo256, Hash};
/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs
use support::{
    decl_event, decl_module, decl_storage,
    dispatch::{Result, Vec},
    ensure, StorageMap, StorageValue,
};
use system::{ensure_root, ensure_signed};

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, PartialEq, Eq, Encode, Decode)]
struct TransactionInput {
    parent_output: H256,
    signature: H512,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, PartialEq, Eq, Encode, Decode)]
struct TransactionOutput {
    value: u128,
    pubkey: H256,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, PartialEq, Eq, Encode, Decode)]
pub struct Transaction {
    inputs: Vec<TransactionInput>,
    outputs: Vec<TransactionOutput>,
}

/// The module's configuration trait.
pub trait Trait: system::Trait {
    // TODO: Add other types and constants required configure this module.

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

/// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as utxo {
        // Just a dummy storage item.
        // Here we are declaring a StorageValue, `Something` as a Option<u32>
        // `get(something)` is the default getter which returns either the stored `u32` or `None` if nothing stored
        Something get(something): Option<u32>;
        UnspentOutputs: map H256 => Option<TransactionOutput>;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event<T>() = default;

        // Just a dummy entry point.
        // function that can be called by the external world as an extrinsics call
        // takes a parameter of the type `AccountId`, stores it and emits an event
        pub fn do_something(origin, something: u32) -> Result {
            // TODO: You only need this if you want to check it was signed.
            let who = ensure_signed(origin)?;

            // TODO: Code to execute when something calls this.
            // For example: the following line stores the passed in u32 in the storage
            <Something<T>>::put(something);

            // here we are raising the Something event
            Self::deposit_event(RawEvent::SomethingStored(something, who));
            Ok(())
        }

        fn mint(origin, value: u128, pubkey: H256) -> Result {
            ensure_root(origin)?;

            let utxo = TransactionOutput {
                value: value,
                pubkey: pubkey,
            };
            let hash = BlakeTwo256::hash_of(&utxo);
            runtime_io::print("new utxo");
            runtime_io::print(hash.as_bytes());
            <UnspentOutputs<T>>::insert(hash, utxo);

            Ok(())
        }

        fn execute(origin, transaction: Transaction) -> Result {
            ensure_root(origin)?;

            Self::check_transaction(&transaction)?;
            Self::update_storage(&transaction)?;
            Self::deposit_event(RawEvent::TransactionExecuted(transaction));
            Ok(())
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        // Just a dummy event.
        // Event `Something` is declared with a parameter of the type `u32` and `AccountId`
        // To emit this event, we call the deposit funtion, from our runtime funtions
        SomethingStored(u32, AccountId),
        TransactionExecuted(Transaction),
    }
);

impl<T: Trait> Module<T> {
    fn check_transaction(transaction: &Transaction) -> Result {
        for input in transaction.inputs.iter() {
            if let Some(output) = <UnspentOutputs<T>>::get(input.parent_output) {
                ensure!(
                    sr25519_verify(
                        input.signature.as_fixed_bytes(),
                        input.parent_output.as_fixed_bytes(),
                        output.pubkey
                    ),
                    "signature must be valid"
                );
            } else {
                return Err("parent output not found");
            }
        }

        Ok(())
    }

    fn update_storage(transaction: &Transaction) -> Result {
        for input in &transaction.inputs {
            <UnspentOutputs<T>>::remove(input.parent_output);
        }

        for output in &transaction.outputs {
            let hash = BlakeTwo256::hash_of(output);
            runtime_io::print("insert utxo");
            runtime_io::print(hash.as_bytes());
            <UnspentOutputs<T>>::insert(hash, output);
        }

        Ok(())
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use primitives::{Blake2Hasher, H256};
    use runtime_io::with_externalities;
    use runtime_primitives::{
        testing::{Digest, DigestItem, Header},
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage,
    };
    use support::{assert_ok, impl_outer_origin};

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    impl system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type Digest = Digest;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type Log = DigestItem;
    }
    impl Trait for Test {
        type Event = ();
    }
    type utxo = Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap()
            .0
            .into()
    }

    #[test]
    fn it_works_for_default_value() {
        with_externalities(&mut new_test_ext(), || {
            // Just a dummy test for the dummy funtion `do_something`
            // calling the `do_something` function with a value 42
            assert_ok!(utxo::do_something(Origin::signed(1), 42));
            // asserting that the stored value is equal to what we stored
            assert_eq!(utxo::something(), Some(42));
        });
    }
}
