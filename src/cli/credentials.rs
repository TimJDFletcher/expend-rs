use failure::{format_err, Error, ResultExt};
use keyring::Keyring;
use std::{
    io::{stderr, stdin},
    str::FromStr,
};
use termion::input::TermRead;

const AUTHENTICATION_URL: &str = "https://www.expensify.com/tools/integrations/";

#[derive(Serialize, Deserialize)]
struct Credentials {
    user_id: String,
    user_secret: String,
}

impl From<(String, String)> for Credentials {
    fn from(f: (String, String)) -> Self {
        Credentials {
            user_id: f.0,
            user_secret: f.1,
        }
    }
}

impl From<Credentials> for (String, String) {
    fn from(c: Credentials) -> Self {
        (c.user_id, c.user_secret)
    }
}

impl FromStr for Credentials {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        Ok(serde_json::from_str(s)?)
    }
}

pub fn from_keychain_or_clear(clear: bool) -> Result<Option<(String, String)>, Error> {
    let username = username::get_user_name()?;
    let keyring = Keyring::new("expend-rs cli", &username);
    if clear {
        eprintln!("Clearing previously stored credentials");
        keyring.delete_password().ok();
        Ok(None)
    } else {
        let credentials: Credentials = match keyring.get_password() {
            Ok(pw) => pw.parse()?,
            Err(_) => return Ok(None),
        };
        Ok(Some(credentials.into()))
    }
}

pub fn store_in_keychain(creds: (String, String)) -> Result<(String, String), Error> {
    let username = username::get_user_name()?;
    let keyring = Keyring::new("expend-rs cli", &username);
    let creds: Credentials = creds.into();
    let creds_str = serde_json::to_string(&creds)?;
    keyring
        .set_password(&creds_str)
        .map_err(|_| format_err!("Could not set password"))?;
    Ok(creds.into())
}

pub fn query_from_user() -> Result<(String, String), Error> {
    eprint!("To obtain Expensify credentials, hit enter to generate them in your browser (use 'SAML' for login), or n otherwise: ");
    let mut answer = String::new();
    stdin().read_line(&mut answer)?;
    answer = answer.to_lowercase();
    if answer.trim() != "n" {
        open::that(AUTHENTICATION_URL).with_context(|_| {
            format!("Could not open '{}' in your browser.", AUTHENTICATION_URL)
        })?;
    }

    eprint!("Please enter your user user-id: ");
    let mut user_id = String::new();
    stdin().read_line(&mut user_id)?;

    eprint!("Please enter your user user secret (it won't display): ");
    let user_secret = stdin()
        .read_passwd(&mut stderr())?
        .ok_or_else(|| format_err!("Cannot proceed without a password."))?;
    eprintln!();
    Ok((user_id.trim().to_owned(), user_secret))
}
