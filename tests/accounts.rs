//! Tests the account setup process
/*
#[cfg(not(tarpaulin_include))]
mod common;
use common::*;
use common::program_setup::*;

use elusiv::fields::fr_to_u256_le;
use elusiv::state::{StorageAccount, MT_COMMITMENT_COUNT, HISTORY_ARRAY_COUNT, EMPTY_TREE};
use elusiv::commitment::CommitmentHashingAccount;
use elusiv::instruction::*;
use elusiv::processor::SingleInstancePDAAccountKind;
use solana_program::account_info::Account;
use solana_program_test::*;
use elusiv::state::{
    queue::{CommitmentQueueAccount, BaseCommitmentQueueAccount},
    program_account::{PDAAccount, SizedAccount, MultiAccountAccount, ProgramAccount, MultiAccountProgramAccount},
    fee::FeeAccount,
    NullifierAccount,
    governor::{GovernorAccount, PoolAccount, FeeCollectorAccount},
    MT_HEIGHT,
};

macro_rules! assert_account {
    ($ty: ty, $context: expr, $offset: expr) => {
        {
            let mut data = get_data(&mut $context, <$ty>::find($offset).0).await;

            // Check balance and data size
            assert!(get_balance(<$ty>::find($offset).0, &mut $context).await > 0);
            assert_eq!(data.len(), <$ty>::SIZE);

            // Check pda account fields
            let account = <$ty>::new(&mut data).unwrap();
            assert_eq!(account.get_bump_seed(), <$ty>::find($offset).1);
            assert_eq!(account.get_version(), 0);
            assert_eq!(account.get_initialized(), false);
        }
    };
}

#[tokio::test]
async fn test_setup_pda_accounts() {
    let mut context = start_program_solana_program_test().await;
    setup_pda_accounts(&mut context).await;

    assert_account!(GovernorAccount, context, None);
    assert_account!(FeeAccount, context, Some(0));
    assert_account!(PoolAccount, context, None);
    assert_account!(FeeCollectorAccount, context, None);

    assert_account!(CommitmentHashingAccount, context, None);
    assert_account!(CommitmentQueueAccount, context, None);
    assert_account!(BaseCommitmentQueueAccount, context, Some(0));
}

#[tokio::test]
#[should_panic]
async fn test_setup_pda_accounts_duplicate() {
    let mut context = start_program_solana_program_test().await;
    setup_pda_accounts(&mut context).await;
    setup_pda_accounts(&mut context).await;
}

#[tokio::test]
async fn test_setup_fee_account() {
    let mut context = start_program_solana_program_test().await;
    let mut payer = Actor::new(&mut context).await;

    ix_should_succeed(
        ElusivInstruction::setup_governor_account_instruction(
            SignerAccount(payer.pubkey),
            WritableUserAccount(GovernorAccount::find(None).0)
        ), &mut payer, &mut context
    ).await;

    let ix = ElusivInstruction::init_new_fee_version_instruction(
        0,
        9999,
        111,
        222,
        1,
        2,
        333,
        SignerAccount(payer.pubkey),
    );

    ix_should_succeed(ix.clone(), &mut payer, &mut context).await;

    // Second time will fail
    ix_should_fail(ix.clone(), &mut payer, &mut context).await;
    
    pda_account!(fee, FeeAccount, Some(0), context);
    assert_eq!(fee.get_lamports_per_tx(), 9999);
    assert_eq!(fee.get_base_commitment_network_fee(), 111);
    assert_eq!(fee.get_proof_network_fee(), 222);
    assert_eq!(fee.get_relayer_hash_tx_fee(), 1);
    assert_eq!(fee.get_relayer_proof_tx_fee(), 2);
    assert_eq!(fee.get_relayer_proof_reward(), 333);

    // Attempting to set a version higher than genesis (0) will fail
    let ix = ElusivInstruction::init_new_fee_version_instruction(
        1,
        9999,
        111,
        222,
        1,
        2,
        333,
        SignerAccount(payer.pubkey),
    );
    ix_should_fail(ix.clone(), &mut payer, &mut context).await;
}

#[tokio::test]
async fn test_setup_pda_accounts_invalid_pda() {
    let mut context = start_program_solana_program_test().await;
    let mut payer = Actor::new(&mut context).await;

    ix_should_fail(
        ElusivInstruction::open_single_instance_account_instruction(
            SingleInstancePDAAccountKind::CommitmentQueueAccount,
            SignerAccount(payer.pubkey),
            WritableUserAccount(BaseCommitmentQueueAccount::find(None).0)
        ),
        &mut payer, &mut context
    ).await;
}

#[tokio::test]
async fn test_setup_storage_account() {
    let mut context = start_program_solana_program_test().await;
    let keys = setup_storage_account(&mut context).await;

    storage_account!(storage_account, context);
    assert!(storage_account.get_initialized());
    for (i, &key) in keys.iter().enumerate() {
        assert_eq!(storage_account.get_pubkeys(i), key.to_bytes());
    }
}

#[tokio::test]
#[should_panic]
async fn test_setup_storage_account_duplicate() {
    let mut context = start_program_solana_program_test().await;
    setup_storage_account(&mut context).await;
    setup_storage_account(&mut context).await;
}

#[tokio::test]
async fn test_open_new_merkle_tree() {
    let mut context = start_program_solana_program_test().await;

    // Multiple MTs can be opened
    for mt_index in 0..3 {
        let keys = create_merkle_tree(&mut context, mt_index).await;

        nullifier_account!(nullifier_account, mt_index, context);
        assert!(nullifier_account.get_initialized());
        for (i, &key) in keys.iter().enumerate() {
            assert_eq!(nullifier_account.get_pubkeys(i), key.to_bytes());
        }
    }
}

#[tokio::test]
#[should_panic]
async fn test_open_new_merkle_tree_duplicate() {
    let mut context = start_program_solana_program_test().await;
    create_merkle_tree(&mut context, 0).await;
    create_merkle_tree(&mut context, 0).await;
}

#[tokio::test]
async fn test_close_merkle_tree() {
    let mut context = start_program_solana_program_test().await;
    let mut client = Actor::new(&mut context).await;
    setup_storage_account(&mut context).await;

    let (_, _, storage_account) = storage_accounts(&mut context).await;

    create_merkle_tree(&mut context, 0).await;
    create_merkle_tree(&mut context, 1).await;
    create_merkle_tree(&mut context, 2).await;

    let (_, _, nullifier_0_w)= nullifier_accounts(0, &mut context).await;
    let (_, _, nullifier_1_w)= nullifier_accounts(1, &mut context).await;
    let (_, _, nullifier_2_w)= nullifier_accounts(2, &mut context).await;

    // Failure since active MT is not full
    ix_should_fail(
        ElusivInstruction::reset_active_merkle_tree_instruction(
            0,
            &storage_account,
            &nullifier_0_w,
            &nullifier_1_w,
        ),
        &mut client,
        &mut context
    ).await;

    // Set active MT as full
    set_pda_account::<StorageAccount, _>(&mut context, None, |data| {
        let mut storage_account = StorageAccount::new(data, vec![]).unwrap();
        storage_account.set_next_commitment_ptr(&(MT_COMMITMENT_COUNT as u32));
        for i in 0..HISTORY_ARRAY_COUNT {
            storage_account.set_active_mt_root_history(i, &[1; 32]);
        }
    }).await;

    // Failure since active_nullifier_account is invalid
    ix_should_fail(
        ElusivInstruction::reset_active_merkle_tree_instruction(
            0,
            &storage_account,
            &nullifier_1_w,
            &nullifier_2_w,
        ),
        &mut client,
        &mut context
    ).await;

    // Failure since next_nullifier_account index is mismatched
    ix_should_fail(
        ElusivInstruction::reset_active_merkle_tree_instruction(
            0,
            &storage_account,
            &nullifier_0_w,
            &nullifier_2_w,
        ),
        &mut client,
        &mut context
    ).await;

    // Success
    ix_should_succeed(
        ElusivInstruction::reset_active_merkle_tree_instruction(
            0,
            &storage_account,
            &nullifier_0_w,
            &nullifier_1_w,
        ),
        &mut client,
        &mut context
    ).await;

    // TODO: Check root in closed tree
    nullifier_account!(nullifier_account, 0, context);
    assert_eq!(nullifier_account.get_root(), fr_to_u256_le(&EMPTY_TREE[MT_HEIGHT as usize]));

    // Check active index
    storage_account!(storage_account, context);
    assert_eq!(storage_account.get_trees_count(), 1);
    for i in 0..HISTORY_ARRAY_COUNT {
        assert_eq!(storage_account.get_active_mt_root_history(i), [0; 32]);
    }
}*/