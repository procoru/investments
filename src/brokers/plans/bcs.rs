#[cfg(test)] use crate::commissions::CommissionCalc;
use crate::commissions::{CommissionSpec, CommissionSpecBuilder, CumulativeCommissionSpecBuilder};
#[cfg(test)] use crate::currency::Cash;
#[cfg(test)] use crate::types::TradeType;
use crate::util::RoundingMethod;

pub fn professional() -> CommissionSpec {
    CommissionSpecBuilder::new("RUB")
        .cumulative(CumulativeCommissionSpecBuilder::new()
            .tiers(btreemap!{
                dec!(         0) => dec!(0.0531),
                dec!(   100_000) => dec!(0.0413),
                dec!(   300_000) => dec!(0.0354),
                dec!( 1_000_000) => dec!(0.0295),
                dec!( 5_000_000) => dec!(0.0236),
                dec!(15_000_000) => dec!(0.0177),
            }).unwrap()
            .minimum_daily(dec!(35.4))
            .minimum_monthly(dec!(177))
            .percent_fee(dec!(0.01)) // Exchange fee
            .monthly_depositary(dec!(177))
            .build())
        .rounding_method(RoundingMethod::Truncate)
        .build()
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use super::*;

    #[rstest(trade_type => [TradeType::Buy, TradeType::Sell])]
    fn professional(trade_type: TradeType) {
        let mut calc = CommissionCalc::new(super::professional());

        let currency = "RUB";
        for &(date, shares, price) in &[
            (date!(2, 12, 2019),  35, dec!(2959.5)),
            (date!(2, 12, 2019),   3, dec!(2960)),
            (date!(2, 12, 2019),  18, dec!(2960)),
            (date!(3, 12, 2019), 107, dec!( 782.4)),
        ] {
            assert_eq!(
                calc.add_trade(date, trade_type, shares.into(), Cash::new(currency, price)).unwrap(),
                Cash::new(currency, dec!(0)),
            );
        }

        assert_eq!(calc.calculate(), hashmap!{
            date!(2, 12, 2019) => Cash::new(currency, dec!(68.45) + dec!(16.57)),
            date!(3, 12, 2019) => Cash::new(currency, dec!(44.45) + dec!(8.37)),

            // Actually we have different dates, but use fist day of the next month for simplicity
            date!(1,  1, 2020) => Cash::new(currency,
                dec!(64.10) + // Monthly minimum
                dec!(177) // Monthly depositary
            ),
        });
    }
}