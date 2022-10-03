use crate::CwTokenError;
use cosmwasm_std::Storage;
use cw_storage_plus::Item;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub(crate) const STORE_TOKEN_KEY: &str = "store_token";

pub trait TokenStorage: Serialize + DeserializeOwned + Sized {
    fn load(storage: &dyn Storage) -> Result<Self, CwTokenError> {
        Ok(Self::get_item().load(storage)?)
    }

    fn save(&self, storage: &mut dyn Storage) -> Result<(), CwTokenError> {
        Ok(Self::get_item().save(storage, self)?)
    }

    fn get_item<'a>() -> Item<'a, Self> {
        Item::<Self>::new(STORE_TOKEN_KEY)
    }
}
