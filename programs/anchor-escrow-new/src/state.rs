use anchor_lang::prelude::*;

#[account]
pub struct Offer {
    // We store the offer maker's key so that they can cancel the offer (we need
    // to know who should sign).
    pub maker: Pubkey,

    // What kind of tokens does the offer maker want in return, and how many of
    // them?
    pub taker_mint: Pubkey,
    pub taker_amount: u64,

    // When the maker makes their offer, we store their offered tokens in an
    // escrow account that lives at a program-derived address, with seeds given
    // by the `Offer` account's address. Storing the corresponding bump here
    // means the client doesn't have to keep passing it.
    pub escrowed_maker_tokens_bump: u8,
}