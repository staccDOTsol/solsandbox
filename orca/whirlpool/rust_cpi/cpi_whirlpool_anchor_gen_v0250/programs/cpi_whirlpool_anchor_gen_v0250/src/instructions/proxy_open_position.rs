use anchor_lang::{prelude::*, solana_program::{system_instruction, program::invoke}};
use anchor_spl::{token::{self, Token}, associated_token::{AssociatedToken}};
use whirlpools::{self, state::*};
use gpl_session::{SessionError, SessionToken, session_auth_or, Session};
use anchor_lang::solana_program::clock;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use anchor_lang::solana_program::sysvar;
// Define for inclusion in IDL
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, Copy)]
pub struct OpenPositionBumps {
  pub position_bump: u8,
}

use { 
  clockwork_sdk::{
      state::{Thread, ThreadAccount, ThreadResponse},
  }
};
#[derive(Accounts, Session)]
pub struct ProxyOpenPosition<'info> {


  #[account(address = hydra.pubkey(), signer)]
  pub hydra: Account<'info, Thread>,
  pub whirlpool_program: Program<'info, whirlpools::program::Whirlpool>,

  #[account(mut)]
  pub funder: Signer<'info>,

  pub user: UncheckedAccount<'info>,
  #[session(
      // The ephemeral keypair signing the transaction
      signer = funder,
      // The authority of the user account which must have created the session
      authority = user.key()
  )]
  // Session Tokens are passed as optional accounts
  pub session_token: Option<Account<'info, SessionToken>>,

  #[account(mut, constraint=dev.key()==Pubkey::new_from_array([
    232, 158, 159,  87,  31,  86, 208,
     28, 245, 115, 130, 214, 193, 219,
     66, 228,  51, 230, 127, 133, 163,
    242,  27,  69, 157, 185, 123, 176,
    143,  63,  68, 191
  ]))]
  /// CHECK: safe (the owner of position_token_account)
  pub dev: UncheckedAccount<'info>,
  /// CHECK: safe (the owner of position_token_account)
  pub owner: UncheckedAccount<'info>,

  /// CHECK: init by whirlpool
  #[account(mut)]
  pub position: UncheckedAccount<'info>,

  /// CHECK: init by whirlpool
  #[account(mut)]
  pub position_mint: Signer<'info>,

  /// CHECK: init by whirlpool
  #[account(mut)]
  pub position_token_account: UncheckedAccount<'info>,

  pub whirlpool: Box<Account<'info, Whirlpool>>,

  #[account(address = token::ID)]
  pub token_program: Program<'info, Token>,
  pub system_program: Program<'info, System>,
  pub rent: Sysvar<'info, Rent>,
  pub associated_token_program: Program<'info, AssociatedToken>,

  #[account(address = sysvar::recent_blockhashes::id())]
  /// CHECK:
  recent_blockhashes: AccountInfo<'info>,
}

use anchor_lang::error;
use arrayref::array_ref;

pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[derive(Hash)]
pub struct HashOfHash {
    pub(crate) recent_blockhash: u64,
    pub(crate) user: [u8; 32],
    pub(crate) clock: [u8; 1]
}

pub fn handler(
  ctx: Context<ProxyOpenPosition>,
  bumps: OpenPositionBumps,
)-> Result<ThreadResponse> 
 {
  let cpi_program = ctx.accounts.whirlpool_program.to_account_info();
  let whirlpool = &ctx.accounts.whirlpool;


  let authority = &ctx.accounts.dev;
  let pay = &ctx.accounts.funder.to_account_info();
  invoke(
  &system_instruction::transfer(&pay.key, &authority.key(), 1_000_000),&[ctx.accounts.funder.to_account_info(), 
  authority.to_account_info()])?;

  let recent_blockhashes = &ctx.accounts.recent_blockhashes;
  let data = recent_blockhashes.data.borrow();
  let most_recent = array_ref![data, 8, 8];
  let user_head = &ctx.accounts.funder.key();


  let index3 = u64::from_le_bytes(*most_recent);
  let clock = clock::Clock::get().unwrap().unix_timestamp as u8;
  let clock_arr: [u8; 1] = [clock];
  let index = calculate_hash(&HashOfHash {
      recent_blockhash: index3,
      user: user_head.to_bytes(),
      clock: clock_arr
  });
  msg!(&index.to_string());
  let last = index.to_string().chars().nth(0).unwrap();

  let tick_lower_index = &whirlpool.tick_current_index
      - &whirlpool.tick_current_index % whirlpool.tick_spacing as i32
      - whirlpool.tick_spacing as i32 * (last as i32 % 4);
  let tick_upper_index = &whirlpool.tick_current_index
      - &whirlpool.tick_current_index % whirlpool.tick_spacing as i32
      + whirlpool.tick_spacing as i32 *  (last as i32 % 4);
     
  let cpi_accounts = whirlpools::cpi::accounts::OpenPosition {
    funder: ctx.accounts.funder.to_account_info(),
    owner: ctx.accounts.owner.to_account_info(),
    position: ctx.accounts.position.to_account_info(),
    position_mint: ctx.accounts.position_mint.to_account_info(),
    position_token_account: ctx.accounts.position_token_account.to_account_info(),
    whirlpool: ctx.accounts.whirlpool.to_account_info(),
    token_program: ctx.accounts.token_program.to_account_info(),
    system_program: ctx.accounts.system_program.to_account_info(),
    rent: ctx.accounts.rent.to_account_info(),
    associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
  };

  let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

  // execute CPI
  msg!("CPI: whirlpool open_position instruction");
  whirlpools::cpi::open_position(
    cpi_ctx,
    whirlpools::typedefs::OpenPositionBumps { position_bump: bumps.position_bump },
    tick_lower_index,
    tick_upper_index,
  )?;

  Ok(ThreadResponse {
    next_instruction: None,
    kickoff_instruction: None,
})
}