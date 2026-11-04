use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, OnceLock};

use keyring_core::api::{CredentialApi, CredentialStoreApi};
use keyring_core::{Credential, Entry, Error, Result};
use tokio::runtime::Runtime;

fn platform(error: impl std::error::Error + Send + Sync + 'static) -> Error {
    Error::PlatformFailure(Box::new(error))
}

// Private runtime that owns oo7's D-Bus connection and tasks.
fn rt() -> &'static Runtime {
    static RT: OnceLock<Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("keyring runtime")
    })
}

// Run on our runtime, block the calling thread on a std channel.
fn run<T: Send + 'static>(fut: impl Future<Output = T> + Send + 'static) -> T {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    rt().spawn(async move {
        let _ = tx.send(fut.await);
    });
    rx.recv().expect("keyring task dropped")
}

#[derive(Clone)]
pub struct Store {
    keyring: Arc<oo7::Keyring>,
}

impl std::fmt::Debug for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Store").finish_non_exhaustive()
    }
}

impl Store {
    pub async fn new() -> Result<Arc<Self>> {
        let keyring = rt()
            .spawn(oo7::Keyring::new())
            .await
            .map_err(platform)?
            .map_err(platform)?;
        Ok(Arc::new(Self {
            keyring: Arc::new(keyring),
        }))
    }
}

impl CredentialStoreApi for Store {
    fn vendor(&self) -> String {
        "halloy-secret-keyring-store".into()
    }

    fn id(&self) -> String {
        format!("{}:oo7", env!("CARGO_PKG_VERSION"))
    }

    fn build(
        &self,
        service: &str,
        user: &str,
        modifiers: Option<&HashMap<&str, &str>>,
    ) -> Result<Entry> {
        if modifiers.is_some_and(|m| !m.is_empty()) {
            return Err(Error::Invalid(
                "modifiers".into(),
                "No modifiers are supported".into(),
            ));
        }
        Ok(Entry::new_with_credential(Arc::new(PortalCredential {
            store: self.clone(),
            service: service.into(),
            user: user.into(),
        })))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn debug_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

pub(crate) struct PortalCredential {
    store: Store,
    service: String,
    user: String,
}

impl std::fmt::Debug for PortalCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PortalCredential")
            .field("service", &self.service)
            .field("user", &self.user)
            .finish_non_exhaustive()
    }
}

impl PortalCredential {
    // Returns the stored secret, or None if there is no matching item.
    fn find(&self) -> Result<Option<Vec<u8>>> {
        let (kr, svc, usr) = (
            self.store.keyring.clone(),
            self.service.clone(),
            self.user.clone(),
        );
        run(async move {
            let items = kr
                .search_items(&[
                    ("service", svc.as_str()),
                    ("username", usr.as_str()),
                ])
                .await?;
            match items.first() {
                // check oo7's Item API for how to read the secret
                Some(item) => Ok(Some(item.secret().await?.to_vec())),
                None => Ok::<_, oo7::Error>(None),
            }
        })
        .map_err(platform)
    }
}

impl CredentialApi for PortalCredential {
    fn set_secret(&self, secret: &[u8]) -> Result<()> {
        let (kr, svc, usr, secret) = (
            self.store.keyring.clone(),
            self.service.clone(),
            self.user.clone(),
            secret.to_vec(),
        );
        run(async move {
            kr.create_item(
                &format!("Halloy: {svc}"),
                &[("service", svc.as_str()), ("username", usr.as_str())],
                secret,
                true, // replace
            )
            .await
        })
        .map_err(platform)
    }

    fn get_secret(&self) -> Result<Vec<u8>> {
        self.find()?.ok_or(Error::NoEntry)
    }

    fn delete_credential(&self) -> Result<()> {
        self.find()?.ok_or(Error::NoEntry)?;
        let (kr, svc, usr) = (
            self.store.keyring.clone(),
            self.service.clone(),
            self.user.clone(),
        );
        run(async move {
            kr.delete(&[("service", svc.as_str()), ("username", usr.as_str())])
                .await
        })
        .map_err(platform)
    }

    fn get_credential(&self) -> Result<Option<Arc<Credential>>> {
        self.find()?.ok_or(Error::NoEntry)?;
        Ok(None)
    }

    fn get_specifiers(&self) -> Option<(String, String)> {
        Some((self.service.clone(), self.user.clone()))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn debug_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
