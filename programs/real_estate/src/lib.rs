use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Mint, TokenAccount};
use anchor_lang::system_program;

declare_id!("GCPRCAMZWnTCtawVi6wMuXNy9Rdja6dvoastFFPkrBd3");

#[program]
pub mod real_estate {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        msg!("initialized successfully");
        Ok(())
    }
    
    pub fn create_property(ctx: Context<InitializeProperty>, property_data: PropertyData, nft_mint: Pubkey) -> Result<()> {
        require!(property_data.validate(), MarketplaceError::InvalidPropertyData);
        require!(ctx.accounts.validate_nft_mint(&nft_mint), MarketplaceError::InvalidNFTMint);
        
        let property = &mut ctx.accounts.property;
        require!(property_data.check_string_lengths(), MarketplaceError::StringTooLong);
        
        property.property = property_data;
        property.owner = ctx.accounts.signer.key();
        property.nft_mint = nft_mint;
        property.is_listed = false;
        property.list_price = 0;
        property.created_at = Clock::get()?.unix_timestamp;
        property.updated_at = property.created_at;
        
        Ok(())
    }

    pub fn list_property(ctx: Context<ListProperty>, price: u64) -> Result<()> {
        require!(price > 0, MarketplaceError::InvalidPrice);
        require!(price <= MAX_LISTING_PRICE, MarketplaceError::PriceExceedsLimit);
        
        let property = &mut ctx.accounts.property;
        require_keys_eq!(property.owner, ctx.accounts.owner.key(), MarketplaceError::NotPropertyOwner);
        require!(!property.is_listed, MarketplaceError::AlreadyListed);

        property.is_listed = true;
        property.list_price = price;
        property.updated_at = Clock::get()?.unix_timestamp;
        
        Ok(())
    }

    pub fn cancel_listing(ctx: Context<CancelListing>) -> Result<()> {
        let property = &mut ctx.accounts.property;
        
        require!(property.is_listed, MarketplaceError::NotListed);
        require_keys_eq!(property.owner, ctx.accounts.owner.key(), MarketplaceError::NotPropertyOwner);

        property.is_listed = false;
        property.list_price = 0;
        property.updated_at = Clock::get()?.unix_timestamp;
        
        Ok(())
    }

    pub fn buy_property(ctx: Context<BuyProperty>) -> Result<()> {
        let property = &mut ctx.accounts.property;
        let clock = Clock::get()?;
        
        require!(property.is_listed, MarketplaceError::NotListed);
        require_keys_eq!(property.owner, ctx.accounts.seller.key(), MarketplaceError::NotPropertyOwner);
        require_keys_neq!(ctx.accounts.buyer.key(), ctx.accounts.seller.key(), MarketplaceError::CannotBuyOwnProperty);
        
        let buyer_balance = ctx.accounts.buyer.lamports();
        require!(buyer_balance >= property.list_price.checked_add(MINIMUM_BALANCE_FOR_RENT).ok_or(MarketplaceError::Overflow)?,
            MarketplaceError::InsufficientFunds);
        
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.buyer.to_account_info(),
                    to: ctx.accounts.seller.to_account_info(),
                },
            ),
            property.list_price,
        )?;

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.seller_token_account.to_account_info(),
                    to: ctx.accounts.buyer_token_account.to_account_info(),
                    authority: ctx.accounts.seller.to_account_info(),
                },
                &[]
            ),
            1
        )?;

        property.owner = ctx.accounts.buyer.key();
        property.is_listed = false;
        property.list_price = 0;
        property.updated_at = clock.unix_timestamp;
        property.last_sale_price = Some(property.list_price);
        property.last_sale_date = Some(clock.unix_timestamp);

        Ok(())
    }
}

pub const MAX_STRING_LENGTH: usize = 200;
pub const MAX_LISTING_PRICE: u64 = 1_000_000_000_000;
pub const MINIMUM_BALANCE_FOR_RENT: u64 = 1_000_000;

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct InitializeProperty<'info> {
    #[account(init, payer = signer, space = 8 + std::mem::size_of::<Property>())]
    pub property: Account<'info, Property>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> InitializeProperty<'info> {
    pub fn validate_nft_mint(&self, nft_mint: &Pubkey) -> bool {
        true
    }
}

