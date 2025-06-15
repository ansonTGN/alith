use alith_data::wallet::LocalEthWallet;
use alloy::{
    contract::{CallBuilder, CallDecoder},
    network::{Ethereum, EthereumWallet, TransactionBuilder, TransactionBuilderError},
    primitives::{Address, U256, keccak256},
    providers::Provider,
    rpc::types::TransactionReceipt,
    sol_types::SolValue,
};
use std::ops::Deref;
use thiserror::Error;

use crate::{
    ChainConfig, ChainError, ChainManager, Proof, ProofData, SettlementProof, SettlementProofData,
    Wallet, WalletError,
    chain::AlloyProvider,
    contracts::{
        Account, ContractConfig, DataAnchorToken::DataAnchorTokenInstance, FileResponse as File,
        IAIProcess::IAIProcessInstance, IDataRegistry::IDataRegistryInstance,
        ISettlement::ISettlementInstance, IVerifiedComputing::IVerifiedComputingInstance, Job,
        NodeInfo, Permission, User,
    },
};

#[derive(Debug, Clone)]
pub struct Client {
    manager: ChainManager,
    pub config: ContractConfig,
}

impl Client {
    /// New the LazAI client with the chain config and wallet.
    pub fn new(
        wallet: Wallet,
        chain_config: ChainConfig,
        contract_config: ContractConfig,
    ) -> Result<Self, ClientError> {
        let manager = ChainManager::new(chain_config, wallet)?;
        Ok(Self {
            manager,
            config: contract_config,
        })
    }

    /// New the default LazAI client.
    pub fn new_default() -> Result<Self, ClientError> {
        Ok(Self {
            manager: ChainManager::new_default()?,
            config: ContractConfig::default(),
        })
    }

    /// New the LazAI client for the local devnet.
    pub fn new_devnet() -> Result<Self, ClientError> {
        Ok(Self {
            manager: ChainManager::new(ChainConfig::local(), LocalEthWallet::from_env()?)?,
            config: ContractConfig::default(),
        })
    }

    /// Add privacy data url and encrypted key base64 format into the data registry on LazAI.
    pub async fn add_file(&self, url: impl AsRef<str>) -> Result<U256, ClientError> {
        let contract = self.data_registry_contract();
        self.send_transaction(contract.addFile(url.as_ref().to_string()), None)
            .await?;

        self.get_file_id_by_url(url).await
    }

    /// Add privacy data url into the data registry on LazAI.
    pub async fn add_file_with_permissions(
        &self,
        url: impl AsRef<str>,
        owner: Address,
        permissions: Vec<Permission>,
    ) -> Result<U256, ClientError> {
        let contract = self.data_registry_contract();
        self.send_transaction(
            contract.addFileWithPermissions(url.as_ref().to_string(), owner, permissions),
            None,
        )
        .await?;

        self.get_file_id_by_url(url).await
    }

    /// Add the permission for the file.
    pub async fn add_permission_for_file(
        &self,
        file_id: U256,
        permission: Permission,
    ) -> Result<TransactionReceipt, ClientError> {
        let contract = self.data_registry_contract();
        self.send_transaction(
            contract.addPermissionForFile(file_id, permission.account, permission.key),
            None,
        )
        .await
    }

    /// Get the file id by the url.
    pub async fn get_file_id_by_url(&self, url: impl AsRef<str>) -> Result<U256, ClientError> {
        let contract = self.data_registry_contract();
        let builder = self
            .call_builder(
                contract.getFileIdByUrl(url.as_ref().to_string()),
                self.config.data_registry_address,
                None,
            )
            .await?;
        let file_id = builder
            .call()
            .await
            .map_err(|err| ClientError::ContractCallError(err.to_string()))?;
        Ok(file_id)
    }

