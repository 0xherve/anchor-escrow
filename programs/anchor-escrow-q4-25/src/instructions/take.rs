#![allow(unused_imports)]

use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

use crate::Escrow;

#[derive(Accounts)]
pub struct Take<'info> {
    // TODO: Implement Take Accounts
    #[account(mut)]
    taker: Signer<'info>,
    #[account(mut)]
    maker: SystemAccount<'info>,

    //Token Mint accounts
    #[account(mint::token_program = token_program)]
    pub mint_a: InterfaceAccount<'info, Mint>,
    #[account(mint::token_program = token_program)]
    pub mint_b: InterfaceAccount<'info, Mint>,

    //escrow
    #[account(
         mut,
         close = maker,
         seeds = [b"escrow", maker.key().as_ref(), &escrow.seed.to_le_bytes()],
         bump = escrow.bump
     )]
    pub escrow: Account<'info, Escrow>,

    //vault
    #[account(
         mut,
         associated_token::mint = mint_a,
         associated_token::authority = escrow,
         associated_token::token_program = token_program,
     )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    // Maker and Taker ATAs
    #[account(
         mut,
         associated_token::mint = mint_b,
         associated_token::authority = taker,
         associated_token::token_program = token_program,
     )]
    pub taker_ata_b: InterfaceAccount<'info, TokenAccount>,
    #[account(
         init_if_needed,
         payer = taker,
         associated_token::mint = mint_a,
         associated_token::authority = taker,
         associated_token::token_program = token_program,
     )]
    pub taker_ata_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
         init_if_needed,
         payer = taker,
         associated_token::mint = mint_b,
         associated_token::authority = maker,
         associated_token::token_program = token_program,
     )]
    pub maker_ata_b: InterfaceAccount<'info, TokenAccount>,

    //programs
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Take<'info> {
    // TODO: Implement Take Instruction
    // Includes Deposit, Withdraw and Close Vault

    pub fn transfer_token_b(&self) -> Result<()> {
        let ctx = CpiContext::new(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.taker_ata_b.to_account_info(),
                mint: self.mint_b.to_account_info(),
                to: self.maker_ata_b.to_account_info(),
                authority: self.taker.to_account_info(),
            },
        );

        transfer_checked(ctx, self.escrow.receive, self.mint_b.decimals)?;
        Ok(())
    }

    pub fn take(&self) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"escrow",
            self.maker.to_account_info().key.as_ref(),
            &self.escrow.seed.to_le_bytes(),
            &[self.escrow.bump],
        ]];
        
        // Send Tokens type(B) to Maker
    let ctx = CpiContext::new(
        self.token_program.to_account_info(),
        TransferChecked {
            from: self.taker_ata_b.to_account_info(),
            mint: self.mint_b.to_account_info(),
            to: self.maker_ata_b.to_account_info(),
            authority: self.taker.to_account_info(),
        },
    );
    transfer_checked(ctx, self.escrow.receive, self.mint_b.decimals)?;
    

         //Send Tokens type(A) to Taker
    let ctx = CpiContext::new_with_signer(
        self.token_program.to_account_info(),
        TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.taker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        },
        signer_seeds
    );
    transfer_checked(ctx, self.vault.amount, self.mint_a.decimals)?;

        //Close the accounts
    let close_ctx = CpiContext::new_with_signer(
        self.token_program.to_account_info(),
        CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        },
        signer_seeds,
    );
        close_account(close_ctx)?;
        Ok(())
    }
}