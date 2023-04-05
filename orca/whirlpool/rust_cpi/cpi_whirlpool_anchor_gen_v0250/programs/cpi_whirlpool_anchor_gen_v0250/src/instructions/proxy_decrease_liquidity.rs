use anchor_lang::{prelude::*, solana_program::instruction::Instruction};
use anchor_spl::{token::{self, Token, Mint, TokenAccount}, associated_token::{AssociatedToken}};
use whirlpools::{self, state::*};
use gpl_session::{SessionError, SessionToken, session_auth_or, Session};
use anchor_spl::associated_token::get_associated_token_address;

use anchor_lang::solana_program::sysvar;
use { 
  clockwork_sdk::{
      state::{Thread, ThreadAccount, ThreadResponse},
  }
};
#[derive(Accounts, Session)]
pub struct ProxyDecreaseLiquidity<'info> {
 pub owner: UncheckedAccount<'info>,
  pub signer: Signer<'info>,
  #[session(
      // The ephemeral keypair signing the transaction
      signer = signer,
      // The authority of the user account which must have created the session
      authority = owner.key()
  )]

  #[account(mut, constraint=dev.key()==Pubkey::new_from_array([
    232, 158, 159,  87,  31,  86, 208,
     28, 245, 115, 130, 214, 193, 219,
     66, 228,  51, 230, 127, 133, 163,
    242,  27,  69, 157, 185, 123, 176,
    143,  63,  68, 191
  ]))]
  /// CHECK: safe (the owner of position_token_account)
  pub dev: UncheckedAccount<'info>,
  #[session(
      // The ephemeral keypair signing the transaction
      signer = signer,
      // The authority of the user account which must have created the session
      authority = owner.key()
  )]
  // Session Tokens are passed as optional accounts
  pub session_token: Option<Account<'info, SessionToken>>,
  #[account(address = hydra.pubkey(), signer)]
  pub hydra: Account<'info, Thread>,
  pub whirlpool_program: Program<'info, whirlpools::program::Whirlpool>,

  #[account(mut)]
  pub whirlpool: Box<Account<'info, Whirlpool>>,

  #[account(address = token::ID)]
  pub token_program: Program<'info, Token>,

  

  #[account(mut, has_one = whirlpool)]
  pub position: Box<Account<'info, Position>>,
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

  #[account(mut)]
  pub tick_array_lower: UncheckedAccount<'info>,
  #[account(mut)]
  pub tick_array_upper: UncheckedAccount<'info>,
  /// CHECK: safe
  #[account(seeds = [b"authority"], bump)]
  pub authority: UncheckedAccount<'info>,

  pub system_program: Program<'info, System>,
  pub rent: Sysvar<'info, Rent>,
  pub associated_token_program: Program<'info, AssociatedToken>,

  #[account(address = sysvar::recent_blockhashes::id())]
  /// CHECK:
  recent_blockhashes: AccountInfo<'info>,
}

pub fn handler(
  ctx: Context<ProxyDecreaseLiquidity>,
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

    let cpi_accounts = whirlpools::cpi::accounts::DecreaseLiquidity {
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
    msg!("CPI: whirlpool decrease_liquidity instruction");
    whirlpools::cpi::decrease_liquidity(
      cpi_ctx,
      position.liquidity,
      0,
      0,
    )?;
  }
  let funder = ctx.accounts.signer.key();
  let whirlpool = ctx.accounts.whirlpool_program.key();

  // Create a deterministic base seed using the program's public key
  let base_seed = format!("base_seed:{}", whirlpool);

  // Derive the public key for the position_mint account
  let (position_mint_key, _) = Pubkey::find_program_address(
      &[base_seed.as_bytes(), b"position_mint"],
      &whirlpool,
  );

  // Derive the public key for the position account
  let (position_key, _) = Pubkey::find_program_address(
      &[base_seed.as_bytes(), b"position", position_mint_key.as_ref()],
      &whirlpool,
  );

  // Derive the public key for the position_token_account
  let position_token_account_key = get_associated_token_address(&funder, &position_mint_key);

let session_token_key = ctx.accounts.session_token.as_ref().map(|st| st.key());  
    // thread response with swap next_instruction
    Ok(
      ThreadResponse { 
        kickoff_instruction: None, 
        next_instruction: Some(Instruction {
            program_id: crate::ID,
            accounts: [
                crate::accounts::ProxyOpenPosition { 
                  hydra:  ctx.accounts.hydra.key(),
                    system_program: ctx.accounts.system_program.key(), 
                    token_program: ctx.accounts.token_program.key(), 
                    whirlpool: ctx.accounts.whirlpool.key(),
                    funder: ctx.accounts.hydra.key(),
                    user: ctx.accounts.owner.key(),
                    session_token: session_token_key,
                    whirlpool_program: ctx.accounts.whirlpool_program.key(),
                    dev: ctx.accounts.dev.key(),
                    owner: ctx.accounts.owner.key(),
                    associated_token_program: ctx.accounts.associated_token_program.key(),
                    rent: ctx.accounts.rent.key(),
                    recent_blockhashes: ctx.accounts.recent_blockhashes.key(),
                    position_mint: position_mint_key  ,
                    position: position_key,
                    position_token_account: position_token_account_key,
                }.to_account_metas(Some(true)),
            ].concat(),
            data: clockwork_sdk::utils::anchor_sighash("proxy_open_position").to_vec(),
        }.into())
      }
    )
}