use alloy::node_bindings::Anvil;
use alloy::primitives::U256;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::sol;
use eyre::Result;
use alloy::network::EthereumWallet;
use alloy::signers::local::PrivateKeySigner;

#[cfg(test)]
mod factory;
mod router;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    Pair,
    "../../contracts/out/Pair.sol/Pair.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    MockERC20,
    "../../contracts/out/MockERC20.sol/MockERC20.json"
);

#[tokio::test]
async fn test_pair_init_and_mint() -> Result<()> {
    let anvil = Anvil::new().spawn();
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let my_address = signer.address();
    let wallet = EthereumWallet::from(signer);
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(anvil.endpoint().parse()?);

    let my_address = provider.get_accounts().await?[0];

    let token0 = MockERC20::deploy(&provider, "Token A".into(), "TKNA".into()).await?;
    let token1 = MockERC20::deploy(&provider, "Token B".into(), "TKNB".into()).await?;

    let (addr0, addr1) = if *token0.address() < *token1.address() {
        (*token0.address(), *token1.address())
    } else {
        (*token1.address(), *token0.address())
    };

    let pair = Pair::deploy(provider.clone()).await?;

    let init_tx = pair.initialize(addr0, addr1).send().await?.get_receipt().await?;
    assert!(init_tx.status(), "Initialization failed");

    let t0 = pair.token0().call().await?._0;
    assert_eq!(t0, addr0);

    let deposit_amount = U256::from(10000);

    let tk0 = MockERC20::new(addr0, provider.clone());
    let tk1 = MockERC20::new(addr1, provider.clone());

    tk0.mint(*pair.address(), deposit_amount).send().await?.get_receipt().await?;
    tk1.mint(*pair.address(), deposit_amount).send().await?.get_receipt().await?;

    let mint_tx = pair.mint(my_address).send().await?.get_receipt().await?;
    assert!(mint_tx.status(), "Mint failed");

    let reserves = pair.getReserves().call().await?;
    assert_eq!(reserves._0, 10000, "Reserves 0 must be 10000");
    assert_eq!(reserves._1, 10000, "Reserves 1 must be 10000");

    let my_lp_balance = pair.balanceOf(my_address).call().await?._0;
    assert_eq!(my_lp_balance, U256::from(9000), "LP balance must be 9000");

    println!("liquidity added, LP tokens: {}", my_lp_balance);

    Ok(())
}

#[tokio::test]
async fn test_swap() -> Result<()> {
    let anvil = Anvil::new().spawn();
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let my_address = signer.address();
    let wallet = EthereumWallet::from(signer);

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(anvil.endpoint().parse()?);

    let token0 = MockERC20::deploy(&provider, "Token A".into(), "TKNA".into()).await?;
    let token1 = MockERC20::deploy(&provider, "Token B".into(), "TKNB".into()).await?;

    let (addr0, addr1) = if *token0.address() < *token1.address() {
        (*token0.address(), *token1.address())
    } else {
        (*token1.address(), *token0.address())
    };

    let pair = Pair::deploy(&provider).await?;
    pair.initialize(addr0, addr1).send().await?.get_receipt().await?;

    let tk0 = MockERC20::new(addr0, &provider);
    let tk1 = MockERC20::new(addr1, &provider);

    let initial_liquidity = U256::from(10000);
    tk0.mint(*pair.address(), initial_liquidity).send().await?.get_receipt().await?;
    tk1.mint(*pair.address(), initial_liquidity).send().await?.get_receipt().await?;
    pair.mint(my_address).send().await?.get_receipt().await?;

    let reserves_before = pair.getReserves().call().await?;
    assert_eq!(reserves_before._0, 10000);
    assert_eq!(reserves_before._1, 10000);

    let swap_amount_in = U256::from(1000);
    let expected_amount_out = U256::from(906);

    tk0.mint(*pair.address(), swap_amount_in).send().await?.get_receipt().await?;

    let my_balance_before = tk1.balanceOf(my_address).call().await?._0;

    let swap_tx = pair.swap(U256::ZERO, expected_amount_out, my_address)
        .send()
        .await?
        .get_receipt()
        .await?;

    assert!(swap_tx.status(), "Swap transaction err (maybe PAIR: K)!");

    let my_balance_after = tk1.balanceOf(my_address).call().await?._0;
    assert_eq!(
        my_balance_after,
        my_balance_before + expected_amount_out,
        "Balance incorrect"
    );

    let reserves_after = pair.getReserves().call().await?;

    assert_eq!(reserves_after._0, 11000, "Reserves 0 math incorrect");

    assert_eq!(reserves_after._1, 9094, "Reserves 1 math incorrect");

    println!("Swap success, reservers: {} / {}", reserves_after._0, reserves_after._1);

    Ok(())
}