#[derive(Accounts)]
pub struct ListProperty<'info> {
    #[account(mut)]
    pub property: Account<'info, Property>,
    pub owner: Signer<'info>,
    /// CHECK: Token account checked in constraint
    #[account(
        mut,
        constraint = *seller_token_account.owner == owner.key()
    )]
    pub seller_token_account: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CancelListing<'info> {
    #[account(mut)]
    pub property: Account<'info, Property>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(price: u64)]
pub struct BuyProperty<'info> {
    #[account(mut)]
    pub property: Account<'info, Property>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub seller: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    /// CHECK: Token account checked in constraint
    #[account(
        mut,
        constraint = *seller_token_account.owner == seller.key()
    )]
    pub seller_token_account: AccountInfo<'info>,
    /// CHECK: Token account checked in constraint
    #[account(
        mut,
        constraint = *buyer_token_account.owner == buyer.key()
    )]
    pub buyer_token_account: AccountInfo<'info>,
}

#[account]
#[derive(Default)]
pub struct PropertyData {
    pub address: String,
    pub rooms: u64,
    pub bathrooms: u64,
    pub kitchens: u64,
    pub price: u64,
    pub city: String,
    pub north_view: String,
    pub south_view: String,
    pub east_view: String,
    pub west_view: String,
    pub image_url: String,
}

impl PropertyData {
    pub fn validate(&self) -> bool {
        !self.address.is_empty() 
            && !self.city.is_empty()
            && self.rooms > 0 
            && self.bathrooms > 0 
            && self.price > 0
            && !self.image_url.is_empty()
    }

    pub fn check_string_lengths(&self) -> bool {
        self.address.len() <= MAX_STRING_LENGTH
            && self.city.len() <= MAX_STRING_LENGTH
            && self.north_view.len() <= MAX_STRING_LENGTH
            && self.south_view.len() <= MAX_STRING_LENGTH
            && self.east_view.len() <= MAX_STRING_LENGTH
            && self.west_view.len() <= MAX_STRING_LENGTH
            && self.image_url.len() <= MAX_STRING_LENGTH
    }
}

#[account]
pub struct Property {
    pub owner: Pubkey,
    pub nft_mint: Pubkey,
    pub property: PropertyData,
    pub is_listed: bool,
    pub list_price: u64,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_sale_price: Option<u64>,
    pub last_sale_date: Option<i64>,
}

#[error_code]
pub enum MarketplaceError {
    NotPropertyOwner,
    NotListed,
    AlreadyListed,
    InvalidPrice,
    PriceExceedsLimit,
    InvalidPropertyData,
    InsufficientFunds,
    InsufficientNFTBalance,
    StringTooLong,
    InvalidNFTMint,
    NFTDelegated,
    NFTFrozen,
    CannotBuyOwnProperty,
    Overflow,
}





/*
use anchor_lang::prelude::*;
use anchor_spl::token::{Transfer , Token};

declare_id!("GCPRCAMZWnTCtawVi6wMuXNy9Rdja6dvoastFFPkrBd3");

#[program]
pub mod real_estate {


    use super::*;
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("initialized successfully");
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
        
        require_keys_eq!(property.owner , ctx.accounts.seller.key() , MarketplaceErr::NotPropertyOwner);

        let transfer_instruction = Transfer{
            from : ctx.accounts.buyer.to_account_info(),
            to : ctx.accounts.seller.to_account_info(),
            authority : ctx.accounts.buyer.to_account_info()
        };

        let cpi_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            transfer_instruction,
        );

        anchor_spl::token::transfer(cpi_ctx, price)?;

        property.owner = ctx.accounts.buyer.key();
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer{
            from : ctx.accounts.seller_token_account.to_account_info(),
            to : ctx.accounts.buyer_token_account.to_account_info(),
            authority : ctx.accounts.seller.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        anchor_spl::token::transfer(cpi_ctx, 1)?;


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



#[error_code]
pub enum MarketplaceErr{
    #[msg("you are not the owner of this property !")]
    NotPropertyOwner
}
*/
