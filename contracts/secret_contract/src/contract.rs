use secp256k1::{PublicKey, Secp256k1, SecretKey};

use crate::error::ContractError;
use crate::msg::{InstantiateMsg, KeysResponse, QueryMsg};
use crate::state::{MyKeys, MY_KEYS};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, Timestamp,
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    create_keys(deps, env, msg.end_time)?;

    Ok(Response::default())
}

pub fn create_keys(
    deps: DepsMut,
    env: Env,
    end_time: Timestamp,
) -> Result<Response, ContractError> {
    if end_time < env.block.time {
        return Err(ContractError::InvalidEndTime);
    }

    let rng = env.block.random.unwrap().0;
    let secp = Secp256k1::new();

    let private_key = SecretKey::from_slice(&rng).unwrap();
    let private_key_bytes = private_key.secret_bytes().to_vec();

    let public_key = PublicKey::from_secret_key(&secp, &private_key);
    let public_key_bytes = public_key.serialize().to_vec();

    let my_keys = MyKeys {
        private_key: private_key_bytes,
        public_key: public_key_bytes,
        end_time,
    };

    MY_KEYS.save(deps.storage, &my_keys)?;

    Ok(Response::default())
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    let result = match msg {
        QueryMsg::GetKeys {} => to_binary(&query_keys(deps, env)?),
    };
    result.map_err(Into::into)
}

fn query_keys(deps: Deps, env: Env) -> Result<KeysResponse, ContractError> {
    let my_keys = MY_KEYS.load(deps.storage)?;
    let private = (env.block.time > my_keys.end_time).then(|| my_keys.private_key);

    Ok(KeysResponse {
        public: my_keys.public_key,
        private,
        end_time: my_keys.end_time,
    })
}
