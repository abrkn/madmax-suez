use utils::*;

// A market is a collection of bids (buy orders) and asks (sell orders)
pub struct Book {
    pub bids: Vec<Order>,
    pub asks: Vec<Order>,
}

impl Book {
    pub fn new() -> Book {
        Book {
            bids: vec![],
            asks: vec![],
        }
    }

    fn match_orders(maker: &Order, taker: &Order) -> Option<Trade> {
        // Make sure the opposite order offers equal or better price than requested
        if taker.side == OrderSide::Buy && taker.price < maker.price {
            None
        } else if taker.side == OrderSide::Sell && taker.price > maker.price {
            None
        } else {
            Some(Trade::new(maker, taker))
        }
    }

    pub fn cancel_order(&mut self, order_id: OrderId) -> Result<Order, ()> {
        // TODO: Figure out how to keep orders indexed
        match self.bids.iter().position(|x| x.id == order_id) {
            Some(index) => {
                let order = self.bids.remove(index);
                Ok(order)
            }
            None => {
                match self.asks.iter().position(|x| x.id == order_id) {
                    Some(index) => {
                        let order = self.asks.remove(index);
                        Ok(order)
                    }
                    None => {
                        Err(())
                    }
                }
            }
        }
    }

    // find an order of the opposite type. orders are already sorted in best
    // to worst price, allowing only looking at the first order
    pub fn execute_order(&mut self, mut order: Order) -> Vec<Trade> {
        let mut trades = vec![];

        while order.remaining > 0 {
            // Track whether to remove the opposite order in the parent scope
            // to avoid issues with trying to borrow self.bids/self.asks
            // for removing the opposite order if it's filled
            let mut filled_opposite_order = false;

            {
                let opposite_order_some = {
                    match order.side {
                        OrderSide::Buy => self.asks.first_mut(),
                        OrderSide::Sell => self.bids.first_mut(),
                    }
                };

                match opposite_order_some {
                    None => {
                        break;
                    }
                    Some(opposite_order) => {
                        match Book::match_orders(&opposite_order, &order) {
                            None => {
                                break;
                            }
                            Some(trade) => {
                                order.remaining -= trade.size;
                                opposite_order.remaining -= trade.size;

                                trades.push(trade);
                            }
                        }

                        // The opposite order has been entirely filled. Remove it
                        // from the market.
                        if opposite_order.remaining == 0 {
                            filled_opposite_order = true;
                        }
                    }
                }
            }

            if filled_opposite_order {
                match order.side {
                    OrderSide::Buy => { self.asks.remove(0); },
                    OrderSide::Sell => { self.bids.remove(0); },
                }
            }
        }

        // If the order is not entirely filled, insert it into the market
        if order.remaining > 0 {
            match order.side {
                OrderSide::Buy => {
                    let index = self.bids.iter().position(|x| x.price < order.price);
                    match index {
                        None => {
                            self.bids.push(order);
                        }
                        Some(index) => {
                            self.bids.insert(index, order);
                        }
                    }
                },
                OrderSide::Sell => {
                    let index = self.asks.iter().position(|x| x.price > order.price);
                    match index {
                        None => {
                            self.asks.push(order);
                        }
                        Some(index) => {
                            self.asks.insert(index, order);
                        }
                    }
                },
            }
        }

        trades
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils::*;

    #[test]
    fn it_adds_bids_in_correct_order() {
        let mut market = Book::new();

        market.execute_order(Order::new(1, 1, 1, OrderSide::Buy, 1001, 1));
        market.execute_order(Order::new(2, 1, 1, OrderSide::Buy, 1000, 1));
        market.execute_order(Order::new(3, 1, 1, OrderSide::Buy, 998, 1));
        market.execute_order(Order::new(4, 1, 1, OrderSide::Buy, 1002, 1));
        market.execute_order(Order::new(5, 1, 1, OrderSide::Buy, 1004, 1));

        assert_eq!(market.bids.len(), 5);
        assert_eq!(market.bids[0].price, 1004);
        assert_eq!(market.bids[1].price, 1002);
        assert_eq!(market.bids[2].price, 1001);
        assert_eq!(market.bids[3].price, 1000);
        assert_eq!(market.bids[4].price, 998);
    }

    #[test]
    fn it_adds_asks_in_correct_order() {
        let mut market = Book::new();

        market.execute_order(Order::new(1, 1, 1, OrderSide::Sell, 1001, 1));
        market.execute_order(Order::new(2, 1, 1, OrderSide::Sell, 1000, 1));
        market.execute_order(Order::new(3, 1, 1, OrderSide::Sell, 998, 1));
        market.execute_order(Order::new(4, 1, 1, OrderSide::Sell, 1002, 1));
        market.execute_order(Order::new(5, 1, 1, OrderSide::Sell, 1004, 1));

        assert_eq!(market.asks.len(), 5);
        assert_eq!(market.asks[0].price, 998);
        assert_eq!(market.asks[1].price, 1000);
        assert_eq!(market.asks[2].price, 1001);
        assert_eq!(market.asks[3].price, 1002);
        assert_eq!(market.asks[4].price, 1004);
    }

    #[test]
    fn it_matches_in_correct_order() {
        let mut market = Book::new();

        market.execute_order(Order::new(1, 1, 1, OrderSide::Buy, 1001, 10));
        market.execute_order(Order::new(2, 1, 1, OrderSide::Buy, 1002, 10));
        market.execute_order(Order::new(3, 1, 1, OrderSide::Buy, 1000, 10));

        // ASK 15 @ 1010
        // Should take 10 @ 1002 from #2 and 5 @ 1001 from #1
        let trades = market.execute_order(Order::new(4, 1, 1, OrderSide::Sell, 990, 15));

        assert_eq!(trades.len(), 2);
        assert_eq!(trades[0].price, 1002);
        assert_eq!(trades[0].size, 10);
        assert_eq!(trades[0].maker_order_id, 2);
        assert_eq!(trades[0].taker_order_id, 4);

        assert_eq!(trades[1].price, 1001);
        assert_eq!(trades[1].size, 5);
        assert_eq!(trades[1].maker_order_id, 1);
        assert_eq!(trades[1].taker_order_id, 4);
    }

    #[test]
    fn it_can_annihilate_two_orders() {
        let mut market = Book::new();

        market.execute_order(Order::new(1, 1, 1, OrderSide::Sell, 1000, 10));
        let trades = market.execute_order(Order::new(2, 1, 1, OrderSide::Buy, 1010, 10));

        assert_eq!(market.bids.len(), 0);
        assert_eq!(market.asks.len(), 0);
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].price, 1000);
        assert_eq!(trades[0].size, 10);
    }

    #[test]
    fn it_matches_bids_with_same_price_in_correct_order() {
        let mut market = Book::new();

        market.execute_order(Order::new(1, 1, 1, OrderSide::Buy, 1000, 10));
        market.execute_order(Order::new(2, 1, 1, OrderSide::Buy, 1000, 10));
        market.execute_order(Order::new(3, 1, 1, OrderSide::Buy, 1001, 10));
        market.execute_order(Order::new(4, 1, 1, OrderSide::Buy, 1001, 10));

        // ASK 40 @ 1000
        // Should take 10 from 3, 4, 1, 2
        let trades = market.execute_order(Order::new(5, 1, 1, OrderSide::Sell, 990, 40));

        assert_eq!(trades.len(), 4);
        assert_eq!(trades[0].maker_order_id, 3);
        assert_eq!(trades[1].maker_order_id, 4);
        assert_eq!(trades[2].maker_order_id, 1);
        assert_eq!(trades[3].maker_order_id, 2);
    }

    #[test]
    fn it_cancels_order() {
        let mut market = Book::new();

        market.execute_order(Order::new(1, 1, 1, OrderSide::Buy, 1001, 10));
        let canceled_order = market.cancel_order(1).unwrap();

        assert_eq!(canceled_order.remaining, 10);
    }

    #[test]
    fn it_fails_to_cancel_when_order_does_not_exist() {
        let mut market = Book::new();

        market.execute_order(Order::new(1, 1, 1, OrderSide::Buy, 1001, 10));
        assert!(market.cancel_order(2).is_err());
    }

    #[test]
    fn it_cannot_cancel_same_order_twice() {
        let mut market = Book::new();

        market.execute_order(Order::new(1, 1, 1, OrderSide::Buy, 1001, 10));

        let canceled_order = market.cancel_order(1).unwrap();
        assert_eq!(canceled_order.remaining, 10);

        let canceled_order = market.cancel_order(1);
        assert!(canceled_order.is_err());
    }
}
