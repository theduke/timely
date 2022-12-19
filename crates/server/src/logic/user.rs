use anyhow::{bail, Context};
use time::OffsetDateTime;

use crate::{
    db::{
        client_supabase::SupaDb,
        types::{User, UserCreate, UserFilter},
        Db,
    },
    PublicError,
};

pub fn validate_password_hash(hash: &str, password: &str) -> Result<bool, anyhow::Error> {
    let pw_hash = hash_password(password)?;
    Ok(hash == pw_hash)
}

pub fn hash_password(password: &str) -> Result<String, anyhow::Error> {
    Ok(blake3::hash(password.as_bytes()).to_string())
}

type AuthToken = String;

fn build_user_token(jwt_key: &TokenSecret, user: &User) -> Result<String, anyhow::Error> {
    let now = OffsetDateTime::now_utc();
    let expires = now.saturating_add(time::Duration::days(30));

    let claims = TokenClaims {
        iat: now.unix_timestamp() as u64,
        exp: Some(expires.unix_timestamp() as u64),
        sub: user.id.to_string(),
    };

    let token = encode_token(jwt_key, &claims)?;

    Ok(token)
}

pub fn user_login(
    db: &SupaDb,
    jwt_key: &str,
    username: &str,
    password: &str,
) -> Result<(User, AuthToken), anyhow::Error> {
    let user = db
        .user(UserFilter::Name(username.trim().to_string()))
        .context("Could not query user from db")?
        .ok_or_else(|| PublicError::msg("User not found"))?;

    let pw_valid = validate_password_hash(&user.password_hash, password)?;
    if !pw_valid {
        return Err(PublicError::msg("Wrong password").into());
    }

    let token = build_user_token(&new_token_secret(jwt_key), &user)?;

    Ok((user, token))
}

pub type TokenSecret = hmac::Hmac<sha2::Sha256>;

fn new_token_secret(value: &str) -> TokenSecret {
    use hmac::Mac;
    hmac::Hmac::new_from_slice(value.as_bytes()).unwrap()
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
struct TokenClaims {
    pub iat: u64,
    pub exp: Option<u64>,
    // Subject - the user id.
    pub sub: String,
}

fn encode_token(key: &TokenSecret, claims: &TokenClaims) -> Result<String, jwt::Error> {
    jwt::SignWithKey::sign_with_key(claims, key)
}

fn validate_token(key: &TokenSecret, token: &str) -> Result<TokenClaims, jwt::Error> {
    jwt::VerifyWithKey::verify_with_key(token, key)
}

pub fn load_user_for_token(db: &SupaDb, raw_key: &str, token: &str) -> Result<User, anyhow::Error> {
    let key = new_token_secret(raw_key);
    let claims = validate_token(&key, token)?;
    let id = claims
        .sub
        .parse()
        .context("Invalid 'sub' field in token: expected a u64 user id")?;
    let user = db.user(UserFilter::Id(id))?.context("User not found")?;
    Ok(user)
}

#[derive(Clone, Debug)]
pub struct Signup {
    pub username: String,
    pub email: String,
    pub password: String,
}

pub fn user_signup_and_login(
    db: &SupaDb,
    token_key: &str,
    data: Signup,
) -> Result<(User, AuthToken), anyhow::Error> {
    let user = user_signup(db, data)?;
    let token = build_user_token(&new_token_secret(token_key), &user)?;
    Ok((user, token))
}

pub fn user_signup(db: &SupaDb, signup: Signup) -> Result<User, anyhow::Error> {
    validate_email_address(&signup.email)?;
    validate_username(&signup.username)?;
    validate_password(&signup.password)?;

    let password_hash = hash_password(&signup.password)?;

    let pre = UserCreate {
        username: signup.username,
        email: signup.email,
        password_hash,
    };

    db.user_create(pre)
}

fn validate_email_address(val: &str) -> Result<(), anyhow::Error> {
    // FIXME: use proper validator!
    let (a, b) = val.split_once('@').context("invalid email")?;
    if a.trim().is_empty() || b.trim().is_empty() {
        bail!("invalid email");
    }
    Ok(())
}

fn validate_username(val: &str) -> Result<(), anyhow::Error> {
    if !val.chars().all(|c| c.is_alphanumeric()) {
        bail!("Invalid characters in username: only numbers and alphabetic characters allowed");
    }
    Ok(())
}

fn validate_password(val: &str) -> Result<(), anyhow::Error> {
    if val.len() < 8 {
        bail!("password must be at least 8 characters long");
    }
    Ok(())
}
