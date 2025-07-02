use alloy::{
    primitives::{Address, address},
    sol,
};

sol! {
    event FileAdded(uint256 indexed fileId, address indexed ownerAddress, string url);
    event ProofAdded(uint256 indexed fileId, address indexed ownerAddress, uint256 proofIndex, string proofUrl);
    event PermissionGranted(uint256 indexed fileId, address indexed account);

    event RewardRequested(
        address indexed contributorAddress, uint256 indexed fileId, uint256 indexed proofIndex, uint256 rewardAmount
    );

    event PublicKeyUpdated(string newPublicKey);

    event TokenUpdated(address newToken);
    event VerifiedComputingUpdated(address newVerifiedComputing);

    error NotFileOwner();
    error FileUrlAlreadyUsed();
    error FileNotFound();
    error FileAlreadyRewarded();
    error NoPermission();
    error InvalidUrl();
    error InvalidAttestator(bytes32 messageHash, bytes signature, address signer);

    struct ProofData {
        uint256 id;
        uint256 score;
        string fileUrl;
        string proofUrl;
    }

    struct Proof {
        bytes signature;
        ProofData data;
    }

    struct Permission {
        address account;
        string key;
    }

    // `alloy::sol!`` does support solidity mapping here
    // struct File {
    //     uint256 id;
    //     address ownerAddress;
    //     string url;
    //     uint256 timestamp;
    //     uint256 proofIndex;
    //     uint256 proofsCount;
    //     uint256 rewardAmount;
    //     mapping(uint256 proofId => Proof proof) proofs;
    //     mapping(address account => string key) permissions;
    // }

    struct FileResponse {
        uint256 id;
        address ownerAddress;
        string url;
        string hash;
        uint256 proofIndex;
        uint256 rewardAmount;
    }

    // Data registry contract interface

    #[sol(rpc)]
    interface IDataRegistry {
        function name() external view returns (string memory);
        function version() external pure returns (uint256);
        function token() external view returns (DataAnchoringToken);
        function verifiedComputing() external view returns (IVerifiedComputing);
        function updateVerifiedComputing(address newVerifiedComputing) external;

        // Public key operations

        function publicKey() external view returns (string memory);
        function updatePublicKey(string calldata newPublicKey) external;

        // Privacy data and file operations

        function addFile(string memory url, string memory hash) external returns (uint256);
        function addFileWithPermissions(
            string memory url,
            string memory hash,
            address ownerAddress,
            Permission[] memory permissions
        ) external returns (uint256);
        function addPermissionForFile(uint256 fileId, address account, string memory key) external;

        // File view functions

        function getFile(uint256 fileId) external view returns (FileResponse memory);
        function getFileIdByUrl(string memory url) external view returns (uint256);
        function getFilePermission(uint256 fileId, address account) external view returns (string memory);
        function getFileProof(uint256 fileId, uint256 index) external view returns (Proof memory);
        function filesCount() external view returns (uint256);

        // Proof operations

        function addProof(uint256 fileId, Proof memory proof) external;

        // Request reward and token
        function requestReward(uint256 fileId, uint256 proofIndex) external;
    }

    event NodeAdded(address indexed nodeAddress);
    event NodeRemoved(address indexed nodeAddress);

    event JobSubmitted(uint256 indexed jobId, uint256 indexed fileId, address nodeAddress, uint256 bidAmount);
    event JobCanceled(uint256 indexed jobId);

    event JobComplete(address indexed attestator, uint256 indexed jobId, uint256 indexed fileId);
    event Claimed(address indexed nodeAddress, uint256 amount);

    error NodeAlreadyAdded();
    error NodeNotActive();
    error InvalidJobStatus();
    error InvalidJobNode();
    error NothingToClaim();
    error InsufficientFee();
    error NoActiveNode();
    error NotJobOwner();
    error TransferFailed();

    enum NodeStatus {
        None,
        Active,
        Removed
    }

    struct NodeInfo {
        address nodeAddress;
        string url;
        NodeStatus status;
        uint256 amount;
        uint256 jobsCount;
        string publicKey;
    }

    enum JobStatus {
        None,
        Submitted,
        Completed,
        Canceled
    }

    struct Job {
        uint256 fileId;
        uint256 bidAmount;
        JobStatus status;
        uint256 addedTimestamp;
        address ownerAddress;
        address nodeAddress;
    }

    struct Account {
        address user;
        address node;
        uint256 nonce;
        uint256 balance;
        uint256 pendingRefund;
        Refund[] refunds;
    }

    struct Refund {
        uint256 index;
        uint256 amount;
        uint256 createdAt;
        bool processed;
    }

    struct SettlementData {
        // Use string here and sync with the chat/training id.
        string id;
        address user;
        uint256 cost;
        uint256 nonce;
        bytes userSignature;
    }

    struct Settlement {
        bytes signature;
        SettlementData data;
    }

    struct User {
        address addr;
        uint256 availableBalance;
        uint256 totalBalance;
        address[] queryNodes;
        address[] inferenceNodes;
        address[] trainingNodes;
    }

    // Verified Computing Contract for privacy data proof, query, inference and training in CPU/GPU TEE.

    #[sol(rpc)]
    interface IVerifiedComputing {
        function version() external pure returns (uint256);

        // Fee operations

        function nodeFee() external view returns (uint256);
        function updateNodeFee(uint256 newNodeFee) external;

        // Node operations

        function nodeList() external view returns (address[] memory);
        function nodeListAt(uint256 index) external view returns (NodeInfo memory);
        function nodesCount() external view returns (uint256);

        function activeNodesCount() external view returns (uint256);
        function activeNodeList() external view returns (address[] memory);
        function activeNodeListAt(uint256 index) external view returns (NodeInfo memory);

        function getNode(address nodeAddress) external view returns (NodeInfo memory);
        function addNode(address nodeAddress, string memory url, string memory publicKey) external;
        function removeNode(address nodeAddress) external;
        function isNode(address nodeAddress) external view returns (bool);

        function claim() external;

        function requestProof(uint256 fileId) external payable;

        function pause() external;
        function unpause() external;

        function submitJob(uint256 fileId) external payable;
        function completeJob(uint256 jobId) external;
        function fileJobIds(uint256 fileId) external view returns (uint256[] memory);
        function jobsCount() external view returns (uint256);
        function getJob(uint256 jobId) external view returns (Job memory);
    }

    // DAT Token

    interface IERC1155 {
        event TransferSingle(address indexed operator, address indexed from, address indexed to, uint256 id, uint256 value);
        event TransferBatch(
            address indexed operator,
            address indexed from,
            address indexed to,
            uint256[] ids,
            uint256[] values
        );
        event ApprovalForAll(address indexed account, address indexed operator, bool approved);
        event URI(string value, uint256 indexed id);

        function balanceOf(address account, uint256 id) external view returns (uint256);
        function balanceOfBatch(
            address[] calldata accounts,
            uint256[] calldata ids
        ) external view returns (uint256[] memory);
        function setApprovalForAll(address operator, bool approved) external;
        function isApprovedForAll(address account, address operator) external view returns (bool);
        function safeTransferFrom(address from, address to, uint256 id, uint256 value, bytes calldata data) external;
        function safeBatchTransferFrom(
            address from,
            address to,
            uint256[] calldata ids,
            uint256[] calldata values,
            bytes calldata data
        ) external;
    }

    #[sol(rpc)]
    contract DataAnchoringToken is IERC1155 {
        event TokenMinted(address indexed to, uint256 indexed tokenId, string tokenURI);

        function mint(address to, uint256 amount, string memory tokenURI_, bool verified_) public;
        function uri(uint256 tokenId) public view override returns (string memory);
        function verified(uint256 tokenId) public view returns (bool);
        function setTokenVerified(uint256 tokenId, bool verified_);
        function balanceOf(address account, uint256 id) external view returns (uint256);
        function batchMint(address to, uint256[] memory ids, uint256[] memory amounts, string[] memory tokenURIs) public external;
    }

    #[sol(rpc)]
    interface IAIProcess {
        function version() external pure returns (uint256);

        function pause() external;
        function unpause() external;

        // Node operations

        function nodeList() external view returns (address[] memory);
        function nodeListAt(uint256 index) external view returns (NodeInfo memory);
        function nodesCount() external view returns (uint256);

        function activeNodesCount() external view returns (uint256);
        function activeNodeList() external view returns (address[] memory);
        function activeNodeListAt(uint256 index) external view returns (NodeInfo memory);

        function getNode(address nodeAddress) external view returns (NodeInfo memory);
        function addNode(address nodeAddress, string memory url, string memory publicKey) external;
        function removeNode(address nodeAddress) external;
        function isNode(address nodeAddress) external view returns (bool);

        // Settlement account and operations

        function settlement() external view returns (ISettlement);
        function updateSettlement(address newSettlement) external;

        function getAccount(address user, address node) external view returns (Account memory);
        function getAccountPendingRefund(address user, address node) external view returns (uint256);
        function getAllAccounts() external view returns (Account[] memory accounts);
        function accountExists(address user, address node) external view returns (bool);
        function addAccount(address user, address node) external payable;
        function deleteAccount(address user, address node) external;

        function deposit(address user, address node, uint256 cancelRetrievingAmount) external payable;
        function request(address user, address node) external;
        function process(address user, address node)
            external
            returns (uint256 totalAmount, uint256 balance, uint256 pendingRefund);

        function settlementFees(Settlement memory settlement) external;
    }

    #[sol(rpc)]
    interface ISettlement {
        function version() external pure returns (uint256);
        function query() external view returns (IAIProcess);
        function updateQuery(address newTraining) external;
        function inference() external view returns (IAIProcess);
        function updateInference(address newInference) external;
        function training() external view returns (IAIProcess);
        function updateTraining(address newTraining) external;

        function pause() external;
        function unpause() external;

        function getUser(address user) external view returns (User memory);
        function getAllUsers() external view returns (User[] memory users);
        function addUser() external payable;
        function deleteUser() external;
        function deposit() external payable;
        function withdraw(uint256 amount) external;

        function depositQuery(address node, uint256 amount) external;
        function depositInference(address node, uint256 amount) external;
        function depositTraining(address node, uint256 amount) external;

        function retrieveQuery(address[] memory nodes) external;
        function retrieveInference(address[] memory nodes) external;
        function retrieveTraining(address[] memory nodes) external;

        function settlement(address addr, uint256 cost) external;
    }
}

