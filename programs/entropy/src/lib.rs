use {
    anchor_lang::{
        prelude::*,
        solana_program::{hash::hash, program::invoke, system_instruction::transfer, sysvar},
    },
    num_bigint_dig::{prime::probably_prime_miller_rabin, BigUint},
    // vdf::{
    //     VDF,
    //     VDFParams,
    //     WesolowskiVDFParams
    // },
};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

const INT_SIZE_BITS: u16 = 2048;
const PROOF_SIZE_BYTES: usize = ((2048 + 16) >> 4) * 4; // 516 bytes

/// A challenge to produce entropy with a verifiable delay function (VDF) in exchange for a reward
#[account]
pub struct Challenge {
    /// SHA-256 hash of the most recent blockhash when the challenge is initialized
    pub hash: [u8; 32],
    /// Number of iterations that the VDF must be run to solve the challenge
    pub difficulty: u64,
    /// Lamports that the VDF solver will be rewarded for providing the proof
    pub reward: u64,
    /// SHA-256 hash of the valid VDF proof to be used as a source of entropy
    pub entropy: [u8; 32],
}

/// Instruction to initialize a new challenge
#[derive(Accounts)]
pub struct Initialize<'info> {
    /// Account the challenge is stored at
    #[account(init, payer = payer, space = 8 + 32 + 8 + 8 + 32)]
    pub challenge: Account<'info, Challenge>,
    /// Account that will fund the account and pay for the reward
    #[account(mut)]
    pub payer: Signer<'info>,
    /// System program for transferring lamports
    pub system_program: Program<'info, System>,
    /// SlotHashes sysvar for seeding the VDF
    pub slot_hashes: UncheckedAccount<'info>,
}

/// Instruction to provide a proof for a solved challenge
#[derive(Accounts)]
pub struct Prove<'info> {
    /// Account the challenge is stored at
    #[account(mut)]
    pub challenge: Account<'info, Challenge>,
    /// Account that will receive the reward for solving the challenge
    #[account(mut)]
    pub prover: UncheckedAccount<'info>,
}

/// Instruction to provide a proof for a solved challenge
#[derive(Accounts)]
pub struct Prime {}

#[program]
pub mod entropy {
    use super::*;

    pub fn prime(ctx: Context<Prime>) -> ProgramResult {
        // 256-bit prime
        let big_uint =
            "66879465661348111229871989287968040993513351195484998191057052014006844134449"
                .parse::<BigUint>()
                .unwrap();

        msg!(
            "probably_prime_miller_rabin: {}",
            probably_prime_miller_rabin(&big_uint, 1, true)
        );

        Ok(())
    }

    pub fn initialize(ctx: Context<Initialize>, difficulty: u64, reward: u64) -> ProgramResult {
        let challenge = &mut ctx.accounts.challenge;
        let payer = &mut ctx.accounts.payer;
        let system_program = &ctx.accounts.system_program;
        let slot_hashes = &ctx.accounts.slot_hashes;
        if *slot_hashes.key != sysvar::slot_hashes::id() {
            return Err(ProgramError::UnsupportedSysvar); // TODO: customize error
        }

        // Transfer lamports from the payer to the challenge account to incentivize solving it
        if reward != 0 {
            invoke(
                &transfer(payer.key, &challenge.key(), reward),
                &[
                    payer.to_account_info(),
                    challenge.to_account_info(),
                    system_program.to_account_info(),
                ],
            )?;
        }

        // The first 16 bytes are the number of slot hashes and the most recent slot number
        // The next 32 bytes are the most recent slot hash
        let most_recent_slot_hash = &slot_hashes.try_borrow_data()?[16..16 + 32];

        // Initialize the challenge account with a hash of this slot hash as the seed for the VDF
        challenge.hash = hash(most_recent_slot_hash).to_bytes();
        challenge.difficulty = difficulty;
        challenge.reward = reward;
        challenge.entropy = [0; 32];

        Ok(())
    }

    // TODO: replace length of 516 with PROOF_SIZE_BYTES when Anchor supports
    pub fn prove(ctx: Context<Prove>, proof: [u8; 516]) -> ProgramResult {
        let challenge = &mut ctx.accounts.challenge;
        let prover = &mut ctx.accounts.prover;

        // Error if the challenge has already been solved
        if challenge.entropy != [0; 32] {
            return Err(ProgramError::InvalidAccountData); // TODO: customize error
        }

        // Verify the proof
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
        // let vdf = WesolowskiVDFParams(INT_SIZE_BITS).new();
        // vdf.verify(
        //     challenge.hash.as_ref(),
        //     challenge.difficulty,
        //     proof.as_ref(),
        // ).map_err(|_| ProgramError::InvalidArgument)?; // TODO: customize error

        // Hash the proof bytes to produce entropy
        challenge.entropy = hash(proof.as_ref()).to_bytes();

        // Transfer lamports from the challenge account to the prover
        let reward = challenge.reward;
        if reward != 0 {
            challenge.reward = 0;

            **challenge.to_account_info().try_borrow_mut_lamports()? = challenge
                .to_account_info()
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
