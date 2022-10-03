use anchor_lang::prelude::*;
use num_derive::*;
use nanorand::RNG;
declare_id!("2WWFGRA4f81ubcjtkh112obV8brzF6nkhBCDGh7Z8hqo");

#[program]
pub mod dice_roll {
    use super::*;
    use anchor_lang::solana_program::{program::invoke, system_instruction::transfer};
    // use anchor_lang::AccountsClose;

    pub fn setup(ctx: Context<Setup>, player: Pubkey, bet_amount: u64, dice_side_count: u8, vendor_seed: i64) -> Result<()> {
        let dice_roll = &mut ctx.accounts.dice_roll;

        dice_roll.players = [ctx.accounts.vendor.key(), player];
        dice_roll.vendor_seed = vendor_seed;
        dice_roll.bump = *ctx.bumps.get("dice_roll").unwrap();
        dice_roll.bet_amount = bet_amount;
        dice_roll.dice_side_count = dice_side_count;

        invoke(

            //Transfer money
            &transfer(
                ctx.accounts.vendor.to_account_info().key,
                dice_roll.to_account_info().key,
                dice_roll.bet_amount,
            ),
            &[
                ctx.accounts.vendor.to_account_info(),
                dice_roll.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        Ok(())
    }

    
    pub fn play(ctx: Context<Play>) -> Result<()> {
        let dice_roll = &mut ctx.accounts.dice_roll;

        invoke(
            &transfer(
                ctx.accounts.player.to_account_info().key,
                dice_roll.to_account_info().key,
                dice_roll.bet_amount,
            ),
            &[
                ctx.accounts.player.to_account_info(),
                dice_roll.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        let total_bet = dice_roll.bet_amount * 2;

        let result = dice_roll.play();
        match result{
            DiceRollResult::Draw => {},
            DiceRollResult::Finished {winner} => {
                **dice_roll.to_account_info().try_borrow_mut_lamports()? -= total_bet;

                if winner == *ctx.accounts.vendor.key {
                    **ctx.accounts.vendor.try_borrow_mut_lamports()? += total_bet;
                } else {
                    **ctx.accounts.player.to_account_info().try_borrow_mut_lamports()? += total_bet;
                }
            }
        }
        return Ok(());
    }


    pub fn delete(_ctx: Context<Delete>, player: Pubkey) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(player: Pubkey, bet_amount: u64, vendor_seed: i64)]
pub struct Setup<'info> {
    #[account(
        init, 
        payer = vendor, 
        space = DiceRoll::LEN,
        seeds = [b"dice-roll", vendor.key().as_ref(), player.as_ref()], bump
    )]
    pub dice_roll: Account<'info, DiceRoll>,
    #[account(mut)]
    pub vendor: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct Play<'info> {
    #[account(
        mut, 
        seeds = [b"dice-roll", vendor.key().as_ref(), player.key().as_ref()], bump
    )]
    pub dice_roll: Account<'info, DiceRoll>,
    #[account(mut)]
    pub player: Signer<'info>,
    #[account(mut)]
    /// CHECK
    pub vendor : AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(player: Pubkey)]
pub struct Delete<'info> {
    #[account(
        mut, 
        close = vendor,
        seeds = [b"dice-roll", vendor.key().as_ref(), player.as_ref()], bump
    )]
    pub dice_roll: Account<'info, DiceRoll>,
    #[account(mut)]
    pub vendor: Signer<'info>,
    pub system_program: Program<'info, System>,
}


#[account]
#[derive(Default)] 
pub struct DiceRoll {
    players: [Pubkey; 2], 
    vendor_seed: i64,
    state: DiceRollResult,
    bet_amount: u64,
    bump: u8,
    dice_side_count: u8
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Copy)]
pub enum DiceRollResult {
    Draw,
    Finished { winner: Pubkey },
}


impl Default for DiceRollResult {
    fn default() -> Self {
        Self::Draw
    }
}


impl DiceRoll {
    const LEN: usize = 64 + 8 + 33 + 8 + 8 + 8;

    pub fn play(&mut self) -> DiceRollResult {
        let mut rng = nanorand::tls_rng();
        let first_player_score = rng.generate::<u8>() % (self.dice_side_count+1);
        let second_player_score = rng.generate::<u8>() % (self.dice_side_count+1);

        self.state = DiceRollResult::Draw;
        if first_player_score != second_player_score {
            let winnerIndex = if second_player_score > first_player_score {1} else {0};
            self.state = DiceRollResult::Finished {winner: self.players[winnerIndex]};
        }
        return self.state;
    }
}

#[error_code]
pub enum DiceRollError {
    #[msg("Bet amount is too small")]
    BetTooSmall,

}
