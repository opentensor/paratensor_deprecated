/* 
use frame_support::{parameter_types, traits::{StorageMapShim, Contains}};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};


type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		SubtensorModule: pallet_subtensor::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Config<T>, Storage, Event<T>},
		//Subtensor: pallet_subtensor::{Pallet, Call, Config, Storage, Event<T>},
	}
);

#[allow(dead_code)]
pub type SubtensorCall = pallet_subtensor::Call<Test>;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
	pub const TransactionByteFee: Balance = 100;
	pub const SDebug:u64 = 1;
	pub const InitialRho: u64 = 10;
	pub const InitialKappa: u64 = 2;
	pub const SelfOwnership: u64 = 2;
	pub const InitialValidatorBatchSize: u64 = 10;
	pub const InitialValidatorSequenceLen: u64 = 10;
	pub const InitialValidatorEpochLen: u64 = 10;
	pub const InitialValidatorEpochsPerReset: u64 = 10;
	pub const InitialImmunityPeriod: u64 = 2;
	pub const InitialMaxAllowedUids: u64 = 100;
	pub const InitialMinAllowedWeights: u64 = 0;
	pub const InitialBondsMovingAverage: u64 = 500_000;
	pub const InitialIncentivePruningDenominator: u64 = 1;
	pub const InitialStakePruningDenominator: u64 = 1;
	pub const InitialFoundationDistribution: u64 = 0;

	pub const InitialMaxAllowedMaxMinRatio: u64 = 0;
	pub const InitialBlocksPerStep: u64 = 1;
	pub const InitialIssuance: u64 = 548833985028256;
	pub const InitialDifficulty: u64 = 10000;
	pub const MinimumDifficulty: u64 = 10000;
	pub const InitialActivityCutoff: u64 = 5000;
	pub const MaximumDifficulty: u64 = u64::MAX/4;
	pub const InitialAdjustmentInterval: u64 = 100;
	pub const InitialMaxRegistrationsPerBlock: u64 = 2;
	pub const InitialTargetRegistrationsPerInterval: u64 = 2;
}

/// Balance of an account.
#[allow(dead_code)]
pub type Balance = u128;

pub struct TestBaseCallFilter;
impl Contains<Call> for TestBaseCallFilter {
	fn contains(c: &Call) -> bool {
		match *c {
			// Transfer works. Use `transfer_keep_alive` for a call that doesn't pass the filter.
			Call::Balances(pallet_balances::Call::transfer { .. }) => true,
			// For benchmarking, this acts as a noop call
			Call::System(frame_system::Call::remark { .. }) => true,
			_ => false,
		}
	}
}

impl system::Config for Test {
	type BaseCallFilter = TestBaseCallFilter;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_subtensor::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type TransactionByteFee = TransactionByteFee;
	type SDebug = SDebug;
	type InitialRho = InitialRho;
	type InitialKappa = InitialKappa;
	type SelfOwnership = SelfOwnership;
	
	type InitialValidatorBatchSize = InitialValidatorBatchSize;
	type InitialValidatorSequenceLen = InitialValidatorSequenceLen;
	type InitialValidatorEpochLen = InitialValidatorEpochLen;
	type InitialValidatorEpochsPerReset = InitialValidatorEpochsPerReset;

	type InitialImmunityPeriod = InitialImmunityPeriod;
	type InitialMaxAllowedUids = InitialMaxAllowedUids;
	type InitialMinAllowedWeights = InitialMinAllowedWeights;
	type InitialBondsMovingAverage = InitialBondsMovingAverage;
	type InitialMaxAllowedMaxMinRatio = InitialMaxAllowedMaxMinRatio;
	type InitialStakePruningDenominator = InitialStakePruningDenominator;
	type InitialIncentivePruningDenominator = InitialIncentivePruningDenominator;
	type InitialFoundationDistribution = InitialFoundationDistribution;
	type InitialIssuance = InitialIssuance;
	type InitialDifficulty = InitialDifficulty;
	type MinimumDifficulty = MinimumDifficulty;
	type MaximumDifficulty = MaximumDifficulty;
	type InitialBlocksPerStep = InitialBlocksPerStep;
	type InitialActivityCutoff = InitialActivityCutoff;
	type InitialAdjustmentInterval = InitialAdjustmentInterval;
	type InitialMaxRegistrationsPerBlock = InitialMaxRegistrationsPerBlock;
	type InitialTargetRegistrationsPerInterval = InitialTargetRegistrationsPerInterval;
}

#[allow(dead_code)]
pub type AccountId = u64;

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ();
	type AccountStore = StorageMapShim<
		pallet_balances::Account<Test>,
		frame_system::Provider<Test>,
		AccountId,
		pallet_balances::AccountData<Balance>,
	>;
	type MaxLocks = ();
	type WeightInfo = ();
	type MaxReserves = ();
	type ReserveIdentifier = ();
}

#[allow(dead_code)]
pub(crate) fn step_block(n: u64) {
	for _ in 0..n {
		SubtensorModule::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		SubtensorModule::on_initialize(System::block_number());
	}
}

#[allow(dead_code)]
pub fn register_ok_neuron_with_nonce( hotkey_account_id: u64, coldkey_account_id: u64, nonce: u64 ) -> NeuronMetadata<u64> {
	let block_number: u64 = SubtensorModule::get_current_block_as_u64();
	let (nonce2, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number( block_number, nonce );
	let result = SubtensorModule::register( <<Test as frame_system::Config>::Origin>::signed(hotkey_account_id), block_number, nonce2, work, hotkey_account_id, coldkey_account_id );
	assert_ok!(result);
	let neuron = SubtensorModule::get_neuron_for_hotkey(&hotkey_account_id);
	neuron
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
*/
