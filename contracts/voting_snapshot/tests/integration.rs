use near_sdk::NearToken;
use near_workspaces::{network::Sandbox, Account, AccountId, Contract, DevNetwork, Worker};
use serde_json::json;
use voting_snapshot::types::{SnapshotConfig, Status, UserData, VoteWeightConfig};

pub fn default_vote_config() -> VoteWeightConfig {
    VoteWeightConfig {
        threshold_in_nears: 40,
        activity_reward_in_votes: 10,
    }
}

pub fn default_snapshot_config() -> SnapshotConfig {
    SnapshotConfig {
        challenge_threshold_in_nears: 30,

        // 5 seconds
        challenge_timeout_in_millis: 5 * 1000,
        registration_timeout_in_millis: 10 * 1000,
    }
}

pub struct Ctx {
    pub contract: Contract,
    pub admin: Account,
    pub account: Account,
}

impl Ctx {
    async fn new(worker: &Worker<impl DevNetwork>) -> anyhow::Result<Self> {
        let admin_acc = worker.dev_create_account().await?;
        let account_acc = worker.dev_create_account().await?;
        let admin = admin_acc
            .create_subaccount("admin")
            .initial_balance(NearToken::from_near(90))
            .transact()
            .await?
            .into_result()?;
        let account = account_acc
            .create_subaccount("account")
            .initial_balance(NearToken::from_near(90))
            .transact()
            .await?
            .into_result()?;

        let contract = worker
            .dev_deploy(include_bytes!(
                "../../../target/near/voting_snapshot/voting_snapshot.wasm"
            ))
            .await?;

        worker
            .transfer_near(admin.signer(), contract.id(), NearToken::from_near(20))
            .await?
            .into_result()?;

        let res = contract
            .call("new")
            .args_json(json!(
                {
                    "admin": admin.id().clone(),
                    "vote_config": default_vote_config(),
                    "process_config": default_snapshot_config()
                }
            ))
            .max_gas()
            .transact()
            .await?;

        assert!(res.is_success(), "Failed to deploy contract: {:?}", res);
        Ok(Ctx {
            contract,
            admin,
            account,
        })
    }

    pub async fn add_snapshot_data(
        &self,
        account_id: AccountId,
        user_data: UserData,
    ) -> anyhow::Result<()> {
        let input = vec![(account_id, user_data)];
        let res = self
            .admin
            .call(self.contract.id(), "bulk_load_voters")
            .args_json(json!({ "voters": input }))
            .max_gas()
            .transact()
            .await?;
        assert!(res.is_success(), "Failed to add snapshot data: {:?}", res);

        Ok(())
    }

    pub async fn move_to_challenge(&self) -> anyhow::Result<()> {
        let res = self
            .admin
            .call(self.contract.id(), "start_challenge")
            .max_gas()
            .transact()
            .await?;
        assert!(res.is_success(), "Failed to move to challenge: {:?}", res);
        assert_eq!(self.get_status().await?, Status::SnapshotChallenge(0));

        Ok(())
    }

    pub async fn challenge(&self, amount: NearToken, expectation: bool) -> anyhow::Result<()> {
        let res = self
            .account
            .call(self.contract.id(), "challenge_snapshot")
            .deposit(amount)
            .transact()
            .await?;

        assert_eq!(
            res.is_success(),
            expectation,
            "Failed to challenge: {:?}",
            res
        );

        Ok(())
    }

    pub async fn get_status(&self) -> anyhow::Result<Status> {
        let res = self
            .account
            .view(self.contract.id(), "get_status")
            .await?
            .json()?;
        Ok(serde_json::from_value(res)?)
    }

    pub async fn refund_bond(&self) -> anyhow::Result<()> {
        let res = self
            .account
            .call(self.contract.id(), "refund_bond")
            .transact()
            .await?;
        assert!(res.is_success(), "Failed to refund bond: {:?}", res);
        Ok(())
    }

    pub async fn get_challenge_deposit(
        &self,
        account_id: AccountId,
    ) -> anyhow::Result<Option<NearToken>> {
        let res = self
            .account
            .view(self.contract.id(), "get_individual_challenge")
            .args_json(json!({ "challenger": account_id}))
            .await?
            .json()?;
        Ok(serde_json::from_value(res)?)
    }

