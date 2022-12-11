use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::{
  associated_token::AssociatedToken,
  token::{Approve, Mint, MintTo, Revoke, Token, TokenAccount}
};
use mpl_token_metadata::{
  instruction::{freeze_delegated_account, thaw_delegated_account},
  ID as MetadataTokenId,
};
use mpl_token_metadata::state::{Metadata as MetadataAccount, TokenMetadataAccount};

declare_id!("EEtnT4dQAVRj4uKkBND9fszGr4e2UM9Sd6TKF45VPy4");

pub fn get_blocks_arrays() -> ([u8; 15], [u8; 36], [u8; 40], [u8; 28]) {
    let s = [1, 2, 3, 4, 11, 12, 19, 20, 37, 38, 55, 56, 87, 88, 119];
    let p = [5, 6, 7, 8, 9, 10, 13, 14, 15, 16, 17, 18, 31, 32, 33, 34, 35, 36, 49, 50, 51, 52, 53, 54, 81, 82, 83, 84, 85, 86, 113, 114, 115, 116, 117, 118];
    let d = [21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112];
    let f = [57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102];

    (s, p, d, f)
}

pub fn get_index(clock: &Clock, len: u8) -> usize {
  let i: usize = (clock.unix_timestamp % i64::from(len)).try_into().unwrap();
  i
}

pub fn get_block(atomic_number: u8) -> Option<(u8, u8)> {
  let (s, p, d, f) = get_blocks_arrays();
    if s.contains(&atomic_number) {
        Some((10, s.len().try_into().unwrap()))
    } else if p.contains(&atomic_number) {
        Some((20, p.len().try_into().unwrap()))
    } else if d.contains(&atomic_number) {
        Some((30, d.len().try_into().unwrap()))
    } else if f.contains(&atomic_number) {
        Some((40, f.len().try_into().unwrap()))
    } else {
        None
    }
}

#[program]
pub mod anchor_nft_staking {
    use anchor_lang::solana_program::program::invoke_signed;

    use super::*;

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        require!(ctx.accounts.stake_state.stake_state == StakeState::Unstaked, StakeError::AlreadyStaked);
        let clock = Clock::get().unwrap();
        msg!("Approving delegate");
        let cpi_approve_program = ctx.accounts.token_program.to_account_info();
        let cpi_approve_accounts = Approve {
          to: ctx.accounts.nft_token_account.to_account_info(),
          delegate: ctx.accounts.program_authority.to_account_info(),
          authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_approve_ctx = CpiContext::new(cpi_approve_program, cpi_approve_accounts);
        token::approve(cpi_approve_ctx, 1)?;
        msg!("Freezing token account");
        let authority_bump = *ctx.bumps.get("program_authority").unwrap();
        invoke_signed(&freeze_delegated_account(
          ctx.accounts.metadata_program.key(), 
          ctx.accounts.program_authority.key(), 
          ctx.accounts.nft_token_account.key(), 
          ctx.accounts.nft_edition.key(), 
          ctx.accounts.nft_mint.key()
          ), &[
              ctx.accounts.metadata_program.to_account_info(), 
              ctx.accounts.program_authority.to_account_info(), 
              ctx.accounts.nft_token_account.to_account_info(), 
              ctx.accounts.nft_edition.to_account_info(), 
              ctx.accounts.nft_mint.to_account_info()
          ], 
          &[&[b"authority", &[authority_bump]]])?;
          ctx.accounts.stake_state.token_account = ctx.accounts.nft_token_account.key();
          ctx.accounts.stake_state.user_pubkey = ctx.accounts.user.key();
          ctx.accounts.stake_state.stake_state = StakeState::Staked;
          ctx.accounts.stake_state.stake_start_time = clock.unix_timestamp;
          ctx.accounts.stake_state.last_stake_redeem = clock.unix_timestamp;
          ctx.accounts.stake_state.is_initialized = true;
        Ok(())
    }

