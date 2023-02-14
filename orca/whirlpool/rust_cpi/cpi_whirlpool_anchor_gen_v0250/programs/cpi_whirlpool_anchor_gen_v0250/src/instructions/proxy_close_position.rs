use anchor_lang::prelude::*;
use anchor_spl::{token::{self, Token, Mint, TokenAccount}, associated_token::{AssociatedToken}};
use whirlpools::{self, state::*};

use { 
  clockwork_sdk::{
      state::{Thread, ThreadAccount, ThreadResponse},
  }
};
#[derive(Accounts)]
pub struct ProxyClosePosition<'info> {

  #[account(address = hydra.pubkey(), signer)]
  pub hydra: Account<'info, Thread>,
  pub whirlpool_program: Program<'info, whirlpools::program::Whirlpool>,

  #[account(mut)]
  pub whirlpool: Box<Account<'info, Whirlpool>>,
  

  /// CHECK: safe (the account to receive the remaining balance of the closed account)
  #[account(mut)]
  pub receiver: UncheckedAccount<'info>,

  #[account(mut, close = receiver)]
  pub position: Box<Account<'info, Position>>,

  #[account(mut, address = position.position_mint)]
  pub position_mint: Account<'info, Mint>,

  #[account(mut,
      constraint = position_token_account.amount == 1,
      constraint = position_token_account.mint == position.position_mint)]
  pub position_token_account: Box<Account<'info, TokenAccount>>,

  #[account(address = token::ID)]
  pub token_program: Program<'info, Token>,

  /// CHECK: safe
  #[account(seeds = [b"authority"], bump)]
  pub authority: UncheckedAccount<'info>,
}

pub fn handler(
  ctx: Context<ProxyClosePosition>,
  bump: u8
) -> Result<ThreadResponse> {
  let whirlpool = &ctx.accounts.whirlpool;
  let position = &ctx.accounts.position;

  let tick_lower_index = &whirlpool.tick_current_index
      - &whirlpool.tick_current_index % whirlpool.tick_spacing as i32
      - whirlpool.tick_spacing as i32 * 2;
  let tick_upper_index = &whirlpool.tick_current_index
      - &whirlpool.tick_current_index % whirlpool.tick_spacing as i32
      + whirlpool.tick_spacing as i32 * 2;
  let tlip = position.tick_lower_index;
  let tuip = position.tick_upper_index;
  // on start we init, hab a mint. we hab other mints lined up.
    if tlip < tick_lower_index || tuip > tick_upper_index {
    let cpi_program = ctx.accounts.whirlpool_program.to_account_info();

    let cpi_accounts = whirlpools::cpi::accounts::ClosePosition {
      position_authority: ctx.accounts.authority.to_account_info(),
      receiver: ctx.accounts.receiver.to_account_info(),
      position: ctx.accounts.position.to_account_info(),
      position_mint: ctx.accounts.position_mint.to_account_info(),
      position_token_account: ctx.accounts.position_token_account.to_account_info(),
      token_program: ctx.accounts.token_program.to_account_info(),
    };

    let authority_seeds = [b"authority".as_ref(), &[bump]];
    let signer_seeds = [authority_seeds.as_ref()];
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer_seeds);

    // execute CPI
    msg!("CPI: whirlpool close_position instruction");
    whirlpools::cpi::close_position(cpi_ctx)?;
  }
   Ok(ThreadResponse {
        next_instruction: None,
        kickoff_instruction: None,
    })
}