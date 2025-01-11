use anchor_lang::prelude::*;

declare_id!("GCPRCAMZWnTCtawVi6wMuXNy9Rdja6dvoastFFPkrBd3");

#[program]
pub mod real_estate {

    use super::*;
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
    
    pub fn create_property(ctx : Context<InitializeProperty> , property_data : PropertyData) -> Result<()> {
        let property = &mut ctx.accounts.property;
        let owner = property.owner;
        property.property = property_data;
        Ok(())
    }

    pub fn buy_property(ctx : Context<InitializeProperty>) -> Result<()> {
        
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
    pub property : PropertyData
}
