use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("VaultBuggyExamp1e1111111111111111111111111");

/// BUG: This vault has a missing signer check on the emergency_withdraw
/// instruction, allowing any caller to drain the vault.
#[program]
pub mod vault_buggy_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.admin = ctx.accounts.admin.key();
        vault.total_deposited = 0;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        require!(amount > 0, VaultError::ZeroAmount);

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token.to_account_info(),
            to: ctx.accounts.vault_token.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        token::transfer(CpiContext::new(cpi_program, cpi_accounts), amount)?;

        let vault = &mut ctx.accounts.vault;
        vault.total_deposited += amount;

        emit!(DepositEvent {
            user: ctx.accounts.user.key(),
            amount,
        });
        Ok(())
    }

    // BUG: Missing signer check — any caller can drain the vault.
    // There is no verification that ctx.accounts.caller.key() == ctx.accounts.vault.admin.
    pub fn emergency_withdraw(ctx: Context<EmergencyWithdraw>) -> Result<()> {
        let balance = ctx.accounts.vault_token.amount;
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token.to_account_info(),
            to: ctx.accounts.destination.to_account_info(),
            authority: ctx.accounts.vault.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let seeds = &[b"vault".as_ref(), &[ctx.bumps.vault]];
        let signer_seeds = &[&seeds[..]];
        token::transfer(
            CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds),
            balance,
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + 32 + 8,
        seeds = [b"vault"],
        bump,
    )]
    pub vault: Account<'info, VaultState>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut, seeds = [b"vault"], bump)]
    pub vault: Account<'info, VaultState>,
    #[account(mut)]
    pub user_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token: Account<'info, TokenAccount>,
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

// BUG: The 'caller' field is not constrained to be the admin.
// No has_one = admin constraint, no explicit check in the handler.
#[derive(Accounts)]
pub struct EmergencyWithdraw<'info> {
    #[account(mut, seeds = [b"vault"], bump)]
    pub vault: Account<'info, VaultState>,
    #[account(mut)]
    pub vault_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub destination: Account<'info, TokenAccount>,
    pub caller: Signer<'info>, // BUG: no constraint linking caller to vault.admin
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct VaultState {
    pub admin: Pubkey,
    pub total_deposited: u64,
}

#[event]
pub struct DepositEvent {
    pub user: Pubkey,
    pub amount: u64,
}

#[error_code]
pub enum VaultError {
    #[msg("Amount must be greater than zero")]
    ZeroAmount,
}
