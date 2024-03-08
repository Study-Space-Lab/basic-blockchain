extern crate blake2;

use sha2::{Digest, Sha256};
// use blake2::{Blake2b, Digest};
// use blake2::{Blake2b512, Blake2s256};
use std::collections::HashMap;
use std::convert::From;
use std::convert::Into;
use std::string::String;
use std::time::SystemTime;
use std::vec::Vec;

/// Blockchain contrainer
#[derive(Clone, Debug)]
pub struct Blockchain {
    // Lưu trữ tất cả các khối đã được accepted trong blockchain
    pub blocks: Vec<Block>,

    /// Tra cứu từ AccountID (sau này sẽ là public key) tới Account.
    pub accounts: HashMap<String, Account>,

    // Lưu trữ các giao dịch cần được thêm vào chuỗi ( Cac transaction chưa được thêm vào block)
    pending_transaction: Vec<Transaction>,
}

/// Thể hiện trạng thái hiện tại của blockchain sau khi tất cả các Khối được thực thi
pub trait WorldState {
    // lấy tất cả các id người dùng đã đăng ký.
    fn get_user_ids(&self) -> Vec<String>;

    // Trả về tài khoản có id nếu có. (mutable)
    fn get_account_by_id_mut(&mut self, id: &String) -> Option<&mut Account>;

    // Sẽ trả lại tài khoản đã cung cấp id nếu có
    fn get_account_by_id(&self, id: &String) -> Option<&Account>;

    // Thêm account mới
    fn create_account(&mut self, id: String, account_type: AccountType)
        -> Result<(), &'static str>;
}
/// Cac block ( về cơ bản sẽ chứa dữ liệu, ds các giao dịch)
#[derive(Clone, Debug)]
pub struct Block {
    // Action cua block
    pub(crate) transactions: Vec<Transaction>,

    // ket noi block
    prev_hash: Option<String>,

    // luu hàm băm của khối
    hash: Option<String>,

    // Một số số tùy ý mà sau này sẽ được sử dụng cho Proof of Work
    // concept: Là những chuỗi số 32 bit ngẫu nhiên được sử dụng trong quá trình khai thác khối (block) mới trong blockchain Proof-of-Work (PoW).
    nonce: u128,
}

/// lưu trữ request vào blockchain
#[derive(Clone, Debug)]
pub struct Transaction {
    /// Số duy nhất (sẽ được sử dụng để ngẫu nhiên hóa sau này; ngăn chặn các cuộc tấn công lặp lại)
    nonce: u128,

    /// Account ID
    from: String,

    /// Lưu trữ thời gian giao dịch được tạo
    created_at: SystemTime,

    /// loại giao dịch và thông tin bổ sung của nó
    pub(super) record: TransactionData,

    // Chữ ký băm của toàn bộ tin nhắn
    signature: Option<String>, // cai nay can tim hieu them
}

#[derive(Clone, Debug, PartialEq)]
pub enum TransactionData {
    // Sẽ được sử dụng để lưu trữ tài khoản người dùng mới
    CreateUserAccount(String),

    // Có thể sử dụng để thay đổi hoặc tạo một giá trị tùy ý vào tài khoản
    ChangeStoreValue { key: String, value: String },

    // Sẽ được sử dụng để di chuyển `token` từ chủ sở hữu này sang chủ sở hữu khác
    TransferTokens { to: String, amount: u128 },

    // Tao Token
    CreateTokens { receiver: String, amount: u128 },
    // ... Extend
}
/// Đại diện cho một tài khoản trên blockchain
#[derive(Clone, Debug)]
pub struct Account {
    // thoong tin duoc luu
    store: HashMap<String, String>,
    // lưu trữ nếu đây là tài khoản người dùng or Any
    acc_type: AccountType,
    //Số lượng token mà tài khoản sở hữu (như BTC hoặc ETH)
    tokens: u128,
}

/// AccountType: sau này có thể hỗ trợ nhiều loại tài khoản khác nhau
/// Ở đây tui dùng tài khoản User thui
#[derive(Clone, Debug)]
pub enum AccountType {
    // Tai khoan nguoi dung
    User,

