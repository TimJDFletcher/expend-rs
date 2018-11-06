#[macro_use]
extern crate failure;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate chrono;
extern crate time;

use chrono::prelude::*;
use failure::{Error, ResultExt};

pub mod expensify;
pub mod perdiem;

use expensify::TransactionList;

const EXPENSIFY_DATE_FORMAT: &str = "%Y-%m-%d";

mod context {
    use chrono::prelude::*;
    use failure::Error;
    use time::Duration;

    pub enum Country {
        Germany,
    }

    pub enum Currency {
        EUR,
    }

    impl std::fmt::Display for Country {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
            use self::Country::*;
            match self {
                Germany => f.write_str("Germany"),
            }
        }
    }

    impl std::fmt::Display for Currency {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
            use self::Currency::*;
            match self {
                EUR => f.write_str("EUR"),
            }
        }
    }

    impl Currency {
        pub fn symbol(&self) -> &'static str {
            use self::Currency::*;
            match self {
                EUR => "€",
            }
        }
    }

    impl Country {
        pub fn currency(&self) -> Currency {
            use self::Country::*;
            use self::Currency::*;
            match self {
                Germany => EUR,
            }
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct UserContext {
        pub project: String,
        pub email: String,
    }

    impl UserContext {
        pub fn apply_to_value(&self, mut payload: serde_json::Value) -> serde_json::Value {
            payload
                .get_mut("employeeEmail")
                .map(|v| *v = json!(self.email));
            payload
                .get_mut("transactionList")
                .and_then(serde_json::Value::as_array_mut)
                .map(|a| {
                    for item in a.iter_mut() {
                        item.get_mut("tag")
                            .map(|v| *v = json!(self.project.clone()));
                    }
                });
            payload
        }
    }

    pub struct Context {
        pub user: UserContext,
        pub reference_date: Option<Date<Utc>>,
    }

    impl Context {
        pub fn monday_of_reference_date(&self) -> Result<Date<Utc>, Error> {
            let d = self.reference_date.unwrap_or_else(Utc::today);
            d.checked_sub_signed(Duration::days(d.weekday().num_days_from_monday() as i64))
                .ok_or_else(|| format_err!("Failed to compute Monday from the given date."))
        }
    }

}

pub use context::{Context, UserContext};

pub enum Command {
    Payload(Option<Context>, String, serde_json::Value),
    PerDiem(Context, perdiem::TimePeriod, perdiem::Kind),
}

pub fn execute(
    user_id: String,
    password: String,
    cmd: Command,
    pre_execute: impl FnOnce(&str, &serde_json::Value) -> Result<(), Error>,
) -> Result<serde_json::Value, Error> {
    use self::Command::*;

    let client = expensify::Client::new(None, user_id, password);
    let (payload_type, payload) = match cmd {
        Payload(None, pt, p) => (pt, p),
        Payload(Some(ctx), pt, mut p) => (pt, ctx.user.apply_to_value(p)),
        PerDiem(ctx, period, kind) => {
            let payload =
                serde_json::value::to_value(TransactionList::from_per_diem(ctx, period, kind)?)?;
            ("create".to_string(), payload)
        }
    };
    let payload = serde_json::value::to_value(payload)?;
    pre_execute(&payload_type, &payload)?;
    client.post(&payload_type, payload)
}

pub fn from_date_string(s: &str) -> Result<Date<Utc>, Error> {
    let date_string = format!("{}T00:00:00Z", s);
    Ok(date_string
        .parse::<DateTime<Utc>>()
        .with_context(|_| format!("Could not parse date string '{}'", date_string))?
        .date())
}
