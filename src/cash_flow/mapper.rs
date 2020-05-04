use crate::broker_statement::{
    BrokerStatement, ForexTrade, StockBuy, StockSell, Dividend, Fee, IdleCashInterest};
use crate::currency::{Cash, CashAssets};
use crate::types::Date;

pub fn map_broker_statement_to_cash_flow(statement: &BrokerStatement) -> Vec<CashFlow> {
    CashFlowMapper{cash_flows: Vec::new()}.process(statement)
}

struct CashFlowMapper {
    cash_flows: Vec<CashFlow>,
}

impl CashFlowMapper {
    fn process(mut self, statement: &BrokerStatement) -> Vec<CashFlow> {
        for fee in &statement.fees {
            self.fee(fee);
        }

        for cash_flow in &statement.cash_flows {
            self.deposit_or_withdrawal(cash_flow)
        }

        for interest in &statement.idle_cash_interest {
            self.interest(interest);
        }

        for trade in &statement.forex_trades {
            self.forex_trade(trade);
        }

        self.cash_flows.sort_by_key(|cash_flow| cash_flow.date);
        self.cash_flows
    }

    fn fee(&mut self, fee: &Fee) {
        self.add_static(fee.date, fee.amount, if fee.amount.is_negative() {
            "Комиссия брокера"
        } else {
            "Возврат излишне удержанной комиссии"
        });
    }

    fn deposit_or_withdrawal(&mut self, assets: &CashAssets) {
        self.add_static(assets.date, assets.cash, if assets.cash.is_positive() {
            "Ввод денежных средств"
        } else {
            "Вывод денежных средств"
        });
    }

    fn interest(&mut self, interest: &IdleCashInterest) {
        self.add_static(interest.date, interest.amount, "Проценты на остаток по счету");
    }

    fn forex_trade(&mut self, trade: &ForexTrade) {
        let description = format!("Конвертация {} -> {}", trade.from, trade.to);
        let cash_flow = self.add(trade.conclusion_date, -trade.from, description);
        cash_flow.sibling_amount.replace(trade.to);

        if !trade.commission.is_zero() {
            let description = format!("Комиссия за конвертацию {} -> {}", trade.from, trade.to);
            self.add(trade.conclusion_date, -trade.commission, description);
        };
    }

    fn add_static(&mut self, date: Date, amount: Cash, description: &str) -> &mut CashFlow {
        self.add(date, amount, description.to_owned())
    }

    fn add(&mut self, date: Date, amount: Cash, description: String) -> &mut CashFlow {
        self.cash_flows.push(CashFlow{date, amount, sibling_amount: None, description});
        self.cash_flows.last_mut().unwrap()
    }
}

// FIXME(konishchev): Rewrite all below
#[allow(dead_code)]
fn get_account_cash_flow(statement: &BrokerStatement) -> Vec<CashFlow> {
    let mut cash_flows = Vec::new();

    for trade in &statement.stock_buys {
        let (cash_flow, commission) = new_from_stock_buy(trade);

        cash_flows.push(cash_flow);
        if let Some(cash_flow) = commission {
            cash_flows.push(cash_flow);
        }
    }

    for trade in &statement.stock_sells {
        let (cash_flow, commission) = new_from_stock_sell(trade);

        cash_flows.push(cash_flow);
        if let Some(cash_flow) = commission {
            cash_flows.push(cash_flow);
        }
    }

    for dividend in &statement.dividends {
        let (cash_flow, paid_tax) = new_from_dividend(dividend);

        cash_flows.push(cash_flow);
        if let Some(cash_flow) = paid_tax {
            cash_flows.push(cash_flow);
        }
    }

    cash_flows
}

pub struct CashFlow {
    pub date: Date,
    pub amount: Cash,
    pub sibling_amount: Option<Cash>,
    pub description: String,
}

impl CashFlow {
    fn new(date: Date, amount: Cash, description: String) -> CashFlow {
        CashFlow {date, amount, sibling_amount: None, description}
    }
}

fn new_from_stock_buy(trade: &StockBuy) -> (CashFlow, Option<CashFlow>) {
    // FIXME(konishchev): Rounding
    let volume = trade.price * trade.quantity;
    let description = format!("Покупка {} {}", trade.quantity, trade.symbol);
    let cash_flow = CashFlow::new(trade.conclusion_date, -volume, description);

    let commission = if !trade.commission.is_zero() {
        let description = format!("Комиссия за покупку {} {}", trade.quantity, trade.symbol);
        // FIXME(konishchev): Rounding
        Some(CashFlow::new(trade.conclusion_date, -trade.commission, description))
    } else {
        None
    };

    (cash_flow, commission)
}

fn new_from_stock_sell(trade: &StockSell) -> (CashFlow, Option<CashFlow>) {
    // FIXME(konishchev): Rounding
    let volume = trade.price * trade.quantity;
    let description = format!("Продажа {} {}", trade.quantity, trade.symbol);
    let cash_flow = CashFlow::new(trade.conclusion_date, volume, description);

    let commission = if !trade.commission.is_zero() {
        let description = format!("Комиссия за продажу {} {}", trade.quantity, trade.symbol);
        // FIXME(konishchev): Rounding
        Some(CashFlow::new(trade.conclusion_date, -trade.commission, description))
    } else {
        None
    };

    (cash_flow, commission)
}

fn new_from_dividend(dividend: &Dividend) -> (CashFlow, Option<CashFlow>) {
    // FIXME(konishchev): Rounding
    let description = format!("Дивиденд от {}", dividend.issuer);
    let cash_flow = CashFlow::new(dividend.date, dividend.amount, description);

    let paid_tax = if !dividend.paid_tax.is_zero() {
        let description = format!("Налог, удержанный с дивиденда от {}", dividend.issuer);
        // FIXME(konishchev): Rounding
        Some(CashFlow::new(dividend.date, -dividend.paid_tax, description))
    } else {
        None
    };

    (cash_flow, paid_tax)
}