    // Contract này thì thêm dô cho đủ chứ chưa đủ trình làm :))
    _Contract,

    // Validator
    _Validator {
        correctly_validated_blocks: u128,
        incorrectly_validated_blocks: u128,
        // Thêm các chức năng khác ...
    },
    // Thêm vài trò khác ...
}

/// ** Implemention ** ///

impl Blockchain {
    /// Contructor
    pub fn new() -> Self {
        Blockchain {
            blocks: Vec::new(),
            accounts: HashMap::new(),
            pending_transaction: Vec::new(), // transaction dang chờ xử lý
        }
    }

    /// thêm một khối vào Blockchain
    pub fn append_block(&mut self, block: Block) -> Result<(), String> {
        // check blockchain genesis
        let is_genesis = self.len() == 0;

        if !block.verify_own_hash() {
            return Err("The block hash in mismatching! (code: mod.rs: 140)".into());
        }

        // Kiểm tra xem khối mới được thêm vào có được thêm vào khối cuối cùng không
        if !(block.prev_hash == self.get_last_block_hash()) {
            return Err(
                "The new block has to point to the previous block (Code: mod.rs: 147)".into(),
            );
        }

        // Phải có ít nhất một giao dịch(transaction) trong hàng đợi
        if block.get_transaction_count() == 0 {
            return Err("There has to be at least one transaction \
            inside the block! (Code: mod.rs: 153)"
                .into());
        }

        let old_state = self.accounts.clone();

        // thực hiện từng transaction
        for (i, transaction) in block.transactions.iter().enumerate() {
            // Execute the transaction
            if let Err(err) = transaction.execute(self, &is_genesis) {
                // Recover state on failure
                self.accounts = old_state;

                // ... and reject the block
                return Err(format!(
                    "Could not execute transaction {} due to `{}`. Rolling back \
                (Code: mod.rs: 171)",
                    i + 1,
                    err
                ));
            }
        }

        // nối thêm khối
        self.blocks.push(block);

        Ok(())
    }
    /// hàm sẽ  trả về khối cuối cùng  
    pub fn get_last_block_hash(&self) -> Option<String> {
        if self.len() == 0 {
            return None;
        }
        // Trả về clone `hash` của khối cuối
        self.blocks[self.len() - 1].hash.clone()
    }

    /// Sẽ trả về số lượng khối hiện được lưu trữ
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Check toan bo blockchain
    pub fn check_validity(&self) -> Result<(), String> {
        // check Block
        for (block_num, block) in self.blocks.iter().enumerate() {
            // Kiểm tra xem khối băm đã lưu có khớp với hàm băm được tính toán hay không
            if !block.verify_own_hash() {
                return Err(format!(
                    "Stored hash for Block #{} \
                    does not match calculated hash (Code: mod.rs: 204)",
                    block_num + 1
                )
                .into());
            }

            // Check previous
            if block_num == 0 {
                // Genesis block - Block genesis phải trỏ vào None
                if block.prev_hash.is_some() {
                    // Trả về lỗi khi genesis block có prev_háh
                    return Err("The genesis block has a previous hash set which \
                     it shouldn't Code :mod.rs: 215"
                        .into());
                }
            } else {
                // Các block sau có prev_hash
                if block.prev_hash.is_none() {
                    // trả về chỉ số block không có prev_hash
                    return Err(format!("Block #{} has no previous hash set", block_num + 1).into());
                }

                // Lưu trữ các giá trị cục bộ để sử dụng chúng trong thông báo lỗi khi xảy ra lỗi
                let prev_hash_proposed = block.prev_hash.as_ref().unwrap();
                println!("prev_hash_proposed: {}", prev_hash_proposed);
                let prev_hash_actual = self.blocks[block_num - 1].hash.as_ref().unwrap();
                println!("prev_hash_actual: {}", prev_hash_actual);

                if !(&block.prev_hash == &self.blocks[block_num - 1].hash) {
                    return Err(format!(
                        //     "Block #{} is not connected to previous block (Hashes do \
                        // not match. Should be `{}`  but is `{}`)",
                        "Block #{} is not connected to previous block",
                        block_num // block_num, prev_hash_proposed, prev_hash_actual
                    )
                    .into());
                }
            }

            // Check Transaction
            for (transaction_num, transaction) in block.transactions.iter().enumerate() {
                // Hiện tại sẽ chỉ check signature chưa được ký ( None) thì có giá trị.
                if transaction.is_signed() && !transaction.check_signature() {
                    return Err(format!(
                        "Transaction #{} for Block #{} has an invalid signature \
                    (Code: mod.rs: 245)",
                        transaction_num + 1,
                        block_num + 1
                    ));
                }
            }
        }
        Ok(())
    }
}

