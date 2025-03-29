// pallets/legal_and_compliance/tests.rs

use crate as legal_and_compliance;
use frame_support::{
    assert_noop, assert_ok,
    traits::{OnFinalize, OnInitialize},
};
use sp_core::H256;
use frame_system as system;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

use legal_and_compliance::Event as ComplianceEvent;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Legal: legal_and_compliance,
    }
);

#[derive(Clone, Eq, PartialEq)]
pub struct Test;

impl system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type AccountId = u64;
    type Lookup = IdentityLookup<u64>;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type Header = Header;
    type RuntimeDbWeight = ();
    type BlockHashCount = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl legal_and_compliance::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MinComplianceLevel = MinComplianceLevel;
}

pub struct MinComplianceLevel;
impl frame_support::traits::Get<u32> for MinComplianceLevel {
    fn get() -> u32 {
        10 // Niveau minimum requis pour conformité
    }
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    let ext: sp_io::TestExternalities = storage.into();
    ext
}

#[test]
fn compliance_update_success() {
    new_test_ext().execute_with(|| {
        let account_id = 1;
        let new_status = 20;

        assert_ok!(Legal::update_compliance_status(RuntimeOrigin::signed(account_id), new_status));

        // Vérifie la valeur stockée
        assert_eq!(Legal::compliance_status(account_id), new_status);

        // Vérifie l'historique
        let history = Legal::compliance_history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0], (account_id, new_status, 1_640_000_000));

        // Vérifie l’événement
        system::Pallet::<Test>::assert_last_event(RuntimeEvent::Legal(
            ComplianceEvent::ComplianceUpdated(account_id, new_status),
        ));
    });
}

#[test]
fn compliance_update_rejected_below_min_level() {
    new_test_ext().execute_with(|| {
        let account_id = 42;
        let new_status = 5; // inférieur au niveau minimum

        assert_noop!(
            Legal::update_compliance_status(RuntimeOrigin::signed(account_id), new_status),
            legal_and_compliance::Error::<Test>::ComplianceLevelTooLow
        );

        // Aucune mise à jour ne doit avoir été faite
        assert_eq!(Legal::compliance_status(account_id), 0);
        assert!(Legal::compliance_history().is_empty());
    });
}

#[test]
fn multiple_updates_logged_correctly() {
    new_test_ext().execute_with(|| {
        let account_id = 7;

        assert_ok!(Legal::update_compliance_status(RuntimeOrigin::signed(account_id), 15));
        assert_ok!(Legal::update_compliance_status(RuntimeOrigin::signed(account_id), 25));

        let status = Legal::compliance_status(account_id);
        assert_eq!(status, 25);

        let history = Legal::compliance_history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0], (account_id, 15, 1_640_000_000));
        assert_eq!(history[1], (account_id, 25, 1_640_000_000));
    });
}

#[test]
fn edge_case_zero_status_rejected() {
    new_test_ext().execute_with(|| {
        let account_id = 99;
        assert_noop!(
            Legal::update_compliance_status(RuntimeOrigin::signed(account_id), 0),
            legal_and_compliance::Error::<Test>::ComplianceLevelTooLow
        );
    });
}
