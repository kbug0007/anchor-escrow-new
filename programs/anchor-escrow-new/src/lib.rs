pub mod state;
pub mod instructions;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use crate::state::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod anchor_escrow {
    use super::*;

    // Make a binding offer of `offer_maker_amount` of one kind of tokens in
    // exchange for `offer_taker_amount` of some other kind of tokens. This
    // will store the offer maker's tokens in an escrow account.
    pub fn make(
        ctx: Context<Make>,
        escrowed_maker_tokens_bump: u8,
        offer_maker_amount: u64,
        offer_taker_amount: u64,
    ) -> ProgramResult {
        // Store some state about the offer being made. We'll need this later if
        // the offer gets accepted or cancelled.
        let offer = &mut ctx.accounts.offer;
        offer.maker = ctx.accounts.offer_maker.key();
        offer.taker_mint = ctx.accounts.taker_mint.key();
        offer.taker_amount = offer_taker_amount;
        offer.escrowed_maker_tokens_bump = escrowed_maker_tokens_bump;

        // Transfer the maker's tokens to the escrow account.
        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.offer_makers_maker_tokens.to_account_info(),
                    to: ctx.accounts.escrowed_maker_tokens.to_account_info(),
                    // The offer_maker had to sign from the client
                    authority: ctx.accounts.offer_maker.to_account_info(),
                },
            ),
            offer_maker_amount,
        )
    }

    // Accept an offer by providing the right amount + kind of tokens. This
    // unlocks the tokens escrowed by the offer maker.
    pub fn accept(ctx: Context<Accept>) -> ProgramResult {
        // Transfer the taker's tokens to the maker.
        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    // Don't need to worry about the accepter sneakily providing
                    // the wrong kind of tokens because we've already checked
                    // that while deriving Accounts for the Accept struct.
                    from: ctx.accounts.offer_takers_taker_tokens.to_account_info(),
                    to: ctx.accounts.offer_makers_taker_tokens.to_account_info(),
                    // The offer_taker had to sign from the client
                    authority: ctx.accounts.offer_taker.to_account_info(),
                },
            ),
            // The necessary amount was set by the offer maker.
            ctx.accounts.offer.taker_amount,
        )?;

        // Transfer the maker's tokens (the ones they escrowed) to the taker.
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.escrowed_maker_tokens.to_account_info(),
                    to: ctx.accounts.offer_takers_maker_tokens.to_account_info(),
                    // Cute trick: the escrowed_maker_tokens is its own
                    // authority/owner (and a PDA, so our program can sign for
                    // it just below)
                    authority: ctx.accounts.escrowed_maker_tokens.to_account_info(),
                },
                &[&[
                    ctx.accounts.offer.key().as_ref(),
                    &[ctx.accounts.offer.escrowed_maker_tokens_bump],
                ]],
            ),
            // The amount here is just the entire balance of the escrow account.
            ctx.accounts.escrowed_maker_tokens.amount,
        )?;

        // Finally, close the escrow account and refund the maker (they paid for
        // its rent-exemption).
        anchor_spl::token::close_account(CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::CloseAccount {
                account: ctx.accounts.escrowed_maker_tokens.to_account_info(),
                destination: ctx.accounts.offer_maker.to_account_info(),
                authority: ctx.accounts.escrowed_maker_tokens.to_account_info(),
            },
            &[&[
                ctx.accounts.offer.key().as_ref(),
                &[ctx.accounts.offer.escrowed_maker_tokens_bump],
            ]],
        ))
    }

    pub fn cancel(ctx: Context<Cancel>) -> ProgramResult {
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.escrowed_maker_tokens.to_account_info(),
                    to: ctx.accounts.offer_makers_maker_tokens.to_account_info(),
                    // Cute trick: the escrowed_maker_tokens is its own
                    // authority/owner (and a PDA, so our program can sign for
                    // it just below)
                    authority: ctx.accounts.escrowed_maker_tokens.to_account_info(),
                },
                &[&[
                    ctx.accounts.offer.key().as_ref(),
                    &[ctx.accounts.offer.escrowed_maker_tokens_bump],
                ]],
            ),
            ctx.accounts.escrowed_maker_tokens.amount,
        )?;

        // Close the escrow's token account and refund the maker (they paid for
        // its rent-exemption).
        anchor_spl::token::close_account(CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::CloseAccount {
                account: ctx.accounts.escrowed_maker_tokens.to_account_info(),
                destination: ctx.accounts.offer_maker.to_account_info(),
                authority: ctx.accounts.escrowed_maker_tokens.to_account_info(),
            },
            &[&[
                ctx.accounts.offer.key().as_ref(),
                &[ctx.accounts.offer.escrowed_maker_tokens_bump],
            ]],
        ))
    }
}