impl WorldState for Blockchain {
    /// Trả về danh sách tên user
    fn get_user_ids(&self) -> Vec<String> {
        self.accounts.keys().map(|s| s.clone()).collect()
    }

    /// Trả về `Option<mut Account>`   
    fn get_account_by_id_mut(&mut self, id: &String) -> Option<&mut Account> {
        self.accounts.get_mut(id)
    }

    /// Trả về `Option<&Account>`
    /// Accout Có hoặc không
    fn get_account_by_id(&self, id: &String) -> Option<&Account> {
        self.accounts.get(id)
    }

    /// Hàm tạo account
    /// `return Ok(()) || Err`
    fn create_account(
        &mut self,
        id: String,
        account_type: AccountType,
    ) -> Result<(), &'static str> {
        return if !self.get_user_ids().contains(&id) {
            let acc = Account::new(account_type);
            self.accounts.insert(id, acc);
            Ok(())
        } else {
            Err("User already exists! (Code: mod.rs: 279)")
        };
    }
}

impl Block {
    pub fn new(prev_hash: Option<String>) -> Self {
        Block {
            nonce: 0,
            hash: None,
            prev_hash,
            transactions: Vec::new(),
        }
    }
    /// Sẽ tính toán hàm băm của toàn bộ khối bao gồm các giao dịch(transaction) Máy băm Blake2
    pub fn calculate_hash(&self) -> Vec<u8> {
        // Vì băm trả về kiểu dộng dynamic
        // hasher ko thể tạo do compiler không biết size cần là bao nhiêu
        let mut hasher = Sha256::new();

        for transaction in self.transactions.iter() {
            hasher.update(transaction.calculate_hash())
        }

        let block_as_string = format!("{:?}", (&self.prev_hash, &self.nonce));
        hasher.update(&block_as_string);
        let hash = hasher.finalize();
        let mut ret: [u8; 32] = <[u8; 32]>::default();
        // let mut ret: Vec<u8> = <Vec<u8>>::default();
        ret.copy_from_slice(&hash);
        Vec::from(ret)

        // return Vec::from(hasher.finalize().as_ref());
    }
    /// Nối thêm transaction vào queue
    /// Cập nhật filed `transactions` cho Block
    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
        self.update_hash();
    }
    /// Trả về số lượng transaction
    pub fn get_transaction_count(&self) -> usize {
        self.transactions.len()
    }
    /// Cập nhật filed `hash` cho Block
    pub(crate) fn update_hash(&mut self) {
        self.hash = Some(byte_vector_to_string(&self.calculate_hash()));
    }
    /// Check xem hàm băm đã được đặt chưa và có khớp với các khối bên trong không
    /// ```
    /// blocks.hash.eq(byte_vector_to_string(&self.calculate_hash()))
    ///
    /// ```
    pub fn verify_own_hash(&self) -> bool {
        if self.hash.is_some()
            && self
                .hash
                .as_ref()
                .unwrap()
                .eq(&byte_vector_to_string(&self.calculate_hash()))
        {
            return true;
        }
        false
    }
}

impl Transaction {
    /// Contructor
    pub fn new(from: String, transaction_data: TransactionData, nonce: u128) -> Self {
        Transaction {
            nonce,
            from,
            created_at: SystemTime::now(),
            record: transaction_data,
            signature: None,
        }
    }

