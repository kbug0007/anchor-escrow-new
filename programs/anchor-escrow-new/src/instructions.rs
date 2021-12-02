use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use crate::state::*;

#[derive(Accounts)]
#[instruction(escrowed_maker_tokens_bump: u8)]
pub struct Make<'info> {
    #[account(init, payer = offer_maker, space = 8 + 32 + 32 + 8 + 1)]
    pub offer: Account<'info, Offer>,

    #[account(mut)]
    pub offer_maker: Signer<'info>,
    #[account(mut, constraint = offer_makers_maker_tokens.mint == maker_mint.key())]
    pub offer_makers_maker_tokens: Account<'info, TokenAccount>,

    // This is where we'll store the offer maker's tokens.
    #[account(
    init,
    payer = offer_maker,
    seeds = [offer.key().as_ref()],
    bump = escrowed_maker_tokens_bump,
    token::mint = maker_mint,
    // We want the program itself to have authority over the escrow token
    // account, so we need to use some program-derived address here. Well,
    // the escrow token account itself already lives at a program-derived
    // address, so we can set its authority to be its own address.
    token::authority = escrowed_maker_tokens,
    )]
    pub escrowed_maker_tokens: Account<'info, TokenAccount>,

    pub maker_mint: Account<'info, Mint>,
    pub taker_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Accept<'info> {
    #[account(
    mut,
    // make sure the offer_maker account really is whoever made the offer!
    constraint = offer.maker == *offer_maker.key,
    // at the end of the instruction, close the offer account (don't need it
    // anymore) and send its rent back to the offer_maker
    close = offer_maker
    )]
    pub offer: Account<'info, Offer>,

    #[account(mut)]
    pub escrowed_maker_tokens: Account<'info, TokenAccount>,

    pub offer_maker: AccountInfo<'info>,
    pub offer_taker: Signer<'info>,

    #[account(
    mut,
    associated_token::mint = taker_mint,
    associated_token::authority = offer_maker,
    )]
    pub offer_makers_taker_tokens: Box<Account<'info, TokenAccount>>,

    #[account(
    mut,
    // double check that the offer_taker is putting up the right kind of
    // tokens!
    constraint = offer_takers_taker_tokens.mint == offer.taker_mint
    )]
    pub offer_takers_taker_tokens: Account<'info, TokenAccount>,
    #[account(mut)]
    pub offer_takers_maker_tokens: Account<'info, TokenAccount>,

    #[account(address = offer.taker_mint)]
    pub taker_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Cancel<'info> {
    #[account(
    mut,
    // make sure the offer_maker account really is whoever made the offer!
    constraint = offer.maker == *offer_maker.key,
    // at the end of the instruction, close the offer account (don't need it
    // anymore) and send its rent lamports back to the offer_maker
    close = offer_maker
    )]
    pub offer: Account<'info, Offer>,

    #[account(mut)]
    // the offer_maker needs to sign if they really want to cancel their offer
    pub offer_maker: Signer<'info>,

    #[account(mut)]
    // this is where to send the previously-escrowed tokens to
    pub offer_makers_maker_tokens: Account<'info, TokenAccount>,

    #[account(
    mut,
    seeds = [offer.key().as_ref()],
    bump = offer.escrowed_maker_tokens_bump
    )]
    pub escrowed_maker_tokens: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}
