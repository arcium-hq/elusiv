#![allow(clippy::large_enum_variant)]

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::system_program;
use solana_program::sysvar::instructions;
use crate::network::BasicWardenNetworkAccount;
use crate::warden::{
    ElusivWardenID,
    ElusivBasicWardenConfig,
    WardensAccount,
    BasicWardenAccount,
    BasicWardenStatsAccount,
    try_stats_account_offset,
};
use crate::macros::ElusivInstruction;
use crate::processor;
pub use elusiv_types::accounts::{UserAccount, SignerAccount, WritableUserAccount, WritableSignerAccount};

#[derive(BorshDeserialize, BorshSerialize, ElusivInstruction)]
pub enum ElusivWardenNetworkInstruction {
    // -------- Program initialization --------

    #[acc(payer, { signer, writable })]
    #[pda(wardens, WardensAccount, { writable, find_pda, account_info })]
    #[pda(basic_network, BasicWardenNetworkAccount, { writable, find_pda, account_info })]
    #[sys(system_program, key = system_program::ID, { ignore })]
    Init,

    // -------- Basic Warden --------

    #[acc(warden, { signer, writable })]
    #[pda(warden_account, BasicWardenAccount, pda_offset = Some(warden_id), { writable, find_pda, account_info })]
    #[pda(wardens, WardensAccount, { writable })]
    #[pda(basic_network, BasicWardenNetworkAccount, { writable })]
    #[sys(system_program, key = system_program::ID, { ignore })]
    RegisterBasicWarden {
        warden_id: ElusivWardenID,
        config: ElusivBasicWardenConfig,
    },

    #[acc(warden, { signer })]
    #[pda(warden_account, BasicWardenAccount, pda_offset = Some(warden_id), { writable })]
    UpdateBasicWardenState {
        warden_id: ElusivWardenID,
        is_active: bool,
    },

    #[acc(warden, { signer })]
    #[pda(warden_account, BasicWardenAccount, pda_offset = Some(warden_id), { writable })]
    #[acc(lut_account)]
    UpdateBasicWardenLut {
        warden_id: ElusivWardenID,
    },

    #[acc(warden, { signer, writable })]
    #[pda(warden_account, BasicWardenAccount, pda_offset = Some(warden_id), { writable, account_info })]
    #[sys(system_program, key = system_program::ID, { ignore })]
    CloseBasicWarden {
        warden_id: ElusivWardenID,
    },

    // -------- Basic Warden statistics --------

    #[acc(payer, { signer, writable })]
    #[pda(stats_account, BasicWardenStatsAccount, pda_offset = Some(try_stats_account_offset(warden_id)?), { writable, find_pda, account_info })]
    #[sys(system_program, key = system_program::ID, { ignore })]
    OpenBasicWardenStatsAccount {
        warden_id: ElusivWardenID,
    },

    #[pda(warden_account, BasicWardenAccount, pda_offset = Some(warden_id))]
    #[pda(stats_account, BasicWardenStatsAccount, { writable })]
    #[sys(system_program, key = instructions::ID)]
    TrackBasicWardenStats {
        warden_id: ElusivWardenID,
    },
}