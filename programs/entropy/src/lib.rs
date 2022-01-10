use {
    anchor_lang::{
        prelude::*,
        solana_program::{
            hash::hash,
            program::invoke,
            system_instruction::transfer
        }
    },
    vdf::{
        VDF,
        VDFParams,
        WesolowskiVDFParams
    }
};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

const INT_SIZE_BITS: u16 = 2048;
const PROOF_SIZE_BYTES: usize = ((2048 + 16) >> 4) * 4; // 516 bytes

#[program]
pub mod entropy {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, difficulty: u64, reward: u64) -> ProgramResult {
        let challenge = &mut ctx.accounts.challenge;
        let payer = &mut ctx.accounts.payer;
        let system_program = &ctx.accounts.system_program;
        let recent_blockhashes = &ctx.accounts.recent_blockhashes;

        if reward != 0 {
            invoke(
                &transfer(
                    payer.key,
                    &challenge.key(),
                    reward,
                ),
                &[
                    payer.to_account_info(),
                    challenge.to_account_info(),
                    system_program.to_account_info(),
                ],
            )?;
        }

        #[allow(deprecated)]
        let most_recent_blockhash = recent_blockhashes
            .first()
            .ok_or(ProgramError::UnsupportedSysvar)?
            .blockhash;

        challenge.hash = hash(most_recent_blockhash.to_bytes().as_ref()).to_bytes();
        challenge.difficulty = difficulty;
        challenge.reward = reward;
        challenge.outcome = [0; 32];

        Ok(())
    }

    pub fn prove(ctx: Context<Prove>, proof: [u8; PROOF_SIZE_BYTES]) -> ProgramResult {
        let challenge = &mut ctx.accounts.challenge;
        let prover = &mut ctx.accounts.prover;

        if challenge.outcome != [0; 32] {
            return Err(ProgramError::InvalidAccountData); // TODO: customize error
        }

        /*
        FIXME:
        Compiling classgroup v0.1.0
        error[E0432]: unresolved imports `libc::c_char`, `libc::c_int`, `libc::c_long`, `libc::c_ulong`, `libc::c_void`, `libc::c_double`, `libc::size_t`, `libc::strnlen`
         --> /Users/jordan/.cargo/registry/src/github.com-1ecc6299db9ec823/classgroup-0.1.0/src/gmp/mpz.rs:2:12
          |
        2 | use libc::{c_char, c_int, c_long, c_ulong, c_void, c_double, size_t, strnlen};
          |            ^^^^^^  ^^^^^  ^^^^^^  ^^^^^^^  ^^^^^^  ^^^^^^^^  ^^^^^^  ^^^^^^^ no `strnlen` in the root
          |            |       |      |       |        |       |         |
          |            |       |      |       |        |       |         no `size_t` in the root
          |            |       |      |       |        |       no `c_double` in the root
          |            |       |      |       |        no `c_void` in the root
          |            |       |      |       no `c_ulong` in the root
          |            |       |      no `c_long` in the root
          |            |       no `c_int` in the root
          |            no `c_char` in the root
         */
        let vdf = WesolowskiVDFParams(INT_SIZE_BITS).new();
        vdf.verify(
            challenge.hash.as_ref(),
            challenge.difficulty,
            proof.as_ref(),
        ).map_err(|_| ProgramError::InvalidArgument)?;

        challenge.outcome = hash(proof.as_ref()).to_bytes();

        let reward = challenge.reward;
        if reward != 0 {
            challenge.reward = 0;

            **challenge.to_account_info().try_borrow_mut_lamports()? = challenge.to_account_info()
                .lamports()
                .checked_sub(reward)
                .ok_or(ProgramError::InsufficientFunds)?; // TODO: customize error

            **prover.try_borrow_mut_lamports()? = prover
                .lamports()
                .checked_add(reward)
                .ok_or(ProgramError::InvalidArgument)?; // TODO: customize error
        }

        Ok(())
    }
}

#[account]
pub struct Challenge {
    pub hash: [u8; 32],
    pub difficulty: u64,
    pub reward: u64,
    pub outcome: [u8; 32],
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = payer, space = 8 + 32 + 8 + 32)]
    pub challenge: Account<'info, Challenge>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    #[allow(deprecated)]
    pub recent_blockhashes: Sysvar<'info, RecentBlockhashes>,
}

#[derive(Accounts)]
pub struct Prove<'info> {
    #[account(mut)]
    pub challenge: Account<'info, Challenge>,
    #[account(mut)]
    pub prover: UncheckedAccount<'info>,
}