    /// Sẽ thay đổi `world state` theo lệnh giao dich
    pub fn execute<T: WorldState>(
        &self,
        world_state: &mut T,
        is_initial: &bool,
    ) -> Result<(), &'static str> {
        // Kiểm tra xem người dùng gửi có tồn tại không (không ai ngoài chuỗi có thể thực hiện giao dịch)
        if let Some(_account) = world_state.get_account_by_id(&self.from) {
            // Do some more checkups later on...
        } else {
            if !is_initial {
                return Err("Account does not exist (Code: mod.rs: 362)");
            }
        }
        // Chúng ta sẽ kiểm tra loại transaction ở đây và thực hiện logic của nó
        return match &self.record {
            TransactionData::CreateUserAccount(account) => {
                world_state.create_account(account.into(), AccountType::User)
            }
            TransactionData::CreateTokens { receiver, amount } => {
                if !is_initial {
                    return Err(
                        "Token creation is only available on initial creation (Code: mod.rs: 373)",
                    );
                }

                // Người dùng nhận: `receiving use`(phải tồn tại)
                return if let Some(account) = world_state.get_account_by_id_mut(receiver) {
                    account.tokens += *amount;
                    Ok(())
                } else {
                    Err("Receiver Account does not exist (Code: mod.rs: 382)")
                };
            }
            TransactionData::TransferTokens { to, amount } => {
                let recv_tokens: u128;
                let sender_tokens: u128;

                // Check lại người nhận
                if let Some(recv) = world_state.get_account_by_id_mut(to) {
                    recv_tokens = recv.tokens;
                } else {
                    return Err("Receiver Account does not exist! (Code: mod.rs: 393)");
                }
                // Check người gửi
                if let Some(sender) = world_state.get_account_by_id_mut(&self.from) {
                    sender_tokens = sender.tokens;
                } else {
                    return Err("That account does not exist! (Code: mod.rs: 399)");
                }

                let balance_recv_new = recv_tokens.checked_add(*amount);
                let balance_sender_new = sender_tokens.checked_sub(*amount);

                if balance_recv_new.is_some() && balance_sender_new.is_some() {
                    world_state
                        .get_account_by_id_mut(&self.from)
                        .unwrap()
                        .tokens = balance_sender_new.unwrap(); // cap nhat tokens
                    world_state.get_account_by_id_mut(to).unwrap().tokens =
                        balance_recv_new.unwrap();
                    return Ok(());
                } else {
                    return Err("Overspent or Arithmetic error (Code: mod.rs: 414)");
                }
            }
            _ => {
                // Not implemented transaction type
                Err("Unknown Transaction type (not implemented) (Code: mod.rs: 419)")
            }
        };
    }
    /// Will calculate the hash using Blake2 hasher
    pub fn calculate_hash(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        let transaction_as_string = format!(
            "{:?}",
            (&self.created_at, &self.record, &self.from, &self.nonce)
        );

        hasher.update(&transaction_as_string);
        let hash = hasher.finalize();
        // let mut ret: [u8; 16] = <[u8; 16]>::default();
        let mut ret: [u8; 32] = <[u8; 32]>::default();
        ret.copy_from_slice(&hash);
        // return Vec::from(hasher.finalize().as_ref());
        Vec::from(ret.as_ref())
    }

    /// Will hash the transaction and check if the signature is valid
    /// (i.e., it is created by the owners private key)
    /// if the message is not signed it will always return false
    pub fn check_signature(&self) -> bool {
        if !(self.is_signed()) {
            return false;
        }

        //@TODO check signature
        false
    }

    pub fn is_signed(&self) -> bool {
        self.signature.is_some()
    }
}

impl Account {
    // Contructor
    pub fn new(account_type: AccountType) -> Self {
        Account {
            store: HashMap::new(),
            acc_type: account_type,
            tokens: 0,
        }
    }
}
// Sẽ lấy một mảng byte và chuyển đổi nó thành một chuỗi bằng cách diễn giải từng byte
//  dưới dạng một ký tự
fn byte_vector_to_string(arr: &Vec<u8>) -> String {
    arr.iter().map(|&c| c as char).collect()
}

// hex function - muc dich la muon dich ma hash theo format
// fn hex_to_string(data: &[u8]) -> String {
//     let mut ret = String::new();
//
//     for d in data {
//         let x = format!("{:02x}", d);
//         ret.push(&x);
//     }
//     ret
// }