// Local Devnet Contract addresses

pub const DEFAULT_DATA_ANCHORING_TOKEN_CONTRACT_ADDRESS: Address =
    address!("0x2eD344c586303C98FC3c6D5B42C5616ED42f9D9d");
pub const DEFAULT_VERIFIED_COMPUTING_CONTRACT_ADDRESS: Address =
    address!("0x815da22D880E3560bCEcc85b6e4938b30c8202C4");
pub const DEFAULT_DATA_REGISTRY_CONTRACT_ADDRESS: Address =
    address!("0xEAd077726dC83ecF385e3763ed4A0A50E8Ac5AA0");
pub const DEFAULT_QUERY_CONTRACT_ADDRESS: Address =
    address!("0xE747fd70269a8a540403ddE802D6906CB18C9F50");
pub const DEFAULT_INFERENCE_CONTRACT_ADDRESS: Address =
    address!("0xbb969eaafB3A7124b8dCdf9a6d5Cd5BAa0381361");
pub const DEFAULT_TRAINING_CONTRACT_ADDRESS: Address =
    address!("0xb578AB78bb4780D9007Cc836b358468467814B3E");
pub const DEFAULT_SETTLEMENT_CONTRACT_ADDRESS: Address =
    address!("0xBE94646A0C6C1032c289Eea47169798e09dB5299");
