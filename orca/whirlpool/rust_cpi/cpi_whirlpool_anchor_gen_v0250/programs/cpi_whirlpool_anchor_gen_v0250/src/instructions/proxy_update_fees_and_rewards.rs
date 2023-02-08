use anchor_lang::prelude::*;
use anchor_spl::{token::{self, Token, Mint, TokenAccount}, associated_token::{AssociatedToken}};
use whirlpools::{self, state::*};

use { 
  clockwork_sdk::{
      state::{Thread, ThreadAccount, ThreadResponse},
  }
};
#[derive(Accounts)]
pub struct ProxyUpdateFeesAndRewards<'info> {

  #[account(address = hydra.pubkey(), signer)]
  pub hydra: Account<'info, Thread>,
  pub whirlpool_program: Program<'info, whirlpools::program::Whirlpool>,

  #[account(mut)]
  pub whirlpool: Box<Account<'info, Whirlpool>>,

  #[account(mut, has_one = whirlpool)]
  pub position: Box<Account<'info, Position>>,

  #[account(mut)]
  pub tick_array_lower: UncheckedAccount<'info>,
  #[account(mut)]
  pub tick_array_upper: UncheckedAccount<'info>,
  /// CHECK: safe
  #[account(seeds = [b"authority"], bump)]
  pub authority: UncheckedAccount<'info>,
}

pub fn handler(
  ctx: Context<ProxyUpdateFeesAndRewards>,
  bump: u8
) -> Result<ThreadResponse> {
  let cpi_program = ctx.accounts.whirlpool_program.to_account_info();

  let cpi_accounts = whirlpools::cpi::accounts::UpdateFeesAndRewards {
    whirlpool: ctx.accounts.whirlpool.to_account_info(),
    position: ctx.accounts.position.to_account_info(),
    tick_array_lower: ctx.accounts.tick_array_lower.to_account_info(),
    tick_array_upper: ctx.accounts.tick_array_upper.to_account_info(),
  };

  let authority_seeds = [b"authority".as_ref(), &[bump]];
  let signer_seeds = [authority_seeds.as_ref()];
  let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer_seeds);

  // execute CPI
  msg!("CPI: whirlpool update_fees_and_rewards instruction");
  whirlpools::cpi::update_fees_and_rewards(cpi_ctx)?;

   Ok(ThreadResponse {
        next_instruction: None,
        kickoff_instruction: None,
    })
}