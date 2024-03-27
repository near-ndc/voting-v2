use cosmwasm_std::{from_binary, Coin, Empty, Env, OwnedDeps, Uint128};
use cosmwasm_std::{testing::*, Timestamp};
use secp256k1::{PublicKey, Secp256k1};

use crate::contract::{instantiate, query};
use crate::msg::{InstantiateMsg, KeysResponse, QueryMsg};

fn setup_contract(
    end_time: Timestamp,
    env: Env,
) -> OwnedDeps<MockStorage, MockApi, MockQuerier, Empty> {
    let mut deps = mock_dependencies();
    let info = mock_info(
        "creator",
        &[Coin {
            denom: "earth".to_string(),
            amount: Uint128::new(1000),
        }],
    );

    let init_msg = InstantiateMsg { end_time };

    let res = instantiate(deps.as_mut(), env, info, init_msg).unwrap();

    assert_eq!(0, res.messages.len());

    deps
}

#[test]
fn proper_initialization() {
    let time = Timestamp::from_seconds(500);
    let mut mock_env = mock_env();

    mock_env.block.time = time.minus_seconds(5);

    let deps = setup_contract(time, mock_env.clone());
    // it worked, let's query the state
    let res = query(deps.as_ref(), mock_env, QueryMsg::GetKeys {}).unwrap();
    let value: KeysResponse = from_binary(&res).unwrap();
    assert_eq!(value.private, None);
    assert_eq!(value.end_time, time);
    assert!(PublicKey::from_slice(&value.public).is_ok());
}

#[test]
fn invalid_time_is_failure() {
    let time = Timestamp::from_seconds(500);
    let mut deps = mock_dependencies();

    let mock_env = mock_env();

    let info = mock_info(
        "creator",
        &[Coin {
            denom: "earth".to_string(),
            amount: Uint128::new(1000),
        }],
    );

    let init_msg = InstantiateMsg { end_time: time };

    instantiate(deps.as_mut(), mock_env, info, init_msg).expect_err("Should fail");
}

#[test]
fn secret_revealed_after_time_pass() {
    let time = Timestamp::from_seconds(500);
    let mut mock_env = mock_env();

    mock_env.block.time = time.clone().minus_seconds(5);

    let deps = setup_contract(time, mock_env.clone());

    let res = query(deps.as_ref(), mock_env.clone(), QueryMsg::GetKeys {}).unwrap();
    let value: KeysResponse = from_binary(&res).unwrap();

    assert!(value.private.is_none());

    mock_env.block.time = time.plus_seconds(1);

    let res = query(deps.as_ref(), mock_env.clone(), QueryMsg::GetKeys {}).unwrap();
    let value: KeysResponse = from_binary(&res).unwrap();

    assert!(value.private.is_some());

    let private = secp256k1::SecretKey::from_slice(&value.private.unwrap()).unwrap();
    let inherited_public = private.public_key(&Secp256k1::new());

    assert_eq!(
        inherited_public,
        PublicKey::from_slice(&value.public).unwrap()
    );
}
