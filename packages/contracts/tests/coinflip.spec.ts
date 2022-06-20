import { utils } from "near-api-js";
import { Worker, NearAccount } from "near-workspaces";

interface UserStorage {
  total: string;
  available: string;
}

interface FeeConfig {
  partner_fee: string;
  bet_payment_adjustment: string;
  house_fee: string;
  max_bet: string;
  min_bet: string;
  max_odds: number;
  min_odds: number;
  nft_fee: string;
  owner_fee: string;
}

describe("Greeting Contract Integration Tests", () => {
  let worker: Worker;
  let root: NearAccount;

  let owner: NearAccount;
  let nft_account: NearAccount;
  let project_owner: NearAccount;
  let user: NearAccount;

  let coinflipContractAccount: NearAccount;
  let tokenContractAccount: NearAccount;

  let fee_config: FeeConfig;

  beforeAll(async () => {
    worker = await Worker.init();

    root = worker.rootAccount;

    owner = await root.createAccount("owner");
    nft_account = await root.createAccount("nft_account");
    project_owner = await root.createAccount("project_owner");
    user = await root.createAccount("user");

    coinflipContractAccount = await root.createAndDeploy(
      "coinflip",
      __dirname + "/../out/coin_flip.wasm"
    );
    fee_config = {
      partner_fee: "5000",
      bet_payment_adjustment: "100000",
      house_fee: "5000",
      max_bet: utils.format.parseNearAmount("3")!,
      min_bet: utils.format.parseNearAmount("0.1")!,
      max_odds: 200,
      min_odds: 50,
      nft_fee: "5000",
      owner_fee: "1500",
    };

    tokenContractAccount = await root.createAndDeploy(
      "token",
      __dirname + "/../out/test_token.wasm"
    );
    await root.call(tokenContractAccount, "new", {});
    await root.call(tokenContractAccount, "mint", {
      account_id: project_owner.accountId,
      amount: utils.format.parseNearAmount("10000"),
    });
    await root.call(tokenContractAccount, "mint", {
      account_id: coinflipContractAccount.accountId,
      amount: "0",
    });
    await root.call(tokenContractAccount, "mint", {
      account_id: user.accountId,
      amount: utils.format.parseNearAmount("100"),
    });

    expect(await coinflipContractAccount.exists()).toBe(true);
    expect(await tokenContractAccount.exists()).toBe(true);
  });

  afterAll(async () => {
    await worker.tearDown();
  });

  it("should initialize the contract", async () => {
    let contract_balance = await coinflipContractAccount.balance();

    await owner.call(coinflipContractAccount.accountId, "new", {
      owner_id: owner.accountId,
      nft_account: nft_account.accountId,
    });

    let internal_storage_deposit: UserStorage =
      await coinflipContractAccount.view("storage_balance_of", {
        account_id: coinflipContractAccount.accountId,
      });

    expect(internal_storage_deposit.total).toEqual(
      contract_balance.total.toString()
    );
  });

  it("should create game", async () => {
    let initial_storage_deposit: UserStorage =
      await coinflipContractAccount.view("storage_balance_of", {
        account_id: coinflipContractAccount.accountId,
      });

    await owner.call(
      coinflipContractAccount,
      "create_new_partner",
      {
        partner_owner: project_owner,
        nft_contract: nft_account.accountId,
        token_contract: tokenContractAccount.accountId,
        ...fee_config,
      },
      { attachedDeposit: "1" }
    );

    let final_storage_deposit: UserStorage = await coinflipContractAccount.view(
      "storage_balance_of",
      { account_id: coinflipContractAccount.accountId }
    );

    expect(initial_storage_deposit.total).toEqual(final_storage_deposit.total);
    expect(BigInt(final_storage_deposit.available)).toBeLessThan(
      BigInt(initial_storage_deposit.available)
    );
  });

  it("should fund game", async () => {
    let balance_to_fund = utils.format.parseNearAmount("10000");

    await project_owner.call(
      tokenContractAccount.accountId,
      "ft_transfer_call",
      {
        receiver_id: coinflipContractAccount.accountId,
        amount: balance_to_fund,
        memo: null,
        msg: JSON.stringify({
          type: "FundGame",
          game_id: nft_account.accountId,
        }),
      },
      {
        gas: "300 TGas",
        attachedDeposit: "1",
      }
    );

    let game_state: any = await coinflipContractAccount.view(
      "view_partner_data",
      { nft_contract: nft_account.accountId }
    );

    expect(game_state.house_funds).toEqual(balance_to_fund);
  });

  it("should register an user", async () => {
    // minimum storage is 0.25 NEAR
    let deposit_value: string = utils.format.parseNearAmount("1")!;

    const user_not_found = async () => {
      await coinflipContractAccount.view("get_credits", {
        token_type: tokenContractAccount.accountId,
        account_id: user.accountId,
      });
    };
    await expect(user_not_found()).rejects.toThrow(
      "ERR_001: Account is not registered"
    );

    let initial_user_balance = await user.balance();

    let user_deposit_balance: UserStorage = await user.call(
      coinflipContractAccount,
      "storage_deposit",
      {
        account_id: user.accountId,
        registration_only: true,
      },
      {
        attachedDeposit: deposit_value,
      }
    );

    let final_user_balance = await user.balance();
    expect(user_deposit_balance.total).toEqual(
      utils.format.parseNearAmount("0.25")
    );
    expect(BigInt(final_user_balance.total.toString())).toBeGreaterThanOrEqual(
      BigInt(initial_user_balance.total.toString()) - BigInt(deposit_value)
    );
  });

  it("should accept token deposits from players", async () => {
    let value_to_deposit: string = utils.format.parseNearAmount("10")!;

    let user_initial_balance: string = await tokenContractAccount.view(
      "ft_balance_of",
      { account_id: user.accountId }
    );
    let contract_finitial_balance: string = await tokenContractAccount.view(
      "ft_balance_of",
      { account_id: coinflipContractAccount.accountId }
    );
    let user_initial_contract_balance: string =
      await coinflipContractAccount.view("get_credits", {
        token_type: tokenContractAccount.accountId,
        account_id: user.accountId,
      });

    await user.call(
      tokenContractAccount,
      "ft_transfer_call",
      {
        receiver_id: coinflipContractAccount.accountId,
        amount: value_to_deposit,
        memo: null,
        msg: JSON.stringify({ type: "DepositBalance" }),
      },
      {
        gas: "300 TGas",
        attachedDeposit: "1",
      }
    );

    let user_final_balance: string = await tokenContractAccount.view(
      "ft_balance_of",
      { account_id: user.accountId }
    );
    let contract_final_balance: string = await tokenContractAccount.view(
      "ft_balance_of",
      { account_id: coinflipContractAccount.accountId }
    );
    let user_final_contract_balance: string =
      await coinflipContractAccount.view("get_credits", {
        token_type: tokenContractAccount.accountId,
        account_id: user.accountId,
      });

    expect(BigInt(user_final_balance)).toEqual(
      BigInt(user_initial_balance) - BigInt(value_to_deposit)
    );
    expect(BigInt(contract_final_balance)).toEqual(
      BigInt(contract_finitial_balance) + BigInt(value_to_deposit)
    );
    expect(user_initial_contract_balance).toEqual("0");
    expect(user_final_contract_balance).toEqual(value_to_deposit);
  });

  it("should be able to play the game, with different odds and bet sizes", async () => {
    let user_initial_token_balance: string = await coinflipContractAccount.view(
      "get_credits",
      { token_type: tokenContractAccount.accountId, account_id: user.accountId }
    );
    let user_current_balance: string = user_initial_token_balance;
    let new_user_balance: string;
    let bet_size: string = utils.format.parseNearAmount("3")!;
    while (
      BigInt(user_current_balance) > BigInt(utils.format.parseNearAmount("3")!)
    ) {
      let won_status: boolean = await user.call(
        coinflipContractAccount,
        "play",
        {
          game_code: nft_account.accountId,
          bet_size,
          odds: 128,
          _bet_type: "whatever",
        }
      );

      new_user_balance = await coinflipContractAccount.view("get_credits", {
        token_type: tokenContractAccount.accountId,
        account_id: user.accountId,
      });
      if (won_status) {
        expect(BigInt(new_user_balance)).toBeGreaterThan(
          BigInt(user_current_balance)
        );
      } else {
        expect(BigInt(new_user_balance)).toEqual(
          BigInt(user_current_balance) - BigInt(bet_size)
        );
      }
      user_current_balance = new_user_balance;
    }
  });

  it("should enable user to withdraw credits", async () => {
    let user_credits_initial_balance: BigInt = BigInt(
      await coinflipContractAccount.view("get_credits", {
        token_type: tokenContractAccount.accountId,
        account_id: user.accountId,
      })
    );
    let user_wallet_initial_balance: BigInt = BigInt(
      await tokenContractAccount.view("ft_balance_of", {
        account_id: user.accountId,
      })
    );

    await user.call(
      coinflipContractAccount,
      "retrieve_credits",
      {
        token_contract: tokenContractAccount.accountId,
        amount: user_credits_initial_balance.toString(),
      },
      {
        gas: "300 TGas",
      }
    );

    let user_credits_final_balance: BigInt = BigInt(
      await coinflipContractAccount.view("get_credits", {
        token_type: tokenContractAccount.accountId,
        account_id: user.accountId,
      })
    );
    let user_wallet_final_balance: BigInt = BigInt(
      await tokenContractAccount.view("ft_balance_of", {
        account_id: user.accountId,
      })
    );

    expect(user_credits_final_balance).toEqual(BigInt("0"));
    expect(user_wallet_final_balance).toEqual(
      BigInt(user_wallet_initial_balance.toString()) +
        BigInt(user_credits_initial_balance.toString())
    );
  });

  it("should enable project_owner to withdraw house_funds and project_funds", async () => {
    let partner_data: any = await coinflipContractAccount.view(
      "view_partner_data",
      { nft_contract: nft_account.accountId }
    );
    let partner_funds: string = partner_data.partner_balance;
    let house_funds: string = partner_data.house_funds;
    let initial_partner_wallet: string = await tokenContractAccount.view(
      "ft_balance_of",
      { account_id: project_owner.accountId }
    );

    await project_owner.call(
      coinflipContractAccount,
      "retrieve_partner_balance",
      {
        game_code: nft_account.accountId,
      },
      {
        gas: "300 TGas",
      }
    );
    await project_owner.call(
      coinflipContractAccount,
      "retrieve_house_funds",
      {
        game_code: nft_account.accountId,
        quantity: house_funds.toString(),
      },
      {
        gas: "300 TGas",
      }
    );

    let final_partner_data: any = await coinflipContractAccount.view(
      "view_partner_data",
      { nft_contract: nft_account.accountId }
    );
    let final_partner_funds: string = final_partner_data.partner_balance;
    let final_house_funds: string = final_partner_data.house_funds;
    let final_partner_wallet: string = await tokenContractAccount.view(
      "ft_balance_of",
      { account_id: project_owner.accountId }
    );

    expect(BigInt(final_partner_wallet)).toEqual(
      BigInt(initial_partner_wallet) +
        BigInt(partner_funds) +
        BigInt(house_funds)
    );
    expect(final_partner_funds).toEqual("0");
    expect(final_house_funds).toEqual("0");
  });

  it(`shouldn't enable user to play if there aren't enough house funds`, async () => {
    await user.call(
      tokenContractAccount,
      "ft_transfer_call",
      {
        receiver_id: coinflipContractAccount.accountId,
        amount: utils.format.parseNearAmount("1"),
        memo: null,
        msg: JSON.stringify({ type: "DepositBalance" }),
      },
      {
        gas: "300 TGas",
        attachedDeposit: "1",
      }
    );

    const should_fail = async () => {
      await user.call(coinflipContractAccount, "play", {
        game_code: nft_account.accountId,
        bet_size: utils.format.parseNearAmount("1"),
        odds: 128,
        _bet_type: "whatever",
      });
    };

    await expect(should_fail()).rejects.toThrow(
      "ERR_407: Bet denied, house_funds are not enough to cover your possible win value"
    );
  });
});
