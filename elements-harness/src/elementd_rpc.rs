use anyhow::{bail, Context, Result};
use elements_fun::{
    bitcoin::Amount, bitcoin_hashes::hex::FromHex, encode::serialize_hex, secp256k1::SecretKey,
    Address, AssetId, ExplicitAsset, ExplicitTxOut, ExplicitValue, OutPoint, Transaction, TxOut,
    Txid,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[jsonrpc_client::api(version = "1.0")]
pub trait ElementsRpc {
    async fn getblockchaininfo(&self) -> BlockchainInfo;
    async fn getnewaddress(&self) -> Address;
    #[allow(clippy::too_many_arguments)]
    async fn sendtoaddress(
        &self,
        address: &Address,
        amount: f64,
        comment: Option<String>,
        comment_to: Option<String>,
        subtract_fee_from_amount: Option<bool>,
        replaceable: Option<bool>,
        conf_target: Option<u64>,
        estimate_mode: Option<String>,
        asset_id: Option<AssetId>,
        ignore_blind_fail: bool,
    ) -> Txid;
    async fn dumpassetlabels(&self) -> HashMap<String, AssetId>;
    async fn getrawtransaction(&self, txid: Txid) -> String;
    async fn sendrawtransaction(&self, tx_hex: String) -> Txid;
    async fn issueasset(
        &self,
        asset_amount: f64,
        token_amount: f64,
        blind: bool,
    ) -> IssueAssetResponse;
    async fn getbalance(
        &self,
        dummy: Option<String>,
        minconf: Option<u64>,
        include_watchonly: Option<bool>,
        asset_id: Option<AssetId>,
    ) -> f64;
    async fn fundrawtransaction(&self, tx_hex: String) -> FundRawTransactionResponse;
    async fn dumpblindingkey(&self, address: &Address) -> SecretKey;
    async fn listunspent(
        &self,
        minconf: Option<u64>,
        maxconf: Option<u64>,
        addresses: Option<&[Address]>,
        include_unsafe: Option<bool>,
        query_options: Option<ListUnspentOptions>,
    ) -> Vec<UtxoInfo>;
    async fn signrawtransactionwithwallet(
        &self,
        tx_hex: String,
    ) -> SignRawTransactionWithWalletResponse;
    async fn generatetoaddress(&self, nblocks: u32, address: &Address) -> Vec<String>;
    async fn dumpmasterblindingkey(&self) -> String;
    async fn unblindrawtransaction(&self, tx_hex: String) -> UnblindRawTransactionResponse;
    async fn lockunspent(&self, unlock: bool, utxos: Vec<OutPoint>) -> bool;
    async fn reissueasset(&self, asset: AssetId, amount: f64) -> ReissueAssetResponse;
}

#[jsonrpc_client::implement(ElementsRpc)]
#[derive(Clone, Debug)]
pub struct Client {
    inner: reqwest::Client,
    base_url: reqwest::Url,
}

#[derive(Debug, Deserialize)]
pub struct UnblindRawTransactionResponse {
    hex: String,
}

#[derive(Debug, Deserialize)]
pub struct SignRawTransactionWithWalletResponse {
    hex: String,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct ReissueAssetResponse {
    txid: Txid,
    vin: u8,
}

impl Client {
    pub fn new(base_url: String) -> Result<Self> {
        Ok(Self {
            inner: reqwest::Client::new(),
            base_url: base_url.parse()?,
        })
    }

    pub async fn get_bitcoin_asset_id(&self) -> Result<AssetId> {
        let labels = self.dumpassetlabels().await?;
        let bitcoin_asset_tag = "bitcoin";
        let bitcoin_asset_id = labels
            .get(bitcoin_asset_tag)
            .context("failed to get asset id for bitcoin")?;

        Ok(*bitcoin_asset_id)
    }

    pub async fn send_asset_to_address(
        &self,
        address: &Address,
        amount: Amount,
        asset_id: Option<AssetId>,
    ) -> Result<Txid> {
        let txid = self
            .sendtoaddress(
                address,
                amount.as_btc(),
                None,
                None,
                None,
                None,
                None,
                None,
                asset_id,
                true,
            )
            .await?;

        Ok(txid)
    }

    pub async fn get_raw_transaction(&self, txid: Txid) -> Result<Transaction> {
        let tx_hex = self.getrawtransaction(txid).await?;
        let tx = elements_fun::encode::deserialize(&Vec::<u8>::from_hex(&tx_hex).unwrap())?;

        Ok(tx)
    }

    pub async fn send_raw_transaction(&self, tx: &Transaction) -> Result<Txid> {
        let tx_hex = serialize_hex(tx);
        let txid = self.sendrawtransaction(tx_hex).await?;
        Ok(txid)
    }

    pub async fn unblind_raw_transaction(&self, tx: &Transaction) -> Result<Transaction> {
        let tx_hex = serialize_hex(tx);
        let res = self.unblindrawtransaction(tx_hex).await?;
        let tx = elements_fun::encode::deserialize(&Vec::<u8>::from_hex(&res.hex).unwrap())?;

        Ok(tx)
    }

    /// Use elementsd's coin selection algorithm to find a set of
    /// UTXOs which can pay for an output of type `asset ` and value
    /// `amount`.
    ///
    /// If `should_lock` is set to true, all selected UTXOs will be
    /// exempt from being chosen again unless explicitly unlocked or
    /// after the elementsd node has been restarded.
    pub async fn select_inputs_for(
        &self,
        asset: AssetId,
        amount: Amount,
        should_lock: bool,
    ) -> Result<Vec<(OutPoint, TxOut)>> {
        let placeholder_address = self.getnewaddress().await.unwrap();
        let tx = Transaction {
            output: vec![TxOut::Explicit(ExplicitTxOut {
                asset: ExplicitAsset(asset),
                value: ExplicitValue(amount.as_sat()),
                script_pubkey: placeholder_address.script_pubkey(),
                nonce: None,
            })],
            ..Default::default()
        };

        let tx_hex = serialize_hex(&tx);
        let res = self
            .fundrawtransaction(tx_hex)
            .await
            .context("cannot fund raw transaction")?;

        let tx: Transaction =
            elements_fun::encode::deserialize(&Vec::<u8>::from_hex(&res.hex).unwrap())
                .context("cannot deserialize funded transaction")?;

        let mut utxos = Vec::new();
        for input in tx.input.iter() {
            let source_txid = input.previous_output.txid;
            let source_vout = input.previous_output.vout;
            let source_tx = self
                .get_raw_transaction(source_txid)
                .await
                .context("cannot get raw source transaction")?;

            let unblinded_raw_tx = self
                .unblind_raw_transaction(&source_tx)
                .await
                .context("cannot unblind raw source transaction")?;
            if unblinded_raw_tx.output[source_vout as usize]
                .as_explicit()
                .expect("explicit output")
                .asset
                .0
                == asset
            {
                let source_txout = source_tx.output[input.previous_output.vout as usize].clone();

                utxos.push((
                    OutPoint {
                        txid: source_txid,
                        vout: source_vout,
                    },
                    source_txout,
                ))
            }
        }

        if should_lock {
            self.lock_utxos(utxos.iter().map(|(utxo, _)| *utxo).collect())
                .await?;
        }

        Ok(utxos)
    }

    pub async fn sign_raw_transaction(&self, tx: &Transaction) -> Result<Transaction> {
        let tx_hex = serialize_hex(tx);
        let res = self.signrawtransactionwithwallet(tx_hex).await?;
        let tx = elements_fun::encode::deserialize(&Vec::<u8>::from_hex(&res.hex).unwrap())?;

        Ok(tx)
    }

    pub async fn lock_utxos(&self, utxos: Vec<OutPoint>) -> Result<()> {
        let res = self.lockunspent(false, utxos).await?;

        if res {
            Ok(())
        } else {
            bail!("Could not lock outputs")
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BlockchainInfo {
    pub chain: String,
    mediantime: u32,
}

#[derive(Debug, Deserialize)]
pub struct IssueAssetResponse {
    pub txid: Txid,
    pub vin: u8,
    pub entropy: String,
    pub asset: AssetId,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct FundRawTransactionResponse {
    pub hex: String,
    pub fee: f64,
    pub changepos: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Default)]
pub struct ListUnspentOptions {
    pub minimum_amount: Option<u64>,
    pub max_amount: Option<u64>,
    pub maximum_count: Option<u64>,
    pub minimum_sum_amount: Option<u64>,
    pub asset: Option<AssetId>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UtxoInfo {
    pub txid: Txid,
    pub vout: u32,
    pub address: Option<Address>,
    pub spendable: bool,
    pub amount: f64,
}

#[cfg(all(test))]
mod test {
    use super::*;
    use crate::Elementsd;
    use testcontainers::clients::Cli;

    #[tokio::test]
    async fn get_network_info() {
        let tc_client = Cli::default();
        let (client, _container) = {
            let blockchain = Elementsd::new(&tc_client, "0.18.1.9").unwrap();
            (
                Client::new(blockchain.node_url.clone().into_string()).unwrap(),
                blockchain,
            )
        };

        let blockchain_info: BlockchainInfo = client.getblockchaininfo().await.unwrap();
        let network = blockchain_info.chain;

        assert_eq!(network, "elementsregtest")
    }

    #[tokio::test]
    async fn send_to_generated_address() {
        let tc_client = Cli::default();
        let (client, _container) = {
            let blockchain = Elementsd::new(&tc_client, "0.18.1.9").unwrap();

            (
                Client::new(blockchain.node_url.clone().into_string()).unwrap(),
                blockchain,
            )
        };

        let address = client.getnewaddress().await.unwrap();
        let _txid = client
            .sendtoaddress(
                &address, 1.0, None, None, None, None, None, None, None, true,
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn dump_labels() {
        let tc_client = Cli::default();
        let (client, _container) = {
            let blockchain = Elementsd::new(&tc_client, "0.18.1.9").unwrap();

            (
                Client::new(blockchain.node_url.clone().into_string()).unwrap(),
                blockchain,
            )
        };

        let _labels = client.dumpassetlabels().await.unwrap();
    }

    #[tokio::test]
    async fn issue_asset() {
        let tc_client = Cli::default();
        let (client, _container) = {
            let blockchain = Elementsd::new(&tc_client, "0.18.1.9").unwrap();

            (
                Client::new(blockchain.node_url.clone().into_string()).unwrap(),
                blockchain,
            )
        };

        let expected_balance = 0.1;

        let asset_id = client
            .issueasset(expected_balance, 0.0, true)
            .await
            .unwrap()
            .asset;
        let balance = client
            .getbalance(None, None, None, Some(asset_id))
            .await
            .unwrap();

        let error_margin = f64::EPSILON;

        assert!((balance - expected_balance).abs() < error_margin)
    }

    #[tokio::test]
    async fn find_inputs_for() {
        let tc_client = Cli::default();
        let (client, _container) = {
            let blockchain = Elementsd::new(&tc_client, "0.18.1.9").unwrap();

            (
                Client::new(blockchain.node_url.clone().into_string()).unwrap(),
                blockchain,
            )
        };

        let labels = client.dumpassetlabels().await.unwrap();
        let _tx = client
            .select_inputs_for(*labels.get("bitcoin").unwrap(), Amount::ONE_BTC, false)
            .await
            .unwrap();
    }
}