    pub fn redeem(ctx: Context<Redeem>) -> Result<()> {
        require!(ctx.accounts.stake_state.is_initialized, StakeError::UninitializedAccount);
        require!(ctx.accounts.stake_state.stake_state == StakeState::Staked, StakeError::InvalidStakeState);
        let metadata: MetadataAccount = MetadataAccount::from_account_info(&ctx.accounts.mint_metadata)?;
        let name = &metadata.data.name;
        let split = name.split('#');
        let last = split.last().unwrap();
        let trimmed = last.trim();
        let num_str = trimmed.replace("\0", "");
        require!(num_str.chars().all(|c| c.is_digit(10)), StakeError::InvalidElementName);
        require!(num_str.len() > 0, StakeError::InvalidElementName);
        let atomic_number = num_str.parse::<u8>().unwrap();
        let (block, len) = get_block(atomic_number).unwrap();
        let (s, p, d, f) = get_blocks_arrays();
        let clock = Clock::get()?;
        let mut random = u8::MIN;
        let mut i: usize = get_index(&clock, len);
        if i == 0 {
          i = 1
        }
        msg!("Block: {}, Length: {}, Index: {}", block, len, i);
        if usize::from(len) == s.len() {
            random = s[i - 1];
            msg!("Random: {:?}", random);
        } else if usize::from(len) == p.len() {
            random = p[i - 1];
            msg!("Random: {:?}", random);
        } else if usize::from(len) == d.len() {
            random =  d[i - 1];
            msg!("Random: {:?}", random);
        } else if usize::from(len) == f.len() {
            random =  f[i - 1];
            msg!("Random: {:?}", random);
        } else {
            return err!(StakeError::InvalidBlockData)
        }
        msg!("Stake last redeem: {:?}", ctx.accounts.stake_state.last_stake_redeem);
        msg!("Current time: {:?}", clock.unix_timestamp);
        let unix_time = clock.unix_timestamp - ctx.accounts.stake_state.last_stake_redeem;
        msg!("Seconds since last redeem: {}", unix_time);
        let redeem_amount = (10 * i64::pow(10, 2) * unix_time) / (24 * 60 * 60) * (i64::from(random) / i64::from(block));
        msg!("Eligible redeem amount: {}", redeem_amount);
        msg!("Minting staking rewards");
        token::mint_to(
          CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), MintTo {
            mint: ctx.accounts.stake_mint.to_account_info(),
            to: ctx.accounts.user_stake_ata.to_account_info(),
            authority: ctx.accounts.stake_authority.to_account_info(),
          }, 
          &[&[
            b"mint".as_ref(), 
            &[*ctx.bumps.get("stake_authority").unwrap()]
            ]]), redeem_amount.try_into().unwrap())?;

        ctx.accounts.stake_state.last_stake_redeem = clock.unix_timestamp;
        msg!("Updated last stake redeem time: {:?}", ctx.accounts.stake_state.last_stake_redeem);
        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        require!(ctx.accounts.stake_state.is_initialized, StakeError::UninitializedAccount);
        require!(ctx.accounts.stake_state.stake_state == StakeState::Staked, StakeError::InvalidStakeState);
        let metadata: MetadataAccount = MetadataAccount::from_account_info(&ctx.accounts.mint_metadata)?;
        let name = &metadata.data.name;
        let split = name.split('#');
        let last = split.last().unwrap();
        let trimmed = last.trim();
        let num_str = trimmed.replace("\0", "");
        require!(num_str.chars().all(|c| c.is_digit(10)), StakeError::InvalidElementName);
        require!(num_str.len() > 0, StakeError::InvalidElementName);
        let atomic_number = num_str.parse::<u8>().unwrap();
        let (block, len) = get_block(atomic_number).unwrap();
        let (s, p, d, f) = get_blocks_arrays();
        let clock = Clock::get()?;
        let mut random = u8::MIN;
        let mut i: usize = get_index(&clock, len);
        if i == 0 {
          i = 1
        }
        msg!("Block: {}, Length: {}, Index: {}", block, len, i);
        if usize::from(len) == s.len() {
            random = s[i - 1];
            msg!("Random: {:?}", random);
        } else if usize::from(len) == p.len() {
            random = p[i - 1];
            msg!("Random: {:?}", random);
        } else if usize::from(len) == d.len() {
            random =  d[i - 1];
            msg!("Random: {:?}", random);
        } else if usize::from(len) == f.len() {
            random =  f[i - 1];
            msg!("Random: {:?}", random);
        } else {
            return err!(StakeError::InvalidBlockData)
        }
        msg!("Thawing token account");
        let authority_bump = *ctx.bumps.get("program_authority").unwrap();
        invoke_signed(
          &thaw_delegated_account(
            ctx.accounts.metadata_program.key(), 
            ctx.accounts.program_authority.key(), 
            ctx.accounts.nft_token_account.key(), 
            ctx.accounts.nft_edition.key(), 
            ctx.accounts.nft_mint.key(),
            ), &[
                ctx.accounts.metadata_program.to_account_info(), 
                ctx.accounts.program_authority.to_account_info(), 
                ctx.accounts.nft_token_account.to_account_info(), 
                ctx.accounts.nft_edition.to_account_info(), 
                ctx.accounts.nft_mint.to_account_info()
            ], 
            &[&[b"authority", &[authority_bump]]]
            )?;
        msg!("Revoking delegate");
        let cpi_revoke_program = ctx.accounts.token_program.to_account_info();
        let cpi_revoke_accounts = Revoke {
          source: ctx.accounts.nft_token_account.to_account_info(),
          authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_revoke_ctx = CpiContext::new(cpi_revoke_program, cpi_revoke_accounts);
        token::revoke(cpi_revoke_ctx)?;
        let clock = Clock::get()?;
        msg!("Stake last redeem: {:?}", ctx.accounts.stake_state.last_stake_redeem);
        msg!("Current time: {:?}", clock.unix_timestamp);
        let unix_time = clock.unix_timestamp - ctx.accounts.stake_state.last_stake_redeem;
        msg!("Seconds since last redeem: {}", unix_time);
        let redeem_amount = (10 * i64::pow(10, 2) * unix_time) / (24 * 60 * 60) * (i64::from(random) / i64::from(block));
        msg!("Eligible redeem amount: {}", redeem_amount);
        msg!("Minting staking rewards");
        token::mint_to(
          CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), MintTo {
            mint: ctx.accounts.stake_mint.to_account_info(),
            to: ctx.accounts.user_stake_ata.to_account_info(),
            authority: ctx.accounts.stake_authority.to_account_info(),
          }, 
          &[&[
            b"mint".as_ref(), 
            &[*ctx.bumps.get("stake_authority").unwrap()]
            ]]), redeem_amount.try_into().unwrap())?;

        ctx.accounts.stake_state.last_stake_redeem = clock.unix_timestamp;
        msg!("Updated last stake redeem time: {:?}", ctx.accounts.stake_state.last_stake_redeem);
        ctx.accounts.stake_state.stake_state = StakeState::Unstaked;
        msg!("NFT Unstaked");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Stake<'info> {
  #[account(mut)]
  pub user: Signer<'info>,
  #[account(
    mut,
    associated_token::mint=nft_mint,
    associated_token::authority=user
  )]
  pub nft_token_account: Account<'info, TokenAccount>,
  pub nft_mint: Account<'info, Mint>,
  /// CHECK: Manual validation
  #[account(owner=MetadataTokenId)]
  pub nft_edition: UncheckedAccount<'info>,
  #[account(
    init_if_needed,
    payer=user,
    space= std::mem::size_of::<UserStakeInfo>() + 8,
    seeds = [user.key().as_ref(), nft_token_account.key().as_ref()],
    bump
  )]
  pub stake_state: Account<'info, UserStakeInfo>,
  /// CHECK: Manual validation
  #[account(mut, seeds=["authority".as_bytes().as_ref()], bump)]
  pub program_authority: UncheckedAccount<'info>,
  pub token_program: Program<'info, Token>,
  pub system_program: Program<'info, System>,
  pub metadata_program: Program<'info, Metadata>,
}