    /// Get the file information according to the file id on LazAI.
    pub async fn get_file(&self, file_id: U256) -> Result<File, ClientError> {
        let contract = self.data_registry_contract();
        let builder = self
            .call_builder(
                contract.getFile(file_id),
                self.config.data_registry_address,
                None,
            )
            .await?;
        let file = builder
            .call()
            .await
            .map_err(|err| ClientError::ContractCallError(err.to_string()))?;
        Ok(file)
    }

    /// Get the encryption key for the account on LazAI.
    pub async fn get_file_permission(
        &self,
        file_id: U256,
        account: Address,
    ) -> Result<String, ClientError> {
        let contract = self.data_registry_contract();
        let builder = self
            .call_builder(
                contract.getFilePermission(file_id, account),
                self.config.data_registry_address,
                None,
            )
            .await?;
        let key = builder
            .call()
            .await
            .map_err(|err| ClientError::ContractCallError(err.to_string()))?;
        Ok(key)
    }

    /// Get the file proof on LazAI.
    pub async fn get_file_proof(&self, file_id: U256, index: U256) -> Result<Proof, ClientError> {
        let contract = self.data_registry_contract();
        let builder = self
            .call_builder(
                contract.getFileProof(file_id, index),
                self.config.data_registry_address,
                None,
            )
            .await?;
        let proof = builder
            .call()
            .await
            .map_err(|err| ClientError::ContractCallError(err.to_string()))?;
        Ok(proof)
    }

    /// Get the file total count.
    pub async fn get_files_count(&self) -> Result<U256, ClientError> {
        let contract = self.data_registry_contract();
        let builder = self
            .call_builder(
                contract.filesCount(),
                self.config.data_registry_address,
                None,
            )
            .await?;
        let count = builder
            .call()
            .await
            .map_err(|err| ClientError::ContractCallError(err.to_string()))?;
        Ok(count)
    }

    pub async fn add_node(
        &self,
        address: Address,
        url: impl AsRef<str>,
        public_key: impl AsRef<str>,
    ) -> Result<TransactionReceipt, ClientError> {
        let contract = self.verified_computing_contract();
        self.send_transaction(
            contract.addNode(
                address,
                url.as_ref().to_string(),
                public_key.as_ref().to_string(),
            ),
            None,
        )
        .await
    }

    pub async fn remove_node(&self, address: Address) -> Result<TransactionReceipt, ClientError> {
        let contract = self.verified_computing_contract();
        self.send_transaction(contract.removeNode(address), None)
            .await
    }

    pub async fn get_node(&self, address: Address) -> Result<Option<NodeInfo>, ClientError> {
        let contract = self.verified_computing_contract();
        let builder = self
            .call_builder(
                contract.getNode(address),
                self.config.verified_computing_address,
                None,
            )
            .await?;
        let info = builder
            .call()
            .await
            .map_err(|err| ClientError::ContractCallError(err.to_string()))?;
        if info.url.is_empty() {
            Ok(None)
        } else {
            Ok(Some(info))
        }
    }

    /// Get the node address list
    pub async fn node_list(&self) -> Result<Vec<Address>, ClientError> {
        let contract = self.verified_computing_contract();
        let builder = self
            .call_builder(
                contract.nodeList(),
                self.config.verified_computing_address,
                None,
            )
            .await?;
        let list = builder
            .call()
            .await
            .map_err(|err| ClientError::ContractCallError(err.to_string()))?;
        Ok(list)
    }

    pub async fn update_node_fee(&self, fee: U256) -> Result<TransactionReceipt, ClientError> {
        let contract = self.verified_computing_contract();
        self.send_transaction(contract.updateNodeFee(fee), None)
            .await
    }

    pub async fn node_fee(&self) -> Result<U256, ClientError> {
        let contract = self.verified_computing_contract();
        let builder = self
            .call_builder(
                contract.nodeFee(),
                self.config.verified_computing_address,
                None,
            )
            .await?;
        let fee = builder
            .call()
            .await
            .map_err(|err| ClientError::ContractCallError(err.to_string()))?;
        Ok(fee)
    }

