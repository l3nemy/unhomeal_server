#![doc = include_str!("../routes.md")]
mod apply;
mod loginout;
mod meal;
mod rate;
mod test;
mod user;

pub use apply::{apply_route, ApplyParam, GetApplicationParam};
pub use loginout::{login, logout};
pub use rate::{get_rate_route, post_rate_route, rank, Rate, RateLevel, RateParam};
pub use test::test_route;
pub use user::user_route;