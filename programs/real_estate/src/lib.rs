use anchor_lang::prelude::*;
use anchor_lang::system_program::Transfer;
use anchor_spl::token::{Token};

declare_id!("GCPRCAMZWnTCtawVi6wMuXNy9Rdja6dvoastFFPkrBd3");

#[program]
pub mod real_estate {


    use super::*;
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
    
    pub fn create_property(ctx : Context<InitializeProperty> , property_data : PropertyData , nft_mint : Pubkey) -> Result<()> {
        let property = &mut ctx.accounts.property;
        property.property = property_data;
        property.owner = property.owner.key();
        property.nft_mint = nft_mint;
        Ok(())
    }

    pub fn buy_property(ctx : Context<BuyProperty> , price : u64) -> Result<()> {
        let property = &mut ctx.accounts.property;
        let transfer_instruction = Transfer{
            from : ctx.accounts.seller,
            to : ctx.accounts.buyer,
            authority : ctx.accounts.buyer
        };
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize{}


#[derive(Accounts)]
pub struct InitializeProperty<'info>{
    #[account(
        init,
        payer = signer,
        space = 8 + std::mem::size_of::<Property>()
    )]
    pub property : Account<'info , Property>,
    #[account(mut)]
    pub signer : Signer<'info>,
    pub system_program : Program<'info , System>
}

#[derive(Accounts)]
pub struct BuyProperty<'info> {
    #[account(mut)]
    pub property: Account<'info, Property>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub seller: SystemAccount<'info>,
    /// CHECK: Safe because we only use it for CPI
    pub system_program: Program<'info, System>,
    /// CHECK: Safe because we only use it for CPI
    pub token_program: Program<'info, Token>,
    #[account(mut, constraint = seller_token_account.owner == seller.key())]
    pub seller_token_account: Account<'info, anchor_spl::token::TokenAccount>,
    #[account(mut)]
    pub buyer_token_account: Account<'info, anchor_spl::token::TokenAccount>,
}


#[account]
pub struct PropertyData{
    pub address : String,
    pub rooms : u64,
    pub bathrooms : u64,
    pub kitchens : u64,
    pub price : u64,
    pub city : String,
    pub north_view : String,
    pub south_view : String,
    pub east_view : String,
    pub west_view : String,
    pub image_url : String
}


#[account]
pub struct Property{
    pub owner : Pubkey,
    pub nft_mint : Pubkey,
    pub property : PropertyData
}




