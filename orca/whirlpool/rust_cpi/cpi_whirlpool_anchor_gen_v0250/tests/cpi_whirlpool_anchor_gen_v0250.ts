import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { CpiWhirlpoolAnchorGenV0250 } from "../target/types/cpi_whirlpool_anchor_gen_v0250";
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import {
  ORCA_WHIRLPOOL_PROGRAM_ID, ORCA_WHIRLPOOLS_CONFIG,
  PDAUtil, PriceMath, TickUtil, AccountFetcher, SwapUtils,
  swapQuoteByInputToken, WhirlpoolContext, buildWhirlpoolClient,
  increaseLiquidityQuoteByInputToken, decreaseLiquidityQuoteByLiquidity,
  collectFeesQuote, collectRewardsQuote, TickArrayUtil, PoolUtil,
} from "@orca-so/whirlpools-sdk";
import { TOKEN_PROGRAM_ID, AccountLayout, ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { TransactionBuilder, resolveOrCreateATA, deriveATA, DecimalUtil, Percentage } from "@orca-so/common-sdk";
import { assert, expect } from "chai";
import {
  ThreadProgram as ThreadProgramType,
  IDL as ThreadProgramIdl_v1_3_15, 
} from './thread_program';
export const CLOCKWORK_THREAD_PROGRAM_ID = new PublicKey(
  '3XXuUFfweXBwFgFfYaejLvZE4cGZiHgKiGfMtdxNzYmv',
);
const SOL = {mint: new PublicKey("So11111111111111111111111111111111111111112"), decimals: 9};
const ORCA = {mint: new PublicKey("orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE"), decimals: 6};
const BONK = {mint: new PublicKey("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"), decimals: 5};
const USDC = {mint: new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), decimals: 6};

