use anchor_lang::prelude::*;

declare_id!("9dyeUpFkXJXceLChsrFrgoprCSW4xHCgpQevnbqKgoUB");

pub mod instructions;
use instructions::*;

use { 
  clockwork_sdk::{
      state::{ThreadResponse},
  }
};
#[program]
pub mod cpi_whirlpool_anchor_gen_v0250 {
  use super::*;

  pub fn verify_whirlpools_config_account(
    ctx: Context<VerifyWhirlpoolsConfigAccount>
  ) -> Result<()>{
    return instructions::verify_account::handler_whirlpools_config(ctx);
  }

  pub fn verify_feetier_account(
    ctx: Context<VerifyFeeTierAccount>,
  ) -> Result<()>{
    return instructions::verify_account::handler_feetier(ctx);
  }

  pub fn verify_whirlpool_account(
    ctx: Context<VerifyWhirlpoolAccount>,
  ) -> Result<()>{
    return instructions::verify_account::handler_whirlpool(ctx);
  }

  pub fn verify_tickarray_account(
    ctx: Context<VerifyTickArrayAccount>,
    sampling1: u32,
    sampling2: u32,
    sampling3: u32,
    sampling4: u32,
    sampling5: u32,
    sampling6: u32,
    sampling7: u32,
    sampling8: u32,
  ) -> Result<()>{
    return instructions::verify_account::handler_tickarray(
      ctx,
      sampling1, sampling2, sampling3, sampling4,
      sampling5, sampling6, sampling7, sampling8,
    );
  }

  pub fn verify_position_account(
    ctx: Context<VerifyPositionAccount>,
  ) -> Result<()>{
    return instructions::verify_account::handler_position(ctx);
  }

  pub fn proxy_swap(
    ctx: Context<ProxySwap>,
    amount: u64,
    other_amount_threshold: u64,
    sqrt_price_limit: u128,
    amount_specified_is_input: bool,
    a_to_b: bool, bump: u8
  ) -> Result<ThreadResponse> {
    return instructions::proxy_swap::handler(
      ctx,
      amount,
      other_amount_threshold,
      sqrt_price_limit,
      amount_specified_is_input,
      a_to_b,
      bump
    );
  }

  pub fn proxy_open_position(
    ctx: Context<ProxyOpenPosition>,
    bumps: OpenPositionBumps,
  ) -> Result<()> {
    return instructions::proxy_open_position::handler(
      ctx,
      bumps,
    );
  }

  pub fn proxy_increase_liquidity(
    ctx: Context<ProxyIncreaseLiquidity>,
    liquidity: u128,
    token_max_a: u64,
    token_max_b: u64, bump: u8
  ) -> Result<ThreadResponse> {
    return instructions::proxy_increase_liquidity::handler(
      ctx,
      liquidity,
      token_max_a,
      token_max_b,
      bump
    );
  }

  pub fn proxy_decrease_liquidity(
    ctx: Context<ProxyDecreaseLiquidity>,bump: u8
  ) -> Result<ThreadResponse> {
    return instructions::proxy_decrease_liquidity::handler(
      ctx,
      bump 
    );
  }

  pub fn proxy_update_fees_and_rewards(
    ctx: Context<ProxyUpdateFeesAndRewards>, bump: u8
  ) -> Result<ThreadResponse> {
    return instructions::proxy_update_fees_and_rewards::handler(
      ctx,
      bump 
    );
  }

  pub fn proxy_collect_fees(
    ctx: Context<ProxyCollectFees>, bump: u8
  ) -> Result<ThreadResponse> {
    return instructions::proxy_collect_fees::handler(
      ctx,
      bump 
    );
  }

  pub fn proxy_collect_reward(
    ctx: Context<ProxyCollectReward>,
    reward_index: u8, bump: u8
  ) -> Result<ThreadResponse> {
    return instructions::proxy_collect_reward::handler(
      ctx,
      reward_index,
      bump 
    );
  }

  pub fn proxy_close_position(
    ctx: Context<ProxyClosePosition>, bump: u8
  ) -> Result<ThreadResponse> {
    return instructions::proxy_close_position::handler(
      ctx,
      bump 
    );
  }

}
