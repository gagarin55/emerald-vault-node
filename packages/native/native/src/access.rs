use std::convert::TryFrom;
use std::path::Path;
use std::str::FromStr;

use neon::handle::Handle;
use neon::object::Object;
use neon::prelude::{FunctionContext, JsObject, JsString};
use neon::types::{JsNull, JsUndefined,};

use emerald_vault::{
    Address,
    core::chains::{Blockchain, EthereumChainId},
    storage::{
        default_path,
        error::VaultError,
        vault::{
            VaultAccess, VaultStorage
        },
    },
    structs::{
        wallet::{Wallet, WalletAccount},
    }
};

pub struct VaultConfig {
    pub chain: Option<EthereumChainId>,
    pub dir: String,
    pub show_hidden: bool
}

pub struct MigrationConfig {
    pub dir: String,
}

pub fn obj_get_str(cx: &mut FunctionContext, obj: &Handle<JsObject>, name: &str) -> Option<String> {
    match obj.get(cx, name) {
        Ok(val) => {
            if val.is_a::<JsNull>() {
                None
            } else if val.is_a::<JsUndefined>() {
                None
            } else {
                Some(val.downcast::<JsString>().expect("Not a string").value())
            }
        },
        Err(_) => None
    }
}

pub fn args_get_str(cx: &mut FunctionContext, pos: i32) -> Option<String> {
    match cx.argument_opt(pos) {
        None => None,
        Some(v) => if v.is_a::<JsString>() {
            match v.downcast::<JsString>() {
                Ok(v) => Some(v.value()),
                Err(_) => None
            }
        } else {
            None
        }
    }
}

impl VaultConfig {

    pub fn get_config(cx: &mut FunctionContext) -> VaultConfig {
        let config = cx.argument::<JsObject>(0).expect("Vault Config is not provided");
        let dir = match obj_get_str(cx, &config, "dir") {
            Some(val) => val,
            None => default_path().to_str().expect("No default path for current OS").to_string()
        };

        let chain = match obj_get_str(cx, &config, "chain") {
            Some(chain) => Some(EthereumChainId::from_str(chain.as_str()).expect("Invalid chain")),
            None => None
        };

        return VaultConfig {
            chain,
            dir: dir.to_string(),
            show_hidden: false
        }
    }

    pub fn get_storage(&self) -> VaultStorage {
        let dir = Path::new(&self.dir);
        let vault = VaultStorage::create(dir)
            .expect("Vault is not created");
        vault
    }
}

impl MigrationConfig {
    pub fn get_config(cx: &mut FunctionContext) -> MigrationConfig {
        let config = cx.argument::<JsObject>(0).expect("Vault Config is not provided");
        let dir = match obj_get_str(cx, &config, "dir") {
            Some(val) => val,
            None => default_path().to_str().expect("No default path for current OS").to_string()
        };
        return MigrationConfig {
            dir: dir.to_string()
        }
    }
}

pub struct WrappedVault {
    pub cfg: VaultConfig
}

impl WrappedVault {

    pub fn new(cfg: VaultConfig) -> WrappedVault {
        WrappedVault {cfg}
    }

    pub fn find_account<'a>(w: &'a Wallet, addr: &Address, blockchain: Blockchain) -> Option<&'a WalletAccount> {
        w.accounts.iter()
            .find(|a| {
                let address_match = a.address == Some(*addr);
                address_match && a.blockchain == blockchain
            })
    }

    pub fn get_blockchain(&self) -> Blockchain {
        Blockchain::try_from(self.cfg.chain.unwrap()).expect("Unsupported chain")
    }

    pub fn load_wallets(&self) -> Vec<Wallet> {
        let storage = &self.cfg.get_storage();
        let wallets: Vec<Wallet> = storage.wallets().list().expect("Wallets are not loaded")
            .iter()
            .map(|id| storage.wallets().get(id))
            .map(|w| w.ok())
            .filter(|w| w.is_some())
            .map(|w| w.unwrap())
            .collect();
        wallets
    }


    pub fn get_wallet_by_addr(&self, addr: &Address) -> Option<Wallet> {
        let storage = &self.cfg.get_storage();

        let wallets = self.load_wallets();
        let wallet = wallets.iter()
            .find( |w| WrappedVault::find_account(w, addr, self.get_blockchain()).is_some());

        wallet.cloned()
    }

}