    pub async fn get_ending_time_of_phase(&self) -> anyhow::Result<(u64, u64)> {
        let res = self
            .account
            .view(self.contract.id(), "get_end_time")
            .await?
            .json()?;
        Ok(serde_json::from_value(res)?)
    }
}

#[tokio::test]
async fn halt_and_return_flow() {
    let worker = near_workspaces::sandbox().await.unwrap();
    let ctx = Ctx::new(&worker).await.unwrap();

    ctx.add_snapshot_data(
        ctx.account.id().clone(),
        UserData {
            active_months: 1,
            stake: NearToken::from_near(2),
        },
    )
    .await
    .unwrap();

    ctx.move_to_challenge().await.unwrap();

    let balance_before = ctx.account.view_account().await.unwrap().balance;
    ctx.challenge(NearToken::from_near(40), true).await.unwrap();
    let balance_after = ctx.account.view_account().await.unwrap().balance;

    assert_eq!(
        balance_before
            .checked_sub(NearToken::from_near(40))
            .unwrap()
            .as_near()
            - 1,
        balance_after.as_near()
    );
    assert_eq!(ctx.get_status().await.unwrap(), Status::SnapshotHalted(0));
    assert_eq!(
        ctx.get_challenge_deposit(ctx.account.id().clone())
            .await
            .unwrap(),
        Some(NearToken::from_near(40))
    );

    ctx.refund_bond().await.unwrap();

    let balance_after_refund = ctx.account.view_account().await.unwrap().balance;
    assert_eq!(balance_before.as_near() - 1, balance_after_refund.as_near());

    assert_eq!(
        ctx.get_challenge_deposit(ctx.account.id().clone())
            .await
            .unwrap(),
        None,
    );
}

async fn time_travel(worker: &Worker<Sandbox>, seconds_to_advance: u64) -> anyhow::Result<()> {
    let blocks_to_advance = (seconds_to_advance * 1_000_000_000) / 1_200_000_000;
    worker.fast_forward(blocks_to_advance).await?;
    anyhow::Ok(())
}

#[tokio::test]
async fn challenged_but_not_halted_still_can_retrieve_deposit_later() {
    let worker = near_workspaces::sandbox().await.unwrap();
    let ctx = Ctx::new(&worker).await.unwrap();

    ctx.add_snapshot_data(
        ctx.account.id().clone(),
        UserData {
            active_months: 1,
            stake: NearToken::from_near(2),
        },
    )
    .await
    .unwrap();

    ctx.move_to_challenge().await.unwrap();

    let balance_before = ctx.account.view_account().await.unwrap().balance;
    ctx.challenge(NearToken::from_near(15), true).await.unwrap();
    let balance_after = ctx.account.view_account().await.unwrap().balance;

    assert_eq!(
        balance_before
            .checked_sub(NearToken::from_near(15))
            .unwrap()
            .as_near()
            - 1,
        balance_after.as_near()
    );
    assert_eq!(
        ctx.get_status().await.unwrap(),
        Status::SnapshotChallenge(0)
    );
    assert_eq!(
        ctx.get_challenge_deposit(ctx.account.id().clone())
            .await
            .unwrap(),
        Some(NearToken::from_near(15))
    );

    time_travel(&worker, 15).await.unwrap();

    // Test that we get back deposit after challenge state is over
    ctx.challenge(NearToken::from_near(25), false)
        .await
        .unwrap();

    let balance_after2 = ctx.account.view_account().await.unwrap().balance;
    assert_eq!(balance_after.as_near(), balance_after2.as_near());

    ctx.refund_bond().await.unwrap();
    let balance_after_refund = ctx.account.view_account().await.unwrap().balance;
    assert_eq!(balance_before.as_near() - 1, balance_after_refund.as_near());

    assert_eq!(ctx.get_status().await.unwrap(), Status::Registration(0));

    assert_eq!(
        ctx.get_challenge_deposit(ctx.account.id().clone())
            .await
            .unwrap(),
        None,
    );
}