    /// Request a job to compute the proof with the given file id and value.
    /// Note that this function is a payable function, so you need to provide a value.
    pub async fn request_proof(
        &self,
        file_id: U256,
        value: U256,
    ) -> Result<TransactionReceipt, ClientError> {
        let contract = self.verified_computing_contract();
        self.send_transaction(contract.requestProof(file_id), Some(value))
            .await
    }

    pub async fn add_proof(
        &self,
        file_id: U256,
        data: ProofData,
    ) -> Result<TransactionReceipt, ClientError> {
        let message_hash = keccak256(data.abi_encode());
        let signature = format!(
            "0x{}",
            self.wallet
                .sign_message_hex(message_hash.as_slice())
                .await?
        );
        let proof = Proof {
            signature: signature.as_bytes().to_vec().into(),
            data,
        };
        let contract = self.data_registry_contract();
        self.send_transaction(contract.addProof(file_id, proof), None)
            .await
    }

    pub async fn complete_job(&self, job_id: U256) -> Result<TransactionReceipt, ClientError> {
        let contract = self.verified_computing_contract();
        self.send_transaction(contract.completeJob(job_id), None)
            .await
    }

    pub async fn get_job(&self, job_id: U256) -> Result<Job, ClientError> {
        let contract = self.verified_computing_contract();
        let builder = self
            .call_builder(
                contract.getJob(job_id),
                self.config.verified_computing_address,
                None,
            )
            .await?;
        let job = builder
            .call()
            .await
            .map_err(|err| ClientError::ContractCallError(err.to_string()))?;
        Ok(job)
    }

    pub async fn file_job_ids(&self, file_id: U256) -> Result<Vec<U256>, ClientError> {
        let contract = self.verified_computing_contract();
        let builder = self
            .call_builder(
                contract.fileJobIds(file_id),
                self.config.verified_computing_address,
                None,
            )
            .await?;
        let ids = builder
            .call()
            .await
            .map_err(|err| ClientError::ContractCallError(err.to_string()))?;
        Ok(ids)
    }

    pub async fn request_reward(
        &self,
        file_id: U256,
        proof_index: Option<U256>,
    ) -> Result<TransactionReceipt, ClientError> {
        let contract = self.data_registry_contract();
        // Get the first proof index if not provided.
        let proof_index = proof_index.unwrap_or(U256::from(1));
        self.send_transaction(contract.requestReward(file_id, proof_index), None)
            .await
    }

    /// Claim any rewards for node validators
    pub async fn claim(&self) -> Result<TransactionReceipt, ClientError> {
        let contract = self.verified_computing_contract();
        self.send_transaction(contract.claim(), None).await
    }

    /// Mint a new Data Anchor Token (DAT) with the specified parameters.
    pub async fn mint_dat(
        &self,
        to: Address,
        amount: U256,
        token_uri: String,
        verified: bool,
    ) -> Result<TransactionReceipt, ClientError> {
        let contract = self.data_anchor_token_contract();
        self.send_transaction(contract.mint(to, amount, token_uri, verified), None)
            .await
    }

    /// Returns the balance of a specific Data Anchor Token (DAT) for a given account and token ID.
    pub async fn get_dat_balance(&self, account: Address, id: U256) -> Result<U256, ClientError> {
        let contract = self.data_anchor_token_contract();
        let builder = self
            .call_builder(
                contract.balanceOf(account, id),
                self.config.data_anchor_token_address,
                None,
            )
            .await?;
        let balance = builder
            .call()
            .await
            .map_err(|err| ClientError::ContractCallError(err.to_string()))?;
        Ok(balance)
    }

    /// Returns the Uri for a specific Data Anchor Token (DAT) by its token ID.
    pub async fn dat_uri(&self, token_id: U256) -> Result<String, ClientError> {
        let contract = self.data_anchor_token_contract();
        let builder = self
            .call_builder(
                contract.uri(token_id),
                self.config.data_anchor_token_address,
                None,
            )
            .await?;
        let uri = builder
            .call()
            .await
            .map_err(|err| ClientError::ContractCallError(err.to_string()))?;
        Ok(uri)
    }