pub const DEFAULT_LAZAI_IDAO_CONTRACT_ADDRESS: Address =
    address!("0xEA30BA91F4DB33Ef0360Fc04d8E201954474dbD1");

// Testnet Contract addresses

pub const TESTNET_ADMIN_ADDRESS: Address = address!("0x34d9E02F9bB4E4C8836e38DF4320D4a79106F194");
pub const TESTNET_DATA_REGISTRY_CONTRACT_ADDRESS: Address =
    address!("0xE7753EeBCA82849D6b19E6689B350f87318A8998");
pub const TESTNET_VERIFIED_COMPUTING_CONTRACT_ADDRESS: Address =
    address!("0x87E43F24Efc2284fd2BBF11CC80d6fcF3E0AD474");
pub const TESTNET_DATA_ANCHORING_TOKEN_CONTRACT_ADDRESS: Address =
    address!("0xD59CDFFEb65aCc539994e41D0B40efF61bE37118");
pub const TESTNET_QUERY_CONTRACT_ADDRESS: Address =
    address!("0x5D7fC5A04328b95cae017B664A8e95fa25Ca3e98");
pub const TESTNET_INFERENCE_CONTRACT_ADDRESS: Address =
    address!("0x69BD47252510573995b22ae227560E879b193738");
pub const TESTNET_TRAINING_CONTRACT_ADDRESS: Address =
    address!("0x7e6646feEC69df501D942e16CBE2d14B7bBEC853");
