use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
};

use crate::{
    error::CampaignError,
    instruction::Instruction,
};

use crate::state::Campaign;
use std::ops::DerefMut;

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = Instruction::unpack(instruction_data)?;

        match instruction {
            Instruction::InitCampaign { target, description } => {
                msg!("Instruction: Initialize fundraiser campaign");
                Self::process_init_campaign(accounts, program_id, target, description)
            },
            Instruction::Donate { amount } => {
                msg!("Instruction: Donate");
                Self::process_donate(accounts, program_id, amount)
            },
            Instruction::Withdraw {} => {
                msg!("Instruction: Withdraw");
                Self::process_withdraw(accounts, program_id)
            }
        }
    }

    fn process_init_campaign(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        target: u64,
        description: [u8; 200]
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let fundstarter = next_account_info(account_info_iter)?;
    
        msg!("Verify signer");
        if !fundstarter.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        msg!("Verify campaign account seeds");
        let (campaign_pda, campaign_bump) = Pubkey::find_program_address(&[b"campaign".as_ref(), fundstarter.key.as_ref()], program_id);
        let campaign_state = next_account_info(account_info_iter)?;
        if &campaign_pda != campaign_state.key {
            return Err(CampaignError::AccountMismatch.into());
        }
        msg!("Check if campaign account is initialized");
        if **campaign_state.try_borrow_lamports()? > 0 {
            return Err(CampaignError::AccountAlreadyInitialized.into());
        }

        let vault_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

        let lamports = Rent::default().minimum_balance(Campaign::LEN);
        let create_campaign_state_ix = solana_program::system_instruction::create_account(
            fundstarter.key,
            campaign_state.key,
            lamports,
            Campaign::LEN as u64,
            program_id,
        );
        let state_seeds = &[
            b"campaign".as_ref(),
            fundstarter.key.as_ref(),
            &[campaign_bump]
        ];
        msg!("Create campaign instruction");
        invoke_signed(
            &create_campaign_state_ix,
            &[
                fundstarter.clone(),
                campaign_state.clone(),
                system_program.clone(),  
            ],
            &[&state_seeds[..]],
        )?;

        msg!("Write to campaign account");
        let mut campaign_info = Campaign::unpack_unchecked(&campaign_state.try_borrow_data()?)?;
        campaign_info.is_initialized = true;
        campaign_info.authority = *fundstarter.key;
        campaign_info.description = description;
        campaign_info.target = target;
        campaign_info.amount_raised = 0;
        campaign_info.bump = campaign_bump;
        
        msg!("Verify vault account seeds");
        let (vault_pda, vault_bump) = Pubkey::find_program_address(&[b"vault".as_ref(), campaign_state.key.as_ref()], program_id);
        if &vault_pda != vault_account.key {
            return Err(CampaignError::AccountMismatch.into());
        }
        if **vault_account.try_borrow_lamports()? > 0 {
            return Err(CampaignError::AccountAlreadyInitialized.into());
        }
        let lamports = Rent::default().minimum_balance(0);
        let create_vault_ix = solana_program::system_instruction::create_account(
            fundstarter.key,
            vault_account.key,
            lamports,
            0,
            program_id,
        );
        let vault_seeds = &[
            b"vault".as_ref(),
            campaign_state.key.as_ref(),
            &[vault_bump]
        ];

        msg!("Create vault instruction");
        invoke_signed(
            &create_vault_ix,
            &[
                fundstarter.clone(),
                vault_account.clone(),
                system_program.clone(),  
            ],
            &[&vault_seeds[..]],
        )?;

        campaign_info.vault = *vault_account.key;
        Campaign::pack(campaign_info, &mut campaign_state.try_borrow_mut_data()?)?;
        Ok(())
    }

    fn process_donate(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let donator = next_account_info(account_info_iter)?;
       
        msg!("Verify signer");
        if !donator.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let campaign_state = next_account_info(account_info_iter)?;
        if campaign_state.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        
        msg!("Unpack campaign account info");
        let mut campaign_info = Campaign::unpack(&campaign_state.try_borrow_data()?)?;

        let vault = next_account_info(account_info_iter)?;
        if campaign_info.vault != *vault.key {
            return Err(CampaignError::AccountMismatch.into());
        }

        msg!("Transfer from donator to vault");
        let system_program = next_account_info(account_info_iter)?;
        let donate_ix = solana_program::system_instruction::transfer(
            donator.key,
            vault.key,
            amount,
        );
        invoke(&donate_ix, &[donator.clone(), vault.clone(), system_program.clone()])?;

        campaign_info.amount_raised = campaign_info.amount_raised.checked_add(amount).unwrap();
        Campaign::pack(campaign_info, &mut campaign_state.try_borrow_mut_data()?)?;
        Ok(())
    }

    fn process_withdraw(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let fundstarter = next_account_info(account_info_iter)?;

        msg!("Verify signer");
        if !fundstarter.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let campaign_state = next_account_info(account_info_iter)?;
        msg!("Verify campaign_state's seeds");
        let (pda, bump) = Pubkey::find_program_address(&[b"campaign".as_ref(), fundstarter.key.as_ref()], program_id);
        if *campaign_state.key != pda {
            return Err(CampaignError::AccountMismatch.into());
        }

        let campaign_info = Campaign::unpack(&campaign_state.try_borrow_data()?)?;
        msg!("Is fundstarter the correct authority");
        if campaign_info.authority != *fundstarter.key {
            return Err(CampaignError::WrongAuthority.into());
        }
        msg!("Verifying bump");
        if campaign_info.bump != bump {
            return Err(CampaignError::AccountMismatch.into());
        }

        let vault = next_account_info(account_info_iter)?;

        msg!("Close vault and credit fundstarter");
        // Withdraw all lamports to the fundstarter and close vault account
        **fundstarter.try_borrow_mut_lamports()? = fundstarter
            .lamports()
            .checked_add(vault.lamports())
            .unwrap();
        **vault.try_borrow_mut_lamports()? = 0;

        msg!("Close campaign account and credit fundstarter");
        // Close state account
        **fundstarter.try_borrow_mut_lamports()? = fundstarter
            .lamports()
            .checked_add(campaign_state.lamports())
            .unwrap();
        **campaign_state.try_borrow_mut_lamports()? = 0;

        msg!("Zero account data");
        let mut data = campaign_state.try_borrow_mut_data()?;
        for byte in data.deref_mut().iter_mut() {
            *byte = 0;
        }
        Ok(())
    }
}