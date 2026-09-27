//! In-memory user store.
//! Handlers only call these methods. So you can swap this for a real DB later.

use std::collections::BTreeMap;
use std::sync::RwLock;

use shared::{CreateUser, UpdateUser, User};

use crate::error::AppError;

/// Data behind the lock. One lock guards both fields, so they stay in sync.
#[derive(Default)]
struct Inner {
    // BTreeMap keeps keys sorted, so the list comes out in id order.
    users: BTreeMap<u64, User>,
    last_id: u64,
}

/// Thread-safe store. Many readers or one writer at a time.
#[derive(Default)]
pub struct UserStore {
    inner: RwLock<Inner>,
}

impl UserStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn list(&self) -> Vec<User> {
        let inner = self.inner.read().expect("lock poisoned");
        // `cloned()` copies each User, so we can drop the lock after this.
        inner.users.values().cloned().collect()
    }

    pub fn get(&self, id: u64) -> Result<User, AppError> {
        let inner = self.inner.read().expect("lock poisoned");
        inner.users.get(&id).cloned().ok_or(AppError::NotFound(id))
    }

    pub fn create(&self, input: CreateUser) -> Result<User, AppError> {
        // Validate before we take the write lock. Keep the lock short.
        let name = validate_name(&input.name)?;
        let email = validate_email(&input.email)?;

        let mut inner = self.inner.write().expect("lock poisoned");
        ensure_email_free(&inner.users, &email, None)?;

        inner.last_id += 1;
        let user = User {
            id: inner.last_id,
            name,
            email,
        };
        inner.users.insert(user.id, user.clone());
        Ok(user)
    }

    pub fn update(&self, id: u64, input: UpdateUser) -> Result<User, AppError> {
        // `Option<&str>` -> `Option<Result<..>>` -> `Result<Option<..>>`, then `?`.
        let name = input.name.as_deref().map(validate_name).transpose()?;
        let email = input.email.as_deref().map(validate_email).transpose()?;

        let mut inner = self.inner.write().expect("lock poisoned");
        if let Some(email) = &email {
            ensure_email_free(&inner.users, email, Some(id))?;
        }

        let user = inner.users.get_mut(&id).ok_or(AppError::NotFound(id))?;
        if let Some(name) = name {
            user.name = name;
        }
        if let Some(email) = email {
            user.email = email;
        }
        Ok(user.clone())
    }

    pub fn delete(&self, id: u64) -> Result<(), AppError> {
        let mut inner = self.inner.write().expect("lock poisoned");
        inner
            .users
            .remove(&id)
            .map(|_| ())
            .ok_or(AppError::NotFound(id))
    }
}

// ---------- Validation helpers ----------

fn validate_name(name: &str) -> Result<String, AppError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("name must not be empty".into()));
    }
    if name.chars().count() > 50 {
        return Err(AppError::Validation("name must be 50 chars or less".into()));
    }
    Ok(name.to_string())
}

/// A very small email check. It is not full RFC 5322.
fn validate_email(email: &str) -> Result<String, AppError> {
    let email = email.trim().to_lowercase();
    let ok = match email.split_once('@') {
        Some((local, domain)) => {
            !local.is_empty() && domain.contains('.') && !email.contains(char::is_whitespace)
        }
        None => false,
    };
    if !ok {
        return Err(AppError::Validation(format!("invalid email: {email:?}")));
    }
    Ok(email)
}

/// Fail if another user already has this email.
/// `except` is the id to skip. We use it on update, so a user can keep its own email.
fn ensure_email_free(
    users: &BTreeMap<u64, User>,
    email: &str,
    except: Option<u64>,
) -> Result<(), AppError> {
    let taken = users
        .values()
        .any(|u| u.email == email && Some(u.id) != except);
    if taken {
        return Err(AppError::Conflict(format!("email already used: {email}")));
    }
    Ok(())
}