    /// Get the user for the settlement.
    pub async fn get_user(&self, user: Address) -> Result<User, ClientError> {
        let contract = self.settlement_contract();
        let builder = self
            .call_builder(contract.getUser(user), self.config.settlement_address, None)
            .await?;
        let user = builder
            .call()
            .await
            .map_err(|err| ClientError::ContractCallError(err.to_string()))?;
        Ok(user)
    }

    /// Get all users registered for the settlement.
    pub async fn get_all_users(&self) -> Result<Vec<User>, ClientError> {
        let contract = self.settlement_contract();
        let builder = self
            .call_builder(contract.getAllUsers(), self.config.settlement_address, None)
            .await?;
        let users = builder
            .call()
            .await
            .map_err(|err| ClientError::ContractCallError(err.to_string()))?;
        Ok(users)
    }

    pub async fn add_user(&self, amount: U256) -> Result<TransactionReceipt, ClientError> {
        let contract = self.settlement_contract();
        self.send_transaction(contract.addUser(), Some(amount))
            .await
    }

    pub async fn delete_user(&self) -> Result<TransactionReceipt, ClientError> {
        let contract = self.settlement_contract();
        self.send_transaction(contract.deleteUser(), None).await
    }

    pub async fn deposit(&self, amount: U256) -> Result<TransactionReceipt, ClientError> {
        let contract = self.settlement_contract();
        self.send_transaction(contract.deposit(), Some(amount))
            .await
    }

    pub async fn withdraw(&self, amount: U256) -> Result<TransactionReceipt, ClientError> {
        let contract = self.settlement_contract();
        self.send_transaction(contract.withdraw(amount), None).await
    }

    pub async fn deposit_training(
        &self,
        node: Address,
        amount: U256,
    ) -> Result<TransactionReceipt, ClientError> {
        let contract = self.settlement_contract();
        self.send_transaction(contract.depositTraining(node, amount), None)
            .await
    }

    pub async fn deposit_inference(
        &self,
        node: Address,
        amount: U256,
    ) -> Result<TransactionReceipt, ClientError> {
        let contract = self.settlement_contract();
        self.send_transaction(contract.depositInference(node, amount), None)
            .await
    }

    pub async fn retrieve_training(
        &self,
        nodes: Vec<Address>,
    ) -> Result<TransactionReceipt, ClientError> {
        let contract = self.settlement_contract();
        self.send_transaction(contract.retrieveTraining(nodes), None)
            .await
    }

    pub async fn retrieve_inference(
        &self,
        nodes: Vec<Address>,
    ) -> Result<TransactionReceipt, ClientError> {
        let contract = self.settlement_contract();
        self.send_transaction(contract.retrieveInference(nodes), None)
            .await
    }

    pub async fn add_inference_node(
        &self,
        address: Address,
        url: impl AsRef<str>,
        public_key: impl AsRef<str>,
    ) -> Result<TransactionReceipt, ClientError> {
        let contract = self.inference_contract();
        self.send_transaction(
            contract.addNode(
                address,
                url.as_ref().to_string(),
                public_key.as_ref().to_string(),
            ),
            None,
        )
        .await
    }

    pub async fn remove_inference_node(
        &self,
        address: Address,
    ) -> Result<TransactionReceipt, ClientError> {
        let contract = self.inference_contract();
        self.send_transaction(contract.removeNode(address), None)
            .await
    }

    pub async fn get_inference_node(
        &self,
        address: Address,
    ) -> Result<Option<NodeInfo>, ClientError> {
        let contract = self.inference_contract();
        let builder = self
            .call_builder(
                contract.getNode(address),
                self.config.inference_address,
                None,
            )
            .await?;
        let info = builder
            .call()
            .await
            .map_err(|err| ClientError::ContractCallError(err.to_string()))?;
        if info.url.is_empty() {
            Ok(None)
        } else {
            Ok(Some(info))
        }
    }

