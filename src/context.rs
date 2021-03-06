use chrono::prelude::*;
use failure::Error;
use time::Duration;

#[derive(Serialize, Deserialize)]
pub enum Country {
    Germany,
}

impl Default for Country {
    fn default() -> Self {
        Country::Germany
    }
}

pub enum Currency {
    EUR,
}

impl std::str::FromStr for Country {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, <Self as std::str::FromStr>::Err> {
        use self::Country::*;
        match s {
            "germany" | "Germany" => Ok(Germany),
            _ => bail!(
                "Invalid country identifier: '{}'. Country is not implemented.",
                s
            ),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum Destination {
    IndiaOther,
}

impl std::fmt::Display for Destination {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        use self::Destination::*;
        match self {
            IndiaOther => f.write_str("India-Other"),
        }
    }
}

impl std::str::FromStr for Destination {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, <Self as std::str::FromStr>::Err> {
        match s {
            "India-Other" | "india-other" => Ok(Destination::IndiaOther),
            _ => bail!(
                "Invalid destination for Germany: '{}'. Destination is not implemented.",
                s
            ),
        }
    }
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
    #[serde(default)]
    pub country: Country,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<Destination>,
    #[serde(default)]
    pub tags: Tags,
    #[serde(default)]
    pub categories: Categories,
}

#[derive(Serialize, Deserialize)]
pub struct Tags {
    pub travel: Tag,
}

#[derive(Serialize, Deserialize)]
pub struct Categories {
    pub per_diems: Category,
}

#[derive(Serialize, Deserialize)]
pub struct Category {
    pub name: String,
}

impl Default for Categories {
    fn default() -> Self {
        Categories {
            per_diems: Category {
                name: "Per Diem/Stipend (pre-approved)".to_string(),
            },
        }
    }
}

impl Default for Tags {
    fn default() -> Self {
        Tags {
            travel: Tag {
                name: "Travel".to_string(),
                billable: true,
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub billable: bool,
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
    pub comment: Option<String>,
}

impl Context {
    pub fn monday_of_reference_date(&self) -> Result<Date<Utc>, Error> {
        let d = self.reference_date.unwrap_or_else(Utc::today);
        d.checked_sub_signed(Duration::days(d.weekday().num_days_from_monday() as i64))
            .ok_or_else(|| format_err!("Failed to compute Monday from the given date."))
    }
}
