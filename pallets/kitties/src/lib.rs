#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		traits::{Randomness, Time},
		BoundedVec,
	};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// Struct for holding Kitty information.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		pub dna: [u8; 8],
		pub owner: T::AccountId,
		pub price: u64,
		pub gender: Gender,
		pub created_date: <<T as Config>::Time as Time>::Moment,
	}
	// Enum and implementation to handle Gender type in Kitty struct.
	#[derive(Clone, Encode, Decode, PartialEq, Copy, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum Gender {
		Male,
		Female,
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Time: Time;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::storage]
	pub type Number<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

	// Storage Kitty ID.
	#[pallet::storage]
	#[pallet::getter(fn kitty_count)]
	pub type CountKitty<T> = StorageValue<_, u32, ValueQuery>;

	// Storage Kitties map.
	#[pallet::storage]
	#[pallet::getter(fn kitty_list)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, [u8; 8], Kitty<T>>;

	// Storage Kitties owned
	#[pallet::storage]
	#[pallet::getter(fn kitty_owned)]
	pub type KittiesOwned<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<[u8; 8]>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		KittyCreated(T::AccountId, [u8; 8]),
		KittyTransferred(T::AccountId, T::AccountId, [u8; 8]),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		KittyDuplicate,
		KittyOverflow,
		NoneKitty,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.

		// create_kitty
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create_kitty(origin: OriginFor<T>, dna: [u8; 8]) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let owner = ensure_signed(origin)?;

			// check gender
			let gender = Self::generate_gender(dna.clone())?;

			// get create date
			let created_date = T::Time::now();

			// create new kitty instance
			let kitty = Kitty::<T> {
				dna: dna.clone(),
				owner: owner.clone(),
				price: 0,
				gender,
				created_date,
			};

			// check kitty exist in storage or not ?
			// ensure!(<Kitties<T>>::contains_key(kitty.dna.clone()), <Error<T>>::KittyDuplicate);

			// check kitty overflow
			let current_id = <CountKitty<T>>::get();
			let next_id = current_id.checked_add(1).ok_or(<Error<T>>::KittyOverflow)?;

			// Update storage.
			<CountKitty<T>>::put(next_id);
			<Kitties<T>>::insert(dna.clone(), kitty);
			<KittiesOwned<T>>::append(owner.clone(), dna.clone());

			// Emit an event.
			Self::deposit_event(Event::KittyCreated(owner.clone(), dna));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn transfer_kitty(
			origin: OriginFor<T>,
			to: T::AccountId,
			dna: [u8; 8],
		) -> DispatchResult {
			// check signed
			let from = ensure_signed(origin)?;
			let mut kitty = <Kitties<T>>::get(dna.clone()).ok_or(<Error<T>>::NoneKitty)?;

			let mut from_owned = KittiesOwned::<T>::get(&from);

			// Remove kitty from list of owned kitties.
			if let Some(ind) = from_owned.iter().position(|ids| *ids == dna) {
				from_owned.swap_remove(ind);
			} else {
				return Err(<Error<T>>::NoneKitty.into());
			}

			// update new owner for kitty
			let mut to_owned = KittiesOwned::<T>::get(&to);
			to_owned.push(dna.clone());
			kitty.owner = to.clone();

			// Write updates to storage
			<Kitties<T>>::insert(&dna, kitty);
			<KittiesOwned<T>>::insert(&to, to_owned);
			<KittiesOwned<T>>::insert(&from, from_owned);

			Self::deposit_event(Event::KittyTransferred(from.clone(), to.clone(), dna.clone()));
			Ok(())
		}
	}
}
impl<T> Pallet<T> {
	fn generate_gender(dna: [u8; 8]) -> Result<Gender, Error<T>> {
		let mut res = Gender::Female;
		if dna.len() % 2 == 0 {
			res = Gender::Male;
		}
		Ok(res)
	}
}
