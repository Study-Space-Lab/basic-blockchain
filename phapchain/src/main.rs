mod phapchain;

use phapchain::{Block, Blockchain, Transaction, TransactionData};
use std::borrow::BorrowMut;

fn main() {
    println!("---------Demo PhapChain Version 1.0---------\n");

    // Create a new Blockchain
    let mut bc = Blockchain::new();

    // Create an empty block (first block has no prev_block)
    let mut genesis = Block::new(None);

    // Người dùng đầu
    let initial_users = vec!["phap", "luong"];

    // Người dùng đầu sẽ tạo 2 transaction
    // 1. Tạo userAccount
    // 2. Tạo token
    for user in initial_users {
        let create_transaction = Transaction::new(
            user.into(),
            TransactionData::CreateUserAccount(user.into()),
            0,
        );

        let token_action = Transaction::new(
            user.into(),
            TransactionData::CreateTokens {
                receiver: user.into(),
                amount: 100_000_000,
            },
            0,
        );

        genesis.add_transaction(create_transaction);

        genesis.add_transaction(token_action);
    }

    let mut res = bc.append_block(genesis);
    println!("Genesis block successfully added: {:?}", res); // thoong bao Block genesis ok
    println!("Full blockchain printout");
    println!("{:#?}", bc);

    // Transfer 1 token from  "luong" to "phap"
    let mut block2 = Block::new(bc.get_last_block_hash());
    block2.add_transaction(Transaction::new(
        "luong".into(),
        TransactionData::TransferTokens {
            to: "phap".into(),
            amount: 1, // token chuyen di
        },
        0,
    ));

    res = bc.append_block(block2);
    println!("------------------\n Block 2 \n");
    println!("Block added: {:?}", res);
    println!("Full blockchain printout");
    println!("{:#?}", bc);
    println!("Blockchain valid: {:?}", bc.check_validity());

    // Attack I: Thay đôi giao dịch ( a transaction)
    // user "phap" sẽ thay đổi giao dịch vì pháp muốn lụm 100 token thay vì 1 token từ luong
    //
    // Đầu tiên là sao chép blockchain
    let mut bc_attack_1 = bc.clone();
    // get the transaction as mutable (second block, first transaction; the token transfer)
    // Lấy transaction dưới dạng có thể thay đổi
    // Lấy dữ liệu transaction - từ block lưu transaction chuyển tiền.
    let transaction_data = bc_attack_1.blocks[1].transactions[0].borrow_mut();

    // Xử lý phần attack: ở đây là acctack lại số tiền chuyển đi
    match transaction_data.record.borrow_mut() {
        &mut TransactionData::TransferTokens {
            to: _,
            ref mut amount,
        } => {
            *amount = 100; // Thay đổi token chuyển  thành 100 token ( như kiểu replace )
        }

        _ => {} // Thay đổi các transaction khác nếu muoón hack thêm. Ở đây thì chỉ thay đổi phần
                // TransferTokens
    }

    println!("\n------------------\n Attack \n------------------ ");
    println!("\nAttack I: Thay đổi số token \"luong\" chuyển qua \"phap\"  \n");
    println!("Changed transaction: {:?}", transaction_data.record);

    println!(" --> Check toàn bộ hash của chain ");
    // In ra lỗi , vì hàm băm block bị thay đổi
    println!(
        "Check Blockchain still valid? {:#?}",
        bc_attack_1.check_validity()
    );

    // Attack II: Thay đổi transaction + updating hash ( tăng token ban đầu khi tạo user)
    let mut bc_attack_2 = bc.clone();

    // Phap tokens
    // Lại lấy transaction trong block tạo user đầu tiên
    let transaction_data = bc_attack_2.blocks[0].transactions[1].borrow_mut();

    // change tokens
    match transaction_data.record.borrow_mut() {
        &mut TransactionData::CreateTokens {
            receiver: _,
            ref mut amount,
        } => {
            *amount = 100_000_000_000;
        }
        _ => {}
    }
    println!("\nAttack II: Thay đổi Transaction + updating hash\n");
    println!("Changed transaction: {:?}", transaction_data.record);

    // In ra lỗi, ở khối đầu tiên hàm băm ra khoog khớp
    println!(
        "Check Blockchain still valid? {:#?}",
        bc_attack_2.check_validity()
    );

    // Nếu cập nhật lại hàm băm đầu tiên
    bc_attack_2.blocks[0].update_hash();

    // Hăm băm đầu tiên đã đúng, nhưng lúc này Block thứ 2 không trỏ đến vì prev_hash mà block 2
    // lưu không = hash của block 1 ban đầu.
    // In ra lỗi
    println!(
        "Check Blockchain still valid? {:#?}",
        bc_attack_2.check_validity()
    );
}
