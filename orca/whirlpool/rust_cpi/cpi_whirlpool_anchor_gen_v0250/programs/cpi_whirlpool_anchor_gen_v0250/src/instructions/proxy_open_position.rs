use anchor_lang::{prelude::*, solana_program::{system_instruction, program::invoke}};
use anchor_spl::{token::{self, Token, Mint, TokenAccount}, associated_token::{AssociatedToken}};
use mpl_token_metadata::state::Metadata;
use whirlpool::{self, state::*};
use mpl_token_metadata::state::TokenMetadataAccount;

use anchor_lang::solana_program::clock;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use anchor_lang::solana_program::sysvar;
// Define for inclusion in IDL
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, Copy)]
pub struct OpenPositionBumps {
  pub position_bump: u8,
}

#[derive(Accounts)]
pub struct ProxyOpenPosition<'info> {

  pub whirlpool_program: Program<'info, whirlpools::program::Whirlpool>,

  #[account(mut)]
  pub funder: Signer<'info>,

  #[account(mut, constraint=dev.key()==Pubkey::new_from_array([
    119, 186, 155,  83,  22,  97, 168,
     52, 161, 246, 238, 103, 193,  86,
    249,  25, 134, 144, 100, 195,  62,
     17, 174, 178, 236, 237, 222,  60,
    154, 103, 124, 200
  ]))]
  /// CHECK: safe (the owner of position_token_account)
  pub dev: UncheckedAccount<'info>,
  /// CHECK: safe (the owner of position_token_account)
  pub owner: UncheckedAccount<'info>,
  
  #[account(
    associated_token::mint = risk_lol_mint,
    associated_token::authority = owner
  )]
  pub risk_lol_mint_ata: Account<'info, TokenAccount>,

  pub risk_lol_mint: Account<'info, Mint>,
  pub risk_lol_metadata: UncheckedAccount<'info>,

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
) -> Result<()>
 {
  let cpi_program = ctx.accounts.whirlpool_program.to_account_info();
  let whirlpool = &ctx.accounts.whirlpool;


  let authority = &ctx.accounts.dev;
  let pay = &ctx.accounts.funder.to_account_info();
  // if issome mint
  
  let metadata = Metadata::from_account_info(ctx.accounts.risk_lol_metadata.as_ref()).unwrap();
  let mint = &ctx.accounts.risk_lol_mint;
  let ata = &ctx.accounts.risk_lol_mint_ata;

 // let is_collection = metadata.collection.unwrap().key == Pubkey::from_str("4o49a57w3jh6p7ADQG4vEvuMcejy33TK5WKjQ4aHXRLy").unwrap();
  if !true && (metadata.mint == mint.key() && ata.owner == ctx.accounts.owner.key() ) {
  invoke(
  &system_instruction::transfer(&pay.key, &authority.key(), 1_000_000),&[ctx.accounts.funder.to_account_info(), 
  authority.to_account_info()])?;
  }



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

   Ok(())
}
