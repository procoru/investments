use serde::Deserialize;

use crate::core::GenericResult;
use crate::broker_statement::partial::PartialBrokerStatement;
use crate::types::Date;
use crate::util;

use super::balance::Balance;
use super::common::{Ignore, deserialize_date};
use super::open_positions::OpenPositions;
use super::security_info::SecurityInfoSection;
use super::transactions::Transactions;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OFX {
    #[serde(rename = "SIGNONMSGSRSV1")]
    _signon: Ignore,

    #[serde(rename = "INVSTMTMSGSRSV1")]
    statement: StatementSection,

    #[serde(rename = "SECLISTMSGSRSV1")]
    security_info: SecurityInfoSection,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StatementSection {
    #[serde(rename = "INVSTMTTRNRS")]
    response: StatementResponse,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StatementResponse {
    #[serde(rename = "TRNUID")]
    _id: Ignore,
    #[serde(rename = "STATUS")]
    _status: Ignore,
    #[serde(rename = "INVSTMTRS")]
    report: Report,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Report {
    #[serde(rename = "DTASOF", deserialize_with = "deserialize_date")]
    date: Date,
    #[serde(rename = "CURDEF")]
    currency: String,
    #[serde(rename = "INVACCTFROM")]
    _account: Ignore,
    #[serde(rename = "INVTRANLIST")]
    transactions: Transactions,
    #[serde(rename = "INVPOSLIST")]
    open_positions: OpenPositions,
    #[serde(rename = "INVBAL")]
    balance: Balance,
}

impl OFX {
    pub fn parse(self) -> GenericResult<PartialBrokerStatement> {
        let report = self.statement.response.report;
        let currency = report.currency;
        let transactions = report.transactions;

        let (start_date, end_date) = util::parse_period(
            transactions.start_date, transactions.end_date)?;

        if report.date < start_date || end_date <= report.date {
            // The report contains transactions for the specified period, but balance and open
            // positions info - for the statement generation date.
            return Err!(concat!(
                "Firstrade reports always must be generated for the period starting from account ",
                "opening date until the date when the report is generated"))
        }

        let mut statement = PartialBrokerStatement::new();
        statement.set_period((start_date, end_date))?;

        statement.set_starting_assets(false)?;
        report.balance.parse(&mut statement, &currency)?;

        let securities = self.security_info.parse()?;
        transactions.parse(&mut statement, &currency, &securities)?;
        report.open_positions.parse(&mut statement, &securities)?;

        statement.validate()
    }
}