#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait StakingContract {
    #[init]
    fn init(&self) {
        // Setting the reward speed at 0.0003 EGLD per second
        self.set_global_reward_speed(&BigUint::from(300000_u128));
        // Initializing the total stake and the last block
        self.set_total_stake(&BigUint::zero());
        self.set_last_block(0);
    }

    #[endpoint]
    #[payable("EGLD")]
    fn stake(&self, #[payment] amount: BigUint) -> SCResult<()> {
        let caller = self.blockchain().get_caller();

        let current_stake = self.get_stake(&caller).unwrap_or_else(BigUint::zero);
        self.update_rewards(&caller);

        let new_stake = &current_stake + &amount;
        self.set_stake(&caller, &new_stake);

        let total_stake = self.get_total_stake().unwrap_or_else(BigUint::zero);
        let new_total_stake = &total_stake + &amount;
        self.set_total_stake(&new_total_stake);

        Ok(())
    }

    #[endpoint]
    fn withdraw(&self, amount: BigUint) -> SCResult<()> {
        let caller = self.blockchain().get_caller();

        let current_stake = self.get_stake(&caller).unwrap_or_else(BigUint::zero);
        require!(
            amount <= current_stake,
            "Not enough staked EGLD to withdraw"
        );

        self.update_rewards(&caller);

        let new_stake = &current_stake - &amount;
        self.set_stake(&caller, &new_stake);

        let total_stake = self.get_total_stake().unwrap_or_else(BigUint::zero);
        let new_total_stake = &total_stake - &amount;
        self.set_total_stake(&new_total_stake);

        // self.send().direct_egld(&caller, &amount, b"withdraw");
        self.direct_egld(&caller, &amount, b"withdraw");

        Ok(())
    }

    #[endpoint]
    fn claim_rewards(&self) -> SCResult<()> {
        let caller = self.blockchain().get_caller();

        self.update_rewards(&caller);
        let rewards = self.get_rewards(&caller).unwrap_or_else(BigUint::zero);

        self.set_rewards(&caller, &BigUint::zero());

        // self.send().direct_egld(&caller, &rewards, b"rewards");
        self.direct_egld(&caller, &rewards, b"rewards");

        Ok(())
    }

    fn update_rewards(&self, address: &ManagedAddress) {
        let current_block = self.blockchain().get_block_nonce();
        let last_block = self.get_last_block().unwrap_or(0);
        let delta_time = current_block - last_block;

        let global_rewards_per_block =
            self.get_global_reward_speed().unwrap_or_else(BigUint::zero) * delta_time;

        let current_stake = self.get_stake(address).unwrap_or_else(BigUint::zero);
        let total_stake = self.get_total_stake().unwrap_or_else(BigUint::zero);

        let user_rewards = if total_stake != BigUint::zero() {
            &global_rewards_per_block * &current_stake / &total_stake
        } else {
            BigUint::zero()
        };

        let current_rewards = self.get_rewards(address).unwrap_or_else(BigUint::zero);
        let new_rewards = &current_rewards + &user_rewards;

        self.set_rewards(address, &new_rewards);
        self.set_last_block(current_block);
    }

    #[view(getStake)]
    #[storage_get("stake")]
    fn get_stake(&self, address: &ManagedAddress) -> Option<BigUint>;

    #[storage_set("stake")]
    fn set_stake(&self, address: &ManagedAddress, stake: &BigUint);

    #[view(getRewards)]
    #[storage_get("rewards")]
    fn get_rewards(&self, address: &ManagedAddress) -> Option<BigUint>;

    #[storage_set("rewards")]
    fn set_rewards(&self, address: &ManagedAddress, rewards: &BigUint);

    #[view(getGlobalRewardSpeed)]
    // #[storage_get("globalRewardSpeed")]
    #[storage_get("1")]
    fn get_global_reward_speed(&self) -> Option<BigUint>;

    // #[storage_get("globalRewardSpeed")]
    #[storage_set("1")]
    fn set_global_reward_speed(&self, speed: &BigUint);

    #[view(getTotalStake)]
    // #[storage_get("totalStake")]
    #[storage_get("2")]
    fn get_total_stake(&self) -> Option<BigUint>;

    // #[storage_get("totalStake")]
    #[storage_set("2")]
    fn set_total_stake(&self, stake: &BigUint);

    #[view(getLastBlock)]
    // #[storage_get("lastBlock")]
    #[storage_get("3")]
    fn get_last_block(&self) -> Option<u64>;

    // #[storage_get("lastBlock")]
    #[storage_set("3")]
    fn set_last_block(&self, block: u64);

    fn direct_egld(&self, to: &ManagedAddress, amount: &BigUint, endpoint_name: &[u8]) {
        let token = &EgldOrEsdtTokenIdentifier::egld();
        let nonce = 0;
        let gas = 0;
        let arguments = &[];

        self.send()
            .direct_with_gas_limit(to, token, nonce, amount, gas, endpoint_name, arguments);
    }
}