describe("cpi_whirlpool_anchor_gen_v0250",() => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const connection = anchor.getProvider().connection;
  const program = anchor.workspace.CpiWhirlpoolAnchorGenV0250 as Program<CpiWhirlpoolAnchorGenV0250>;
  const [authority, bump] = PublicKey.findProgramAddressSync([Buffer.from("authority")], program.programId)
  const SEED_QUEUE = 'thread';
  
  const provider = anchor.getProvider();
  const wallet = anchor.AnchorProvider.env().wallet;
  const fetcher = new AccountFetcher(connection);
  const whirlpool_ctx = WhirlpoolContext.from(connection, wallet, ORCA_WHIRLPOOL_PROGRAM_ID, fetcher);
  const whirlpool_client = buildWhirlpoolClient(whirlpool_ctx);

  const sol_usdc_whirlpool_pubkey = PDAUtil.getWhirlpool(ORCA_WHIRLPOOL_PROGRAM_ID, ORCA_WHIRLPOOLS_CONFIG, SOL.mint, USDC.mint, 64).publicKey;
 
  


  const verify_log = (logs: string[], message: string) => { expect(logs).includes(`Program log: verify! ${message}`); };
  const rent_ta = async () => { return connection.getMinimumBalanceForRentExemption(AccountLayout.span) }
  const sleep = (second) => new Promise(resolve => setTimeout(resolve, second * 1000));

  const pools =[
    "3ne4mWqdYuNiYrYZC9TrA3FcfuFdErghH97vNPbjicr1",
    "ABwq1ro51rdcuknyTs1M5oYqmLxY4E9eaBy19QGvpXym",
    "8QaXeHBrShJTdtN1rWCccBxpSVvKksQ2PCu5nufb2zbk",
    "5P6n5omLbLbP4kaPGL8etqQAHEx2UCkaUyvjLDnwV4EY",
    "BqnpCdDLPV2pFdAaLnVidmn3G93RP2p5oRdGEY2sJGez",
    "GMx3g7HZCVzLuE6Ckdu9sgExgiZQ7kiF7xfCrPnwVzbr",
    "6MSGDnvqB7qnTjXerKFyJwKUPWjNfCc9kyThR5sL8GWZ",
    "J7n8SGZwNqxcQkyABYpM9cVYuRPHTikJmtxCgHuMqDaq",
    "CffWKMFRx2WfrzBpRZBjxAjaLp3R85Uv3ZxYkch4qQUW",
    "D3pqgEGLqAfHGYfP7LTXrNEXqiqDQbrnpaz2AXiahrQi",
    "6u3MNQnnaRsc3rJ4fiKsQ2SZZZmNenhEigJDGRFUTQYS",
    "7FPCHo6cXYxggtMApjqevfYnf6qxhFwnK3qgxh2SGa4C",
    "AKN193cDKKFBVHt1YP3YebLGAYu2e8qW8VdjigMBc5pL",
    "5Rw3aRurTCcKBUbZ8KTRBJiQ3xR8zrd6bhnmAuPwdJ5m",
    "2C8wP2NoDWvLFY74rT3Q5e9zbbWajMHQEmYt6ryGZJsg",
    "6yzjKJRmHtUqXuq33bZqTLZEZEXqhf9YEXVzhB3e5svf",
    "DN5DD7nijNrrjHrjri4VcS9yREaZxEu34zowFuG6hLy6",
    "746PKfNzBUUM8TixzMsY5RxmqbL7XsiM8M1NGjnrhXcC",
    "5D3688sA3FyM14Zky53YnLq9jaGa7atdrCUvQhd3jRBN",
    "HgqiY7bP7k8jZ36RMg48y2pbTRkxVFkusRBPou8CsZru",
    "5ykXFtAygLxGjEmphrMgP5DASTgbbPQeRQ7q7fJuwQqj",
    "4vZs49zRir2WDUYBMmkBffVkW9tRV7ALGjUsJHv946kz",
    "H1zqtQiVopKtcMqbXfPZyfqiGNWKqACVVKGW6gxz7MTs",
    "CcdR5svqhW3NKH5tJsDZUh99DbEuTtYepqMufhdwuzTm",
    "2kechVE4thKxnvtG8E64X4jt8c2Kf3XuTfnQAc3BN1Ad",
    "5nwEbj7dHntAnvDa7yYo7rmTss2BgLxMXc4dQkVJaVWP",
    "EGu5QzaGzmz3CZXFMVNmiqtaskVpd7QMZ2zDhP9wLS26",
    "DMj5Db34TgGfrdMLn7R2dGxR4G9y4d66Le8HsDtZxTva",
  ]
  it("execute proxy open_position", async () => {
    let bonkBal = (await connection.getTokenAccountBalance(new PublicKey("3a6vmVLpwXueJn68LWxtjbwhGaEWGJm4h34KgzXUmyyR"))).value.amount   
    
    
    for (var pool of pools){
      let tx = new TransactionBuilder(connection,wallet) 
      
      const samo_usdc_whirlpool_pubkey = new PublicKey(pool)

      const position_mint_keypair = Keypair.generate();
      const position_mint = position_mint_keypair.publicKey;
      const position_pda = PDAUtil.getPosition(ORCA_WHIRLPOOL_PROGRAM_ID, position_mint);
      
    const position_ta = await deriveATA(wallet.publicKey, position_mint);

    const bumps = { positionBump: position_pda.bump };
    const tick_lower_index = PriceMath.priceToInitializableTickIndex(DecimalUtil.fromNumber(0.01), BONK.decimals, USDC.decimals, 64);
    const tick_upper_index = PriceMath.priceToInitializableTickIndex(DecimalUtil.fromNumber(0.02), BONK.decimals, USDC.decimals, 64);
    var threadName = (Math.floor(Math.random()*99999)).toString()
    var [hydra] = PublicKey.findProgramAddressSync(
      [Buffer.from(SEED_QUEUE, 'utf-8'), wallet.publicKey.toBuffer(), Buffer.from(threadName, 'utf-8')],
      CLOCKWORK_THREAD_PROGRAM_ID,
    );
    console.log(hydra.toBase58())
   
    tx.addInstruction({instructions:[SystemProgram.transfer({
      /** Account that will transfer lamports */
      fromPubkey: wallet.publicKey,
      /** Account that will receive transferred lamports */
      toPubkey: hydra,
      /** Amount of lamports to transfer */
      lamports: 0.00666 * 10 ** 9
    })], signers:[], cleanupInstructions:[]})
    
    var ix = await program.methods
      .proxyOpenPosition(
        bumps,
      )
      .accounts({
        hydra,
        whirlpoolProgram: ORCA_WHIRLPOOL_PROGRAM_ID,
        funder: wallet.publicKey,
        owner: wallet.publicKey,
        position: position_pda.publicKey,
        positionMint: position_mint,
        positionTokenAccount: position_ta,
        whirlpool: samo_usdc_whirlpool_pubkey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .instruction();
      const threadProgram = await new anchor.Program(
        ThreadProgramIdl_v1_3_15,
        CLOCKWORK_THREAD_PROGRAM_ID,
        provider,
      )
      var magic = await threadProgram.methods
      .threadCreate(
        threadName,
        {
          accounts: ix.keys,
          programId: new PublicKey(ix.programId),
          data: ix.data,
        },
        {
          cron: {schedule: "5 * * * * * *"}
        },
      )
      .accounts({
        authority: wallet.publicKey,
        payer: wallet.publicKey,
        thread: hydra,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    await connection.confirmTransaction(magic);

    const position_data = await fetcher.getPosition(position_pda.publicKey, true);
    const samo_usdc_whirlpool = await whirlpool_client.getPool(samo_usdc_whirlpool_pubkey, true);
   const wagering = parseFloat(bonkBal) / pools.length
    var quote = increaseLiquidityQuoteByInputToken(
      BONK.mint,
      DecimalUtil.fromNumber(Math.floor(wagering)),
      position_data.tickLowerIndex,
      position_data.tickUpperIndex,
      Percentage.fromFraction(0, 1000),
      samo_usdc_whirlpool,
    );
    var threadName = (Math.floor(Math.random()*99999)).toString()
    var [hydra] = PublicKey.findProgramAddressSync(
      [Buffer.from(SEED_QUEUE, 'utf-8'), wallet.publicKey.toBuffer(), Buffer.from(threadName, 'utf-8')],
      CLOCKWORK_THREAD_PROGRAM_ID,
    );
    console.log(hydra.toBase58())
      tx.addInstruction({instructions:[SystemProgram.transfer({
      /** Account that will transfer lamports */
      fromPubkey: wallet.publicKey,
      /** Account that will receive transferred lamports */
      toPubkey: hydra,
      /** Amount of lamports to transfer */
      lamports: 0.00666 * 10 ** 9
    })], signers:[], cleanupInstructions:[]})
    
    var ix = await program.methods
      .proxyIncreaseLiquidity(
        quote.liquidityAmount,
        quote.tokenMaxA,
        quote.tokenMaxB,
        bump 
      )
      .accounts({
        hydra,
        whirlpoolProgram: ORCA_WHIRLPOOL_PROGRAM_ID,
        whirlpool: samo_usdc_whirlpool_pubkey,
        tokenProgram: TOKEN_PROGRAM_ID,
        position: position_pda.publicKey,
        positionTokenAccount: await deriveATA(wallet.publicKey, position_mint),
        tokenOwnerAccountA: await deriveATA(wallet.publicKey, BONK.mint),
        tokenOwnerAccountB: await deriveATA(wallet.publicKey, USDC.mint),
        tokenVaultA: samo_usdc_whirlpool.getData().tokenVaultA,
        tokenVaultB: samo_usdc_whirlpool.getData().tokenVaultB,
        tickArrayLower: PDAUtil.getTickArrayFromTickIndex(position_data.tickLowerIndex, 64, samo_usdc_whirlpool_pubkey, ORCA_WHIRLPOOL_PROGRAM_ID).publicKey,
        tickArrayUpper: PDAUtil.getTickArrayFromTickIndex(position_data.tickUpperIndex, 64, samo_usdc_whirlpool_pubkey, ORCA_WHIRLPOOL_PROGRAM_ID).publicKey,
        authority
      })
      .instruction();
     
      var magic = await threadProgram.methods
      .threadCreate(
        threadName,
        {
          accounts: ix.keys,
          programId: new PublicKey(ix.programId),
          data: ix.data,
        },
        {
          cron: {schedule: "5 * * * * * *"}
        },
      )
      .accounts({
        authority: wallet.publicKey,
        payer: wallet.publicKey,
        thread: hydra,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    await connection.confirmTransaction(magic);

    const post_position_data = await fetcher.getPosition(position_pda.publicKey, true);
    const delta_liquidity = post_position_data.liquidity.sub(position_data.liquidity);
   
    const pre_last_updated = (await samo_usdc_whirlpool.refreshData()).rewardLastUpdatedTimestamp;
    var threadName = (Math.floor(Math.random()*99999)).toString()
    var [hydra] = PublicKey.findProgramAddressSync(
      [Buffer.from(SEED_QUEUE, 'utf-8'), wallet.publicKey.toBuffer(), Buffer.from(threadName, 'utf-8')],
      CLOCKWORK_THREAD_PROGRAM_ID,
    );
    console.log(hydra.toBase58())
   tx.addInstruction({instructions:[SystemProgram.transfer({
      /** Account that will transfer lamports */
      fromPubkey: wallet.publicKey,
      /** Account that will receive transferred lamports */
      toPubkey: hydra,
      /** Amount of lamports to transfer */
      lamports: 0.00666 * 10 ** 9
    })], signers:[], cleanupInstructions:[]})
    
    var ix = await program.methods
      .proxyUpdateFeesAndRewards(bump)
      .accounts({
        hydra,
        whirlpoolProgram: ORCA_WHIRLPOOL_PROGRAM_ID,
        whirlpool: samo_usdc_whirlpool_pubkey,
        position: position_pda.publicKey,
        tickArrayLower: PDAUtil.getTickArrayFromTickIndex(position_data.tickLowerIndex, 64, samo_usdc_whirlpool_pubkey, ORCA_WHIRLPOOL_PROGRAM_ID).publicKey,
        tickArrayUpper: PDAUtil.getTickArrayFromTickIndex(position_data.tickUpperIndex, 64, samo_usdc_whirlpool_pubkey, ORCA_WHIRLPOOL_PROGRAM_ID).publicKey,
        authority
      })
      .instruction();
     
      var magic = await threadProgram.methods
      .threadCreate(
        threadName,
        {
          accounts: ix.keys,
          programId: new PublicKey(ix.programId),
          data: ix.data,
        },
        {
          cron: {schedule: "5 * * * * * *"}
        },
      )
      .accounts({
        authority: wallet.publicKey,
        payer: wallet.publicKey,
        thread: hydra,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    await connection.confirmTransaction(magic);

    const post_last_updated = (await samo_usdc_whirlpool.refreshData()).rewardLastUpdatedTimestamp;
  // @ts-ignore
    var quote = await decreaseLiquidityQuoteByLiquidity(
      position_data.liquidity,
      Percentage.fromFraction(0, 1000),
      await whirlpool_client.getPosition(position_pda.publicKey),
      samo_usdc_whirlpool,
    );
    var threadName = (Math.floor(Math.random()*99999)).toString()
    var [hydra] = PublicKey.findProgramAddressSync(
      [Buffer.from(SEED_QUEUE, 'utf-8'), wallet.publicKey.toBuffer(), Buffer.from(threadName, 'utf-8')],
      CLOCKWORK_THREAD_PROGRAM_ID,
    );
    console.log(hydra.toBase58())
    tx.addInstruction({instructions:[SystemProgram.transfer({
      /** Account that will transfer lamports */
      fromPubkey: wallet.publicKey,
      /** Account that will receive transferred lamports */
      toPubkey: hydra,
      /** Amount of lamports to transfer */
      lamports: 0.00666 * 10 ** 9
    })], signers:[], cleanupInstructions:[]})
    
    var ix = await program.methods
      .proxyDecreaseLiquidity(
        bump 
      )
      .accounts({
        hydra,
        whirlpoolProgram: ORCA_WHIRLPOOL_PROGRAM_ID,
        whirlpool: samo_usdc_whirlpool_pubkey,
        tokenProgram: TOKEN_PROGRAM_ID,
        position: position_pda.publicKey,
        positionTokenAccount: await deriveATA(wallet.publicKey, position_mint),
        tokenOwnerAccountA: await deriveATA(wallet.publicKey, BONK.mint),
        tokenOwnerAccountB: await deriveATA(wallet.publicKey, USDC.mint),
        tokenVaultA: samo_usdc_whirlpool.getData().tokenVaultA,
        tokenVaultB: samo_usdc_whirlpool.getData().tokenVaultB,
        tickArrayLower: PDAUtil.getTickArrayFromTickIndex(position_data.tickLowerIndex, 64, samo_usdc_whirlpool_pubkey, ORCA_WHIRLPOOL_PROGRAM_ID).publicKey,
        tickArrayUpper: PDAUtil.getTickArrayFromTickIndex(position_data.tickUpperIndex, 64, samo_usdc_whirlpool_pubkey, ORCA_WHIRLPOOL_PROGRAM_ID).publicKey,
        authority
      })
      .instruction();
      
      var magic = await threadProgram.methods
      .threadCreate(
        threadName,
        {
          accounts: ix.keys,
          programId: new PublicKey(ix.programId),
          data: ix.data,
        },
        {
          cron: {schedule: "5 * * * * * *"}
        },
      )
      .accounts({
        authority: wallet.publicKey,
        payer: wallet.publicKey,
        thread: hydra,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    await connection.confirmTransaction(magic);

    //console.log("fee", position_data.feeOwedA.toString(), position_data.feeOwedB.toString());
    var threadName = (Math.floor(Math.random()*99999)).toString()
    var [hydra] = PublicKey.findProgramAddressSync(
      [Buffer.from(SEED_QUEUE, 'utf-8'), wallet.publicKey.toBuffer(), Buffer.from(threadName, 'utf-8')],
      CLOCKWORK_THREAD_PROGRAM_ID,
    );
    console.log(hydra.toBase58())
    tx.addInstruction({instructions:[SystemProgram.transfer({
      /** Account that will transfer lamports */
      fromPubkey: wallet.publicKey,
      /** Account that will receive transferred lamports */
      toPubkey: hydra,
      /** Amount of lamports to transfer */
      lamports: 0.00666 * 10 ** 9
    })], signers:[], cleanupInstructions:[]})
    
    var ix = await program.methods
      .proxyCollectFees(bump)
      .accounts({
        hydra,
        whirlpoolProgram: ORCA_WHIRLPOOL_PROGRAM_ID,
        whirlpool: samo_usdc_whirlpool_pubkey,
        position: position_pda.publicKey,
        positionTokenAccount: await deriveATA(wallet.publicKey, position_mint),
        tokenOwnerAccountA: await deriveATA(wallet.publicKey, BONK.mint),
        tokenVaultA: samo_usdc_whirlpool.getData().tokenVaultA,
        tokenOwnerAccountB: await deriveATA(wallet.publicKey, USDC.mint),
        tokenVaultB: samo_usdc_whirlpool.getData().tokenVaultB,
        tokenProgram: TOKEN_PROGRAM_ID,
        authority
      })
      .instruction();
      
      var magic = await threadProgram.methods
      .threadCreate(
        threadName,
        {
          accounts: ix.keys,
          programId: new PublicKey(ix.programId),
          data: ix.data,
        },
        {
          cron: {schedule: "5 * * * * * *"}
        },
      )
      .accounts({
        authority: wallet.publicKey,
        payer: wallet.publicKey,
        thread: hydra,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    await connection.confirmTransaction(magic);

   const samo_usdc_whirlpool_data = samo_usdc_whirlpool.getData();

    for (let reward_index=0; reward_index<3; reward_index++) {
      const reward_info = samo_usdc_whirlpool_data.rewardInfos[reward_index];
      if ( !PoolUtil.isRewardInitialized(reward_info) ) {
        break;
      }

      const reward_ta = await resolveOrCreateATA(connection, wallet.publicKey, reward_info.mint, rent_ta);

      //console.log("reward", position_data.rewardInfos[reward_index].amountOwed.toString());
      var threadName = (Math.floor(Math.random()*99999)).toString()
      var [hydra] = PublicKey.findProgramAddressSync(
        [Buffer.from(SEED_QUEUE, 'utf-8'), wallet.publicKey.toBuffer(), Buffer.from(threadName, 'utf-8')],
        CLOCKWORK_THREAD_PROGRAM_ID,
      );
      console.log(hydra.toBase58())
      tx.addInstruction({instructions:[SystemProgram.transfer({
        /** Account that will transfer lamports */
        fromPubkey: wallet.publicKey,
        /** Account that will receive transferred lamports */
        toPubkey: hydra,
        /** Amount of lamports to transfer */
        lamports: 0.00666 * 10 ** 9
      })], signers:[], cleanupInstructions:[]})
      
      var ix = await program.methods
        .proxyCollectReward(
          reward_index,
          bump 
        )
        .accounts({
          hydra,
          whirlpoolProgram: ORCA_WHIRLPOOL_PROGRAM_ID,
          whirlpool: samo_usdc_whirlpool_pubkey,
          position: position_pda.publicKey,
          positionTokenAccount: await deriveATA(wallet.publicKey, position_mint),
          rewardOwnerAccount: reward_ta.address,
          rewardVault: reward_info.vault,
          tokenProgram: TOKEN_PROGRAM_ID,
          authority
        })
        .instruction();

      const transaction = new TransactionBuilder(connection, wallet)
      .addInstruction(reward_ta);
    await  transaction.buildAndExecute()
   
      var magic = await threadProgram.methods
      .threadCreate(
        threadName,
        {
          accounts: ix.keys,
          programId: new PublicKey(ix.programId),
          data: ix.data,
        },
        {
          cron: {schedule: "5 * * * * * *"}
        },
      )
      .accounts({
        authority: wallet.publicKey,
        payer: wallet.publicKey,
        thread: hydra,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    await connection.confirmTransaction(magic);
      const post_position_data = await fetcher.getPosition(position_pda.publicKey, true);
    }
   var threadName = (Math.floor(Math.random()*99999)).toString()
    var [hydra] = PublicKey.findProgramAddressSync(
      [Buffer.from(SEED_QUEUE, 'utf-8'), wallet.publicKey.toBuffer(), Buffer.from(threadName, 'utf-8')],
      CLOCKWORK_THREAD_PROGRAM_ID,
    );
    console.log(hydra.toBase58())
    tx.addInstruction({instructions:[SystemProgram.transfer({
      /** Account that will transfer lamports */
      fromPubkey: wallet.publicKey,
      /** Account that will receive transferred lamports */
      toPubkey: hydra,
      /** Amount of lamports to transfer */
      lamports: 0.00666 * 10 ** 9
    })], signers:[], cleanupInstructions:[]})

    
    var ix = await program.methods
      .proxyClosePosition(bump)
      .accounts({
        hydra,
        whirlpoolProgram: ORCA_WHIRLPOOL_PROGRAM_ID,
        receiver: wallet.publicKey,
        position: position_pda.publicKey,
        positionMint: position_mint,
        positionTokenAccount: await deriveATA(wallet.publicKey, position_mint),
        tokenProgram: TOKEN_PROGRAM_ID,
        authority
      })
      .instruction();
     
      var magic = await threadProgram.methods
      .threadCreate(
        threadName,
        {
          accounts: ix.keys,
          programId: new PublicKey(ix.programId),
          data: ix.data,
        },
        {
          cron: {schedule: "5 * * * * * *"}
        },
      )
      .accounts({
        authority: wallet.publicKey,
        payer: wallet.publicKey,
        thread: hydra,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    await connection.confirmTransaction(magic);
    await tx.buildAndExecute()
    }
  });

});
