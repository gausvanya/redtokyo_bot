use crate::bot::libs::iris_api::{DuelRateError, IrisAPI};

const BASE_BET_GOLD: f64 = 50.0;
const BASE_RATE: f64 = 2.0;
const TARGET_VALUE: f64 = BASE_BET_GOLD * BASE_RATE;

const ROUND_STEP: f64 = 5.0;
const MIN_BET_LOWER: f64 = 15.0;
const MIN_BET_UPPER: f64 = 100.0;

pub struct DuelRate {
    pub min_bet: u64,
    pub rate: f64,
}

pub async fn get_minimum_duel_rate() -> Result<DuelRate, DuelRateError> {
    let api = IrisAPI::new();

    let order_book = api
        .get_order_book()
        .await
        .map_err(|e| DuelRateError::Api(e.to_string()))?;

    let mid_price = order_book
        .mid_price()
        .ok_or(DuelRateError::EmptyOrderBook)?;

    if mid_price <= 0.0 || !mid_price.is_finite() {
        return Err(DuelRateError::InvalidPrice(mid_price));
    }

    let raw_min_bet = TARGET_VALUE / mid_price;
    let rounded = (raw_min_bet / ROUND_STEP).round() * ROUND_STEP;
    let clamped = rounded.clamp(MIN_BET_LOWER, MIN_BET_UPPER);

    Ok(DuelRate {
        min_bet: clamped as u64,
        rate: mid_price,
    })
}