    /// Get the inference node address list
    pub async fn inference_node_list(&self) -> Result<Vec<Address>, ClientError> {
        let contract = self.inference_contract();
        let builder = self
            .call_builder(contract.nodeList(), self.config.inference_address, None)
            .await?;
        let list = builder
            .call()
            .await
            .map_err(|err| ClientError::ContractCallError(err.to_string()))?;
        Ok(list)
    }

    pub async fn get_inference_account(
        &self,
        user: Address,
        node: Address,
    ) -> Result<Account, ClientError> {
        let contract = self.inference_contract();
        let builder = self
            .call_builder(
                contract.getAccount(user, node),
                self.config.inference_address,
                None,
            )
            .await?;
        let list = builder
            .call()
            .await
            .map_err(|err| ClientError::ContractCallError(err.to_string()))?;
        Ok(list)
    }

    pub async fn add_training_node(
        &self,
        address: Address,
        url: impl AsRef<str>,
        public_key: impl AsRef<str>,
    ) -> Result<TransactionReceipt, ClientError> {
        let contract = self.training_contract();
        self.send_transaction(
            contract.addNode(
                address,
                url.as_ref().to_string(),
                public_key.as_ref().to_string(),
            ),
            None,
        )
        .await
    }

    pub async fn remove_training_node(
        &self,
        address: Address,
    ) -> Result<TransactionReceipt, ClientError> {
        let contract = self.training_contract();
        self.send_transaction(contract.removeNode(address), None)
            .await
    }

    pub async fn get_training_node(
        &self,
        address: Address,
    ) -> Result<Option<NodeInfo>, ClientError> {
        let contract = self.training_contract();
        let builder = self
            .call_builder(
                contract.getNode(address),
                self.config.training_address,
                None,
            )
            .await?;
        let info = builder
            .call()
            .await
            .map_err(|err| ClientError::ContractCallError(err.to_string()))?;
        if info.url.is_empty() {
            Ok(None)
        } else {
            Ok(Some(info))
        }
    }

    /// Get the training node address list
    pub async fn training_node_list(&self) -> Result<Vec<Address>, ClientError> {
        let contract = self.training_contract();
        let builder = self
            .call_builder(contract.nodeList(), self.config.training_address, None)
            .await?;
        let list = builder
            .call()
            .await
            .map_err(|err| ClientError::ContractCallError(err.to_string()))?;
        Ok(list)
    }

    pub async fn get_training_account(
        &self,
        user: Address,
        node: Address,
    ) -> Result<Account, ClientError> {
        let contract = self.training_contract();
        let builder = self
            .call_builder(
                contract.getAccount(user, node),
                self.config.training_address,
                None,
            )
            .await?;
        let list = builder
            .call()
            .await
            .map_err(|err| ClientError::ContractCallError(err.to_string()))?;
        Ok(list)
    }

    pub async fn inference_settlement_fees(
        &self,
        data: SettlementProofData,
    ) -> Result<TransactionReceipt, ClientError> {
        let contract = self.inference_contract();
        let message_hash = keccak256(data.abi_encode());
        let signature = format!(
            "0x{}",
            self.wallet
                .sign_message_hex(message_hash.as_slice())
                .await?
        );
        let proof = SettlementProof {
            signature: signature.as_bytes().to_vec().into(),
            data,
        };

        self.send_transaction(contract.settlementFees(proof), None)
            .await
    }

    pub async fn training_settlement_fees(
        &self,
        data: SettlementProofData,
    ) -> Result<TransactionReceipt, ClientError> {
        let contract = self.training_contract();
        let message_hash = keccak256(data.abi_encode());
        let signature = format!(
            "0x{}",
            self.wallet
                .sign_message_hex(message_hash.as_slice())
                .await?
        );
        let proof = SettlementProof {
            signature: signature.as_bytes().to_vec().into(),
            data,
        };

        self.send_transaction(contract.settlementFees(proof), None)
            .await
    }

