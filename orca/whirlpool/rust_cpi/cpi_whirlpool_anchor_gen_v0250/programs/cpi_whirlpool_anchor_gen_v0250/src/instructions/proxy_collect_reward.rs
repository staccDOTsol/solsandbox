use anchor_lang::prelude::*;
use anchor_spl::{token::{self, Token, Mint, TokenAccount}, associated_token::{AssociatedToken}};
use whirlpool::{self, state::*};

use { 
  clockwork_sdk::{
      state::{Thread, ThreadAccount, ThreadResponse},
  }
};
#[derive(Accounts)]
#[instruction(reward_index: u8)]
pub struct ProxyCollectReward<'info> {

  #[account(address = hydra.pubkey(), signer)]
  pub hydra: Account<'info, Thread>,
  pub whirlpool_program: Program<'info, whirlpools::program::Whirlpool>,

  pub whirlpool: Box<Account<'info, Whirlpool>>,

  

  #[account(mut, has_one = whirlpool)]
  pub position: Box<Account<'info, Position>>,
  #[account(
      constraint = position_token_account.mint == position.position_mint,
      constraint = position_token_account.amount == 1
  )]
  pub position_token_account: Box<Account<'info, TokenAccount>>,

  #[account(mut,
      constraint = reward_owner_account.mint == whirlpool.reward_infos[reward_index as usize].mint
  )]
  pub reward_owner_account: Box<Account<'info, TokenAccount>>,

  #[account(mut, address = whirlpool.reward_infos[reward_index as usize].vault)]
  pub reward_vault: Box<Account<'info, TokenAccount>>,

  #[account(address = token::ID)]
  pub token_program: Program<'info, Token>,

  /// CHECK: safe
  #[account(seeds = [b"authority"], bump)]
  pub authority: UncheckedAccount<'info>,
}

pub fn handler(
  ctx: Context<ProxyCollectReward>,
  reward_index: u8,
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

    let cpi_accounts = whirlpools::cpi::accounts::CollectReward {
      whirlpool: ctx.accounts.whirlpool.to_account_info(),
      position_authority: ctx.accounts.authority.to_account_info(),
      position: ctx.accounts.position.to_account_info(),
      position_token_account: ctx.accounts.position_token_account.to_account_info(),
      reward_owner_account: ctx.accounts.reward_owner_account.to_account_info(),
      reward_vault: ctx.accounts.reward_vault.to_account_info(),
      token_program: ctx.accounts.token_program.to_account_info(),
    };

    let authority_seeds = [b"authority".as_ref(), &[bump]];
    let signer_seeds = [authority_seeds.as_ref()];
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer_seeds);

    // execute CPI
    msg!("CPI: whirlpool collect_reward instruction");
    whirlpools::cpi::collect_reward(cpi_ctx, reward_index)?;
  }
   Ok(ThreadResponse {
        next_instruction: None,
        kickoff_instruction: None,
    })
}