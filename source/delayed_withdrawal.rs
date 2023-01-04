use near_sdk::{EpochHeight, Balance, env};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use super::EPOCH_QUANTITY_FOR_DELAYED_WITHDRAWAL;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct DelayedWithdrawal {
    /// Near balance that the user requested to withdraw.
    pub near_amount: Balance,
    /// It is only needed in order to understand when it is possible to give
    /// the user his funds, because the funds can only be returned after EPOCH_QUANTITY_TO_DELAYED_WITHDRAWAL (8) epochs
    /// with delayed_withdraw method.
    pub started_epoch_height: EpochHeight
}

impl DelayedWithdrawal {
    pub fn get_epoch_quantity_to_take_delayed_withdrawal(&self, current_epoch_height: EpochHeight) -> u64 {
        if current_epoch_height < self.started_epoch_height {
            env::panic_str("Current epoch height must be greater or equal to started epoch height.");
        }
        let passed_epoch_height = current_epoch_height - self.started_epoch_height;

        if EPOCH_QUANTITY_FOR_DELAYED_WITHDRAWAL > passed_epoch_height {
            EPOCH_QUANTITY_FOR_DELAYED_WITHDRAWAL - passed_epoch_height
        } else {
            0
        }
    }

    pub fn can_take_delayed_withdrawal(&self, current_epoch_height: EpochHeight) -> bool {
        self.get_epoch_quantity_to_take_delayed_withdrawal(current_epoch_height) == 0
    }
}