    #[inline]
    async fn call_builder<'a, D: CallDecoder>(
        &'a self,
        call_builder: CallBuilder<&'a &'a AlloyProvider, D>,
        to: Address,
        value: Option<U256>,
    ) -> Result<CallBuilder<&'a &'a AlloyProvider, D>, ClientError> {
        Ok(call_builder
            .from(self.wallet.address)
            .to(to)
            .chain_id(self.manager.config.chain_id)
            .value(value.unwrap_or_default())
            .nonce(self.get_nonce().await?))
    }

    async fn send_transaction<'a, D: CallDecoder>(
        &'a self,
        call_builder: CallBuilder<&'a &'a AlloyProvider, D>,
        value: Option<U256>,
    ) -> Result<TransactionReceipt, ClientError> {
        let builder = call_builder
            .from(self.wallet.address)
            .chain_id(self.manager.config.chain_id)
            .value(value.unwrap_or_default())
            .nonce(self.get_nonce().await?);
        let tx = builder.into_transaction_request();
        let gas_limit = self.estimate_tx_gas(tx.clone()).await?;
        let gas_price = self.get_gas_price().await?;
        let priority_fee = self.get_max_priority_fee_per_gas().await?;
        let tx = tx
            .with_gas_limit(gas_limit)
            .with_max_fee_per_gas(gas_price)
            .with_max_priority_fee_per_gas(priority_fee);

        let wallet = EthereumWallet::new(self.wallet.signer.clone());
        let tx_envelope = tx
            .build(&wallet)
            .await
            .map_err(|err| ChainError::SigningError(err.to_string()))?;

        self.provider
            .send_tx_envelope(tx_envelope)
            .await
            .map_err(Into::<ChainError>::into)?
            .get_receipt()
            .await
            .map_err(|err| ClientError::TransactionError(err.to_string()))
    }

    #[inline]
    fn data_registry_contract(&self) -> IDataRegistryInstance<&AlloyProvider> {
        IDataRegistryInstance::new(self.config.data_registry_address, &self.manager.provider)
    }

    #[inline]
    fn verified_computing_contract(&self) -> IVerifiedComputingInstance<&AlloyProvider> {
        IVerifiedComputingInstance::new(
            self.config.verified_computing_address,
            &self.manager.provider,
        )
    }

    #[inline]
    fn data_anchor_token_contract(&self) -> DataAnchorTokenInstance<&AlloyProvider> {
        DataAnchorTokenInstance::new(
            self.config.data_anchor_token_address,
            &self.manager.provider,
        )
    }

    #[inline]
    fn inference_contract(&self) -> IAIProcessInstance<&AlloyProvider> {
        IAIProcessInstance::new(self.config.inference_address, &self.manager.provider)
    }

    #[inline]
    fn training_contract(&self) -> IAIProcessInstance<&AlloyProvider> {
        IAIProcessInstance::new(self.config.training_address, &self.manager.provider)
    }

    #[inline]
    fn settlement_contract(&self) -> ISettlementInstance<&AlloyProvider> {
        ISettlementInstance::new(self.config.settlement_address, &self.manager.provider)
    }
}

impl AsRef<ChainManager> for Client {
    fn as_ref(&self) -> &ChainManager {
        &self.manager
    }
}

impl AsMut<ChainManager> for Client {
    fn as_mut(&mut self) -> &mut ChainManager {
        &mut self.manager
    }
}

impl Deref for Client {
    type Target = ChainManager;

    fn deref(&self) -> &Self::Target {
        &self.manager
    }
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Chain error: {0}")]
    ChainError(#[from] ChainError),
    #[error("Tx build error: {0}")]
    TxBuildError(#[from] TransactionBuilderError<Ethereum>),
    #[error("Wallet error: {0}")]
    WalletError(#[from] WalletError),
    #[error("Contract call error: {0}")]
    ContractCallError(String),
    #[error("Signing error: {0}")]
    SigningError(String),
    #[error("Transaction error: {0}")]
    TransactionError(String),
}
