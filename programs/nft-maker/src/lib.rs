use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::{
        invoke_signed, invoke,
    },
    system_instruction::{
        transfer,
        create_account,
    },
    sysvar::{
        rent::Rent
    },

};
use anchor_spl::{
    associated_token::{self, AssociatedToken, Create},
    token::{self, Mint, Token, MintTo},
};
use mpl_token_metadata::{
    instruction::{
        create_metadata_accounts, create_master_edition, 
        //update_metadata_accounts,

    },
    state::{
        Creator,
    },

};


declare_id!("Eyzo28Tk19g28ojXwPQPGDAWyjCKqtvy1KzJwdArUH1E");

#[program]
pub mod nft_maker {
    use super::*;
    pub fn initialize(
        ctx: Context<Initialize>,
        config_nonce: u8,
        vault_nonce: u8,
        authority: Pubkey,
        amount: u64,
    ) -> ProgramResult {
        msg!("Initializing NFT maker configuration.");
        let config = &mut ctx.accounts.configuration;
        config.nft_count = 0;
        config.payer_vault = *ctx.accounts.payer_vault.key;
        config.authority = authority;
        config.config_nonce = config_nonce;
        config.vault_nonce = vault_nonce;

        if amount != 0 {
            invoke(
                &transfer(
                    ctx.accounts.signer.to_account_info().key,
                    ctx.accounts.payer_vault.key,
                    amount,
                ),
                &[
                    ctx.accounts.signer.to_account_info(),
                    ctx.accounts.payer_vault.clone(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;
        }

        Ok(())
    }

    pub fn minting_nft(
        ctx: Context<MintingNFT>,
        name: String,
        symbol: String,
        uri: String,
        seller_fee_basis_points: u16,
        immutable: bool
    ) -> ProgramResult {
        msg!("Start minting NFT.");

        let recipient_tokens_key = associated_token::get_associated_token_address(
            ctx.accounts.recipient.key,
            ctx.accounts.mint.key,
        );
        if &recipient_tokens_key != ctx.accounts.recipient_token.key {
            return Err(ErrorCode::InvalidAssociatedTokenAddress.into());
        }

        let metaplex_program_id = mpl_token_metadata::ID;
        //let metaplex_program_id = *ctx.accounts.token_metadata_program.key;

        let config = &ctx.accounts.configuration;
        let seeds = &[
            config.to_account_info().key.as_ref(),
            &[config.config_nonce],
        ];
        let pda_signer = &[&seeds[..]];

        //create mint
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(Mint::LEN);
        invoke_signed(
            &create_account(
                ctx.accounts.payer_vault.key,
                ctx.accounts.mint.key,
                lamports,
                Mint::LEN as u64,
                ctx.accounts.token_program.to_account_info().key,
            ),
            &[
                ctx.accounts.payer_vault.clone(),
                ctx.accounts.mint.clone(),
                ctx.accounts.system_program.to_account_info(),
            ],
            pda_signer,
        )?;

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let accounts = anchor_spl::token::InitializeMint {
            mint: ctx.accounts.mint.clone(),
            rent: ctx.accounts.rent.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, accounts).with_signer(pda_signer);
        token::initialize_mint(
            cpi_ctx,
            0,
            ctx.accounts.payer_vault.key,
            Option::<&Pubkey>::Some(ctx.accounts.payer_vault.key),
        )?;
 
        //create associated token account for player
        if ctx.accounts.recipient_token.data_is_empty() {
            let cpi_accounts = Create {
                payer: ctx.accounts.payer_vault.clone(),
                associated_token: ctx.accounts.recipient_token.clone(),
                authority: ctx.accounts.recipient.clone(),
                rent: ctx.accounts.rent.to_account_info(),
                mint: ctx.accounts.mint.clone(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            };
            let cpi_program = ctx.accounts.associated_token_program.to_account_info();
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts).with_signer(pda_signer);
            associated_token::create(cpi_ctx)?;
        }

        //minting NFT for player
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.clone(),
            to: ctx.accounts.recipient_token.clone(),
            authority: ctx.accounts.payer_vault.clone(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts).with_signer(pda_signer);
        token::mint_to(cpi_ctx, 1)?;

        //Derive metadata account
        let metadata_seeds = &[
            "metadata".as_bytes(),
            metaplex_program_id.as_ref(),
            ctx.accounts.mint.key.as_ref(),
        ];
        let (metadata_account, _pda) =
            Pubkey::find_program_address(metadata_seeds, &metaplex_program_id);

        let creators = vec![
            Creator {
                address: *ctx.accounts.payer_vault.key,
                verified: false,
                share: 0,
            },
            Creator {
                address: *ctx.accounts.recipient.key,
                verified: false,
                share: 100,
            }
        ];

        let create_metadata_account_ix = create_metadata_accounts(
            metaplex_program_id,
            metadata_account,
            *ctx.accounts.mint.key,
            *ctx.accounts.payer_vault.key,
            *ctx.accounts.payer_vault.key,
            *ctx.accounts.payer_vault.key,
            name,
            symbol,
            uri,
            Some(creators), 
            seller_fee_basis_points,
            true,
            !immutable
        );

        invoke_signed(
            &create_metadata_account_ix,
            &[
                ctx.accounts.metadata.clone(),
                ctx.accounts.mint.clone(),
                ctx.accounts.payer_vault.clone(),
                ctx.accounts.token_metadata_program.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.rent.to_account_info(),

            ],
            pda_signer,
        )?;

        // Derive Master Edition account
        let master_edition_seeds = &[
            "metadata".as_bytes(),
            metaplex_program_id.as_ref(),
            ctx.accounts.mint.key.as_ref(),
            "edition".as_bytes(),
        ];
        let (master_edition_account, _pda) =
            Pubkey::find_program_address(master_edition_seeds, &metaplex_program_id);

        let create_master_edition_account_ix = create_master_edition(
            metaplex_program_id,
            master_edition_account,
            *ctx.accounts.mint.key,
            *ctx.accounts.payer_vault.key,
            *ctx.accounts.payer_vault.key,
            metadata_account,
            *ctx.accounts.payer_vault.key,
            Some(0),
        );

        invoke_signed(
            &create_master_edition_account_ix,
            &[
                ctx.accounts.masteredition.clone(),
                ctx.accounts.metadata.clone(),
                ctx.accounts.mint.clone(),
                ctx.accounts.payer_vault.clone(),
                ctx.accounts.token_metadata_program.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.rent.to_account_info(),

            ],
            pda_signer,
        )?;

        ctx.accounts.configuration.nft_count += 1;

        emit!(MintEvent {
            mint: ctx.accounts.mint.key.to_string(),
            recipient: ctx.accounts.recipient.key.to_string(),
            nft_count: ctx.accounts.configuration.nft_count.to_string(),
            status: "ok".to_string(),
        });

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(config_nonce: u8, vault_nonce: u8)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [configuration.to_account_info().key.as_ref()], 
        bump = vault_nonce,
    )]
    pub payer_vault: AccountInfo<'info>,
   
    #[account(
        init, payer = signer,
        seeds = [b"nft-maker".as_ref()],
        bump = config_nonce,
    )]
    pub configuration: Box<Account<'info, Configuration>>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}


#[account]
#[derive(Default)]
pub struct Configuration {
    pub config_nonce: u8,
    pub vault_nonce: u8,
    pub authority: Pubkey,
    pub payer_vault: Pubkey,

