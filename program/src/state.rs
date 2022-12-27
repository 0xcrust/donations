use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use arrayref::{array_ref, array_refs, array_mut_ref, mut_array_refs};

/// Campaign state; LEN = 1 + 32 + 200 + 8 + 8 + 1 + 1 = 251
pub struct Campaign {
    pub is_initialized: bool,
    pub authority: Pubkey,
    pub vault: Pubkey,
    pub description: [u8; 200],
    pub target: u64,
    pub amount_raised: u64,
    pub bump: u8,
}

impl Sealed for Campaign {}

impl IsInitialized for Campaign {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for Campaign {
    const LEN: usize = 282;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, Campaign::LEN];
        let (
            is_initialized,
            authority,
            vault,
            description,
            target,
            amount_raised,
            bump,
        ) = array_refs![src, 1, 32, 32, 200, 8, 8, 1];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(Campaign {
            is_initialized,
            authority: Pubkey::new_from_array(*authority),
            vault: Pubkey::new_from_array(*vault),
            description: *description,
            target: u64::from_le_bytes(*target),
            amount_raised: u64::from_le_bytes(*amount_raised),
            bump: bump[0],
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, Campaign::LEN];
        let (
            is_initialized_dst,
            authority_dst,
            vault_dst,
            description_dst,
            target_dst,
            amount_raised_dst,
            bump_dst
        ) = mut_array_refs![dst, 1, 32, 32, 200, 8, 8, 1];

        is_initialized_dst[0] = self.is_initialized as u8;
        authority_dst.copy_from_slice(self.authority.as_ref());
        vault_dst.copy_from_slice(self.vault.as_ref());
        description_dst.copy_from_slice(self.description.as_ref());
        *target_dst = self.target.to_le_bytes();
        *amount_raised_dst = self.amount_raised.to_le_bytes();
        bump_dst[0] = self.bump;
    }
}