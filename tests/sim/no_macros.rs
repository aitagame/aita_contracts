use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_sdk_sim::{to_yocto, DEFAULT_GAS};

use crate::utils::init_no_macros as init;
use crate::utils::register_user;

#[test]
fn simulate_contract_functionable() {
    let initial_balance = to_yocto("100");
    let (root, ft, _) = init(initial_balance);

    let resp: String = root
        .view(
            ft.account_id(),
            "hello",
            &json!({
                "name": "test"
            })
            .to_string()
            .into_bytes(),
        )
        .unwrap_json();

    assert_eq!(resp, format!("Hello test"));
}

#[test]
fn simulate_total_supply() {
    let initial_balance = to_yocto("100");
    let (_, ft, _) = init(initial_balance);

    let total_supply: U128 = ft.view(ft.account_id(), "ft_total_supply", b"").unwrap_json();

    assert_eq!(initial_balance, total_supply.0);
}

#[test]
fn simulate_simple_transfer() {
    let transfer_amount = to_yocto("100");
    let initial_balance = to_yocto("100000");
    let (root, ft, alice) = init(initial_balance);

    // Transfer from root to alice.
    root.call(
        ft.account_id(),
        "ft_transfer",
        &json!({
            "receiver_id": alice.valid_account_id(),
            "amount": U128::from(transfer_amount)
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS,
        1, // deposit
    )
    .assert_success();

    let root_balance: U128 = root
        .view(
            ft.account_id(),
            "ft_balance_of",
            &json!({
                "account_id": root.valid_account_id()
            })
            .to_string()
            .into_bytes(),
        )
        .unwrap_json();
    let alice_balance: U128 = alice
        .view(
            ft.account_id(),
            "ft_balance_of",
            &json!({
                "account_id": alice.valid_account_id()
            })
            .to_string()
            .into_bytes(),
        )
        .unwrap_json();
    assert_eq!(initial_balance - transfer_amount, root_balance.0);
    assert_eq!(transfer_amount, alice_balance.0);
}

#[test]
fn simulate_tokens_purchase() {
    let deposit_amount = to_yocto("100");
    let purchase_amount = to_yocto("100");
    let initial_balance = to_yocto("100000");
    let insufficient_amount = to_yocto("0.01");
    let (root, ft, alice) = init(initial_balance);

    register_user(&ft);
    // Transfer from root to ft (contract balance).
    root.call(
        ft.account_id(),
        "ft_transfer",
        &json!({
            "receiver_id": ft.valid_account_id(),
            "amount": U128::from(deposit_amount * 2)
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS,
        1, // deposit
    )
    .assert_success();

    //Buy for different account
    root.call(
        ft.account_id(),
        "ft_purchase",
        &json!({
            "buyer_account_id": alice.account_id()
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS,
        deposit_amount,
    )
    .assert_success();

    //This try should fail because of insufficient amount
    assert!(
        !root
            .call(
                ft.account_id(),
                "ft_purchase",
                &json!({
                    "buyer_account_id": alice.account_id()
                })
                .to_string()
                .into_bytes(),
                DEFAULT_GAS,
                insufficient_amount,
            )
            .is_ok(),
        "Should not allow amounts < 1 NEAR"
    );

    let root_balance_before: U128 = root
        .view(
            ft.account_id(),
            "ft_balance_of",
            &json!({
                "account_id": root.valid_account_id()
            })
            .to_string()
            .into_bytes(),
        )
        .unwrap_json();

    //Buy for self
    root.call(
        ft.account_id(),
        "ft_purchase",
        &json!({
            "buyer_account_id": root.account_id()
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS,
        deposit_amount,
    )
    .assert_success();

    let root_balance_after: U128 = root
        .view(
            ft.account_id(),
            "ft_balance_of",
            &json!({
                "account_id": root.valid_account_id()
            })
            .to_string()
            .into_bytes(),
        )
        .unwrap_json();
    let alice_balance: U128 = alice
        .view(
            ft.account_id(),
            "ft_balance_of",
            &json!({
                "account_id": alice.valid_account_id()
            })
            .to_string()
            .into_bytes(),
        )
        .unwrap_json();
    assert_eq!(initial_balance - purchase_amount * 2, root_balance_before.0);
    assert_eq!(initial_balance - purchase_amount, root_balance_after.0);
    assert_eq!(purchase_amount, alice_balance.0);
}
