use anchor_lang::prelude::*;

#[account]
pub struct Data {
    pub count: u8,
    pub bump: u8,
    pub timestamp: i64
}

impl Data {
}