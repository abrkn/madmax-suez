use std::collections::HashMap;
use std::sync::{RwLock};
use utils::*;
use std::sync::{Arc};

pub type Amount = i64;

#[derive(Clone)]
pub struct Balances {
    pub balances: Arc<RwLock<HashMap<(UserId, AssetId), Amount>>>,
    config: Config,
}

// TODO: Could lock per user
impl Balances {
    pub fn new(config: Config) -> Balances {
        Balances {
            balances: Arc::new(RwLock::new(HashMap::new())),
            config: config,
        }
    }

    fn get_balance_from_unlocked(map: &HashMap<(UserId, AssetId), Amount>, user_id: UserId, asset_id: AssetId) -> Amount {
        match map.get(&(user_id, asset_id)) {
            None => 0,
            Some(balance) => *balance
        }
    }

    pub fn adjust_balance(&mut self, user_id: UserId, asset_id: AssetId, change: Amount) -> Amount {
        Balances::adjust_balance_from_unlocked(&mut self.balances.write().unwrap(), user_id, asset_id, change)
    }

    fn adjust_balance_from_unlocked(map: &mut HashMap<(UserId, AssetId), Amount>, user_id: UserId, asset_id: AssetId, change: i64) -> Amount {
        let prev_balance = Balances::get_balance_from_unlocked(map, user_id, asset_id);
        let next_balance = prev_balance + change;
        map.insert((user_id, asset_id), next_balance as Amount);
        next_balance
    }

    pub fn get_balance(&self, user_id: UserId, asset_id: AssetId) -> Amount {
        let balances = self.balances.read().unwrap();
        Balances::get_balance_from_unlocked(&*balances, user_id, asset_id)
    }

    // TODO: Move to Order impl
    fn get_requirement_for_order(&self, order: &Order) -> (AssetId, Amount) {
        let ref market = self.config.markets[&order.market_id];
        match order.side {
            OrderSide::Buy => (market.quote_asset_id, (order.remaining * order.price) as Amount),
            OrderSide::Sell => (market.base_asset_id, order.remaining as Amount),
        }
    }

    pub fn settle(&mut self, trade: &Trade) {
        let market = &self.config.markets[&trade.market_id];
        let total = trade.price * trade.size;

        let (buy_user_id, sell_user_id) = match trade.side {
            OrderSide::Buy => (trade.maker_user_id, trade.taker_user_id),
            OrderSide::Sell => (trade.taker_user_id, trade.maker_user_id),
        };

        let mut balances = self.balances.write().unwrap();

        Balances::adjust_balance_from_unlocked(&mut balances, buy_user_id, market.base_asset_id, trade.size as Amount);
        Balances::adjust_balance_from_unlocked(&mut balances, sell_user_id, market.quote_asset_id, total as Amount);
    }

    pub fn user_can_afford_order(&self, order: &Order) -> bool {
        let balances = self.balances.read().unwrap();
        let (asset_id, balance_requirement) = self.get_requirement_for_order(order);
        Balances::get_balance_from_unlocked(&balances, order.user_id, asset_id) >= balance_requirement
    }

    pub fn debit_for_order(&mut self, order: &Order) {
        let mut balances = self.balances.write().unwrap();
        let (asset_id, balance_requirement) = self.get_requirement_for_order(order);
        Balances::adjust_balance_from_unlocked(&mut balances, order.user_id, asset_id, -balance_requirement);
    }

    pub fn credit_for_canceled_order(&mut self, order: &Order) {
        let mut balances = self.balances.write().unwrap();
        let (asset_id, balance_requirement) = self.get_requirement_for_order(order);
        Balances::adjust_balance_from_unlocked(&mut balances, order.user_id, asset_id, balance_requirement);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils::*;

    #[test]
    fn it_returns_zero_balance_for_unknown() {
        let balances = Balances::new(Config::hardcoded());
        let balance = balances.get_balance(123, 123);
        assert_eq!(balance, 0);
    }

    #[test]
    fn it_can_adjust_from_unknown() {
        let mut balances = Balances::new(Config::hardcoded());
        balances.adjust_balance(1, 2, 100);
        balances.adjust_balance(2, 2, -50);

        assert_eq!(balances.get_balance(1, 2), 100);
        assert_eq!(balances.get_balance(2, 2), -50);
    }

    #[test]
    fn it_debits_correct_amount_for_bid() {
        let mut balances = Balances::new(Config::hardcoded());

        {
            let mut unlocked = balances.balances.write().unwrap();
            unlocked.insert((1, 2), 1000);
        }

        let order = Order::new(1, 1, 1, OrderSide::Buy, 10, 20);
        balances.debit_for_order(&order);

        {
            let unlocked = balances.balances.read().unwrap();
            assert_eq!(unlocked[&(1, 2)], 1000 - 10 * 20);
        }
    }

    #[test]
    fn it_debits_correct_amount_for_ask() {
        let mut balances = Balances::new(Config::hardcoded());

        {
            let mut unlocked =  balances.balances.write().unwrap();
            unlocked.insert((1, 1), 1000);
        }

        let order = Order::new(1, 1, 1, OrderSide::Sell, 10, 20);
        balances.debit_for_order(&order);

        {
            let unlocked = balances.balances.read().unwrap();
            assert_eq!(unlocked[&(1, 1)], 1000 - 20);
        }
    }

    // TODO: Giving back remainder when an order is canceled

    #[test]
    fn it_settles_correctly_for_annihilation() {
        let mut balances = Balances::new(Config::hardcoded());
        const buy_user_id: UserId = 101;
        const sell_user_id: UserId = 102;
        const base_asset_id: AssetId = 1;
        const quote_asset_id: AssetId = 2;


        {
            let mut unlocked =  balances.balances.write().unwrap();
            unlocked.insert((buy_user_id, base_asset_id), 25);
            unlocked.insert((sell_user_id, quote_asset_id), 30);
        }

        let trade = Trade {
            price: 1000,
            size: 500,
            maker_order_id: 15,
            taker_order_id: 14,
            maker_user_id: buy_user_id,
            taker_user_id: sell_user_id,
            side: OrderSide::Buy,
            market_id: 1,
        };

        balances.settle(&trade);

        {
            let unlocked = balances.balances.read().unwrap();
            assert_eq!(unlocked[&(buy_user_id, base_asset_id)], 25 + 500);
            assert_eq!(unlocked[&(sell_user_id, quote_asset_id)], 30 + 500 * 1000);
        }
    }
}
