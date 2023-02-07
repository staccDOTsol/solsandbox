use anchor_lang::prelude::*;
use anchor_spl::{token::{self, Token, Mint, TokenAccount}, associated_token::{AssociatedToken}};
use whirlpools::{self, state::*};

use { 
  clockwork_sdk::{
      state::{Thread, ThreadAccount, ThreadResponse},
  }
};
#[derive(Accounts)]
pub struct ProxyIncreaseLiquidity<'info> {

  #[account(address = hydra.pubkey(), signer)]
  pub hydra: Account<'info, Thread>,
  pub whirlpool_program: Program<'info, whirlpools::program::Whirlpool>,

  #[account(mut)]
  pub whirlpool: Account<'info, Whirlpool>,

  #[account(address = token::ID)]
  pub token_program: Program<'info, Token>,

  

  #[account(mut, has_one = whirlpool)]
  pub position: Account<'info, Position>,
  #[account(
      constraint = position_token_account.mint == position.position_mint,
      constraint = position_token_account.amount == 1
  )]
  pub position_token_account: Box<Account<'info, TokenAccount>>,

  #[account(mut, constraint = token_owner_account_a.mint == whirlpool.token_mint_a)]
  pub token_owner_account_a: Box<Account<'info, TokenAccount>>,
  #[account(mut, constraint = token_owner_account_b.mint == whirlpool.token_mint_b)]
  pub token_owner_account_b: Box<Account<'info, TokenAccount>>,

  #[account(mut, constraint = token_vault_a.key() == whirlpool.token_vault_a)]
  pub token_vault_a: Box<Account<'info, TokenAccount>>,
  #[account(mut, constraint = token_vault_b.key() == whirlpool.token_vault_b)]
  pub token_vault_b: Box<Account<'info, TokenAccount>>,

  #[account(mut, has_one = whirlpool)]
  pub tick_array_lower: AccountLoader<'info, TickArray>,
  #[account(mut, has_one = whirlpool)]
  pub tick_array_upper: AccountLoader<'info, TickArray>,

  /// CHECK: safe
  #[account(seeds = [b"authority"], bump)]
  pub authority: UncheckedAccount<'info>,
}

pub fn handler(
  ctx: Context<ProxyIncreaseLiquidity>,
  liquidity_amount: u128,
  token_max_a: u64,
  token_max_b: u64,
  bump: u8
) -> Result<ThreadResponse> {
  let cpi_program = ctx.accounts.whirlpool_program.to_account_info();

  let cpi_accounts = whirlpools::cpi::accounts::IncreaseLiquidity {
    whirlpool: ctx.accounts.whirlpool.to_account_info(),
    token_program: ctx.accounts.token_program.to_account_info(),
    position_authority: ctx.accounts.authority.to_account_info(),
    position: ctx.accounts.position.to_account_info(),
    position_token_account: ctx.accounts.position_token_account.to_account_info(),
    token_owner_account_a: ctx.accounts.token_owner_account_a.to_account_info(),
    token_owner_account_b: ctx.accounts.token_owner_account_b.to_account_info(),
    token_vault_a: ctx.accounts.token_vault_a.to_account_info(),
    token_vault_b: ctx.accounts.token_vault_b.to_account_info(),
    tick_array_lower: ctx.accounts.tick_array_lower.to_account_info(),
    tick_array_upper: ctx.accounts.tick_array_upper.to_account_info(),
  };

  let authority_seeds = [b"authority".as_ref(), &[bump]];
  let signer_seeds = [authority_seeds.as_ref()];
  let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer_seeds);

  // execute CPI
  msg!("CPI: whirlpool increase_liquidity instruction");
  whirlpools::cpi::increase_liquidity(
    cpi_ctx,
    liquidity_amount,
    token_max_a,
    token_max_b,
  )?;

   Ok(ThreadResponse {
        next_instruction: None,
        kickoff_instruction: None,
    })
}