#[derive(Accounts)]
pub struct Redeem<'info> {
  #[account(mut)]
  pub user: Signer<'info>,
  #[account(mut, token::authority=user)]
  pub nft_token_account: Account<'info, TokenAccount>,
  #[account(
    mut,
    constraint = *user.key == stake_state.user_pubkey,
    constraint = nft_token_account.key() == stake_state.token_account,
    seeds = [user.key().as_ref(), nft_token_account.key().as_ref()],
    bump
  )]
  pub stake_state: Account<'info, UserStakeInfo>,
  #[account(mut)]
  pub stake_mint: Account<'info, Mint>,
  /// CHECK: manual check
  #[account(seeds = ["mint".as_bytes().as_ref()], bump)]
  pub stake_authority: UncheckedAccount<'info>,
  #[account(
    init_if_needed,
    payer=user,
    associated_token::mint=stake_mint,
    associated_token::authority=user
  )]
  pub user_stake_ata: Account<'info, TokenAccount>,
  /// CHECK: manual check
  pub mint_metadata: AccountInfo<'info>,
  pub token_program: Program<'info, Token>,
  pub system_program: Program<'info, System>,
  pub associated_token_program: Program<'info, AssociatedToken>,
  pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
  #[account(mut)]
  pub user: Signer<'info>,
  #[account(mut, token::authority=user)]
  pub nft_token_account: Account<'info, TokenAccount>,
  pub nft_mint: Account<'info, Mint>,
  /// CHECK: Manual validation
  #[account(owner=MetadataTokenId)]
  pub nft_edition: UncheckedAccount<'info>,
  #[account(
    mut,
    constraint = *user.key == stake_state.user_pubkey,
    constraint = nft_token_account.key() == stake_state.token_account,
    seeds = [user.key().as_ref(), nft_token_account.key().as_ref()],
    bump
  )]
  pub stake_state: Account<'info, UserStakeInfo>,
  /// CHECK: Manual validation
  #[account(mut, seeds=["authority".as_bytes().as_ref()], bump)]
  pub program_authority: UncheckedAccount<'info>,
  #[account(mut)]
  pub stake_mint: Account<'info, Mint>,
  /// CHECK: manual check
  #[account(seeds = ["mint".as_bytes().as_ref()], bump)]
  pub stake_authority: UncheckedAccount<'info>,
  #[account(
    init_if_needed,
    payer=user,
    associated_token::mint=stake_mint,
    associated_token::authority=user
  )]
  pub user_stake_ata: Box<Account<'info, TokenAccount>>,
  /// CHECK: manual check
  pub mint_metadata: AccountInfo<'info>,
  pub token_program: Program<'info, Token>,
  pub system_program: Program<'info, System>,
  pub metadata_program: Program<'info, Metadata>,
  pub associated_token_program: Program<'info, AssociatedToken>,
  pub rent: Sysvar<'info, Rent>,
}

#[account]
pub struct UserStakeInfo {
  pub token_account: Pubkey,
  pub stake_start_time: i64,
  pub last_stake_redeem: i64,
  pub user_pubkey: Pubkey,
  pub stake_state: StakeState,
  pub is_initialized: bool,
}

#[derive(Debug, PartialEq, AnchorDeserialize, AnchorSerialize, Clone)]
pub enum StakeState {
  Unstaked,
  Staked,
}

impl Default for StakeState {
  fn default() -> Self {
      StakeState::Unstaked
  }
}

#[derive(Clone)]
pub struct Metadata;

impl anchor_lang::Id for Metadata {
  fn id() -> Pubkey {
      MetadataTokenId
  }
}

#[error_code]
pub enum  StakeError {
    #[msg("NFT already staked")]
    AlreadyStaked,
    #[msg("State account is uninitialized")]
    UninitializedAccount,
    #[msg("Stake state is invalid")]
    InvalidStakeState,
    #[msg("Invalid element name")]
    InvalidElementName,
    #[msg("Invalid block data")]
    InvalidBlockData
}