pub const TESTNET_SETTLEMENT_CONTRACT_ADDRESS: Address =
    address!("0xF1398c4Bb36245750393A2511dA8bF1F7828F979");

#[derive(Debug, Clone)]
pub struct ContractConfig {
    pub data_registry_address: Address,
    pub verified_computing_address: Address,
    pub data_anchoring_token_address: Address,
    pub query_address: Address,
    pub inference_address: Address,
    pub training_address: Address,
    pub settlement_address: Address,
}

impl Default for ContractConfig {
    fn default() -> Self {
        Self {
            data_registry_address: DEFAULT_DATA_REGISTRY_CONTRACT_ADDRESS,
            verified_computing_address: DEFAULT_VERIFIED_COMPUTING_CONTRACT_ADDRESS,
            data_anchoring_token_address: DEFAULT_DATA_ANCHORING_TOKEN_CONTRACT_ADDRESS,
            query_address: DEFAULT_QUERY_CONTRACT_ADDRESS,
            inference_address: DEFAULT_INFERENCE_CONTRACT_ADDRESS,
            training_address: DEFAULT_TRAINING_CONTRACT_ADDRESS,
            settlement_address: DEFAULT_SETTLEMENT_CONTRACT_ADDRESS,
        }
    }
}

impl ContractConfig {
    pub fn local() -> Self {
        Self {
            data_registry_address: DEFAULT_DATA_REGISTRY_CONTRACT_ADDRESS,
            verified_computing_address: DEFAULT_VERIFIED_COMPUTING_CONTRACT_ADDRESS,
            data_anchoring_token_address: DEFAULT_DATA_ANCHORING_TOKEN_CONTRACT_ADDRESS,
            query_address: DEFAULT_QUERY_CONTRACT_ADDRESS,
            inference_address: DEFAULT_INFERENCE_CONTRACT_ADDRESS,
            training_address: DEFAULT_TRAINING_CONTRACT_ADDRESS,
            settlement_address: DEFAULT_SETTLEMENT_CONTRACT_ADDRESS,
        }
    }

    pub fn testnet() -> Self {
        Self {
            data_registry_address: TESTNET_DATA_REGISTRY_CONTRACT_ADDRESS,
            verified_computing_address: TESTNET_VERIFIED_COMPUTING_CONTRACT_ADDRESS,
            data_anchoring_token_address: TESTNET_DATA_ANCHORING_TOKEN_CONTRACT_ADDRESS,
            query_address: TESTNET_QUERY_CONTRACT_ADDRESS,
            inference_address: TESTNET_INFERENCE_CONTRACT_ADDRESS,
            training_address: TESTNET_TRAINING_CONTRACT_ADDRESS,
            settlement_address: TESTNET_SETTLEMENT_CONTRACT_ADDRESS,
        }
    }
}