    pub nft_count: u64,
}

#[derive(Accounts)]
pub struct MintingNFT<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    
    pub recipient: AccountInfo<'info>,

    #[account(mut)]
    pub recipient_token: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [configuration.to_account_info().key.as_ref()],
        bump = configuration.vault_nonce,
    )]
    pub payer_vault: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"nft-maker".as_ref()],
        bump = configuration.config_nonce,
        constraint = configuration.payer_vault == payer_vault.key() @ErrorCode::PayerVaultMismatch,
        constraint = configuration.authority == signer.key() @ErrorCode::Unauthorized,
    )]
    pub configuration: Box<Account<'info, Configuration>>,

    #[account(mut, signer)]
    pub mint: AccountInfo<'info>,

    #[account(mut)]
    pub metadata: AccountInfo<'info>,

    #[account(mut)]
    pub masteredition: AccountInfo<'info>,

    pub token_metadata_program: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>,
    
}

#[event]
pub struct MintEvent {
    #[index]
    pub mint: String,
    #[index]
    pub recipient: String,
    #[index]
    pub status: String,
    pub nft_count: String,
}

#[error]
pub enum ErrorCode {
    #[msg("Payer vault account mismatch.")]
    PayerVaultMismatch,
    #[msg("Invalid owner.")]
    InvalidOwner,
    #[msg("You do not have sufficient permissions to perform this action.")]
    Unauthorized,
    #[msg("Invalid associated token address. Did you provide the correct address?")]
    InvalidAssociatedTokenAddress,
   
}
