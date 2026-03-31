use alloy::network::EthereumWallet;
use alloy::node_bindings::Anvil;
use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::{local::PrivateKeySigner, Signer};
use alloy::sol;
use eyre::Result;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    Router,
    "../../contracts/out/Router.sol/Router.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    Factory,
    "../../contracts/out/Factory.sol/Factory.json"
);

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
async fn test_router_add_liquidity() -> Result<()> {
    let anvil = Anvil::new().spawn();
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let my_address = signer.address();
    let wallet = EthereumWallet::from(signer);

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(anvil.endpoint().parse()?);

    // Используем &provider вместо provider.clone()
    let factory = Factory::deploy(&provider).await?;
    let factory_addr = *factory.address();

    let router = Router::deploy(&provider, factory_addr).await?;
    let router_addr = *router.address();

    let supply = U256::from(1_000_000u64) * U256::from(10u64).pow(U256::from(18u64));

    let token_a = MockERC20::deploy(
        &provider,
        "TokenA".to_string(),
        "TKA".to_string(),
    ).await?;

    let token_b = MockERC20::deploy(
        &provider,
        "TokenB".to_string(),
        "TKB".to_string(),
    ).await?;

    let token_a_addr = *token_a.address();
    let token_b_addr = *token_b.address();

    let builder = token_a.mint(my_address, supply);
    let pending = builder.send().await?;
    let _ = pending.get_receipt().await?;

    let builder = token_b.mint(my_address, supply);
    let pending = builder.send().await?;
    let _ = pending.get_receipt().await?;

    let builder = factory.createPair(token_a_addr, token_b_addr);
    let pending = builder.send().await?;
    let _ = pending.get_receipt().await?;

    let pair_addr: Address = factory.getPair(token_a_addr, token_b_addr).call().await?._0;
    assert_ne!(pair_addr, Address::ZERO);

    let pair = Pair::new(pair_addr, &provider);

    let amount_a_desired = U256::from(1_000u64) * U256::from(10u64).pow(U256::from(18u64));
    let amount_b_desired = U256::from(2_000u64) * U256::from(10u64).pow(U256::from(18u64));

    let builder = token_a.approve(router_addr, amount_a_desired);
    let pending = builder.send().await?;
    let _ = pending.get_receipt().await?;

    let builder = token_b.approve(router_addr, amount_b_desired);
    let pending = builder.send().await?;
    let _ = pending.get_receipt().await?;

    let deadline = U256::from(9_999_999_999u64);

    let builder = router.addLiquidity(
        token_a_addr,
        token_b_addr,
        amount_a_desired,
        amount_b_desired,
        U256::ZERO,
        U256::ZERO,
        my_address,
        deadline,
    );
    let pending = builder.send().await?;
    let _ = pending.get_receipt().await?;

    let reserves = pair.getReserves().call().await?;
    assert!(reserves._0 > 0);
    assert!(reserves._1 > 0);

    let lp_balance = pair.balanceOf(my_address).call().await?._0;
    assert!(lp_balance > U256::ZERO);

    let pair_balance_a = token_a.balanceOf(pair_addr).call().await?._0;
    let pair_balance_b = token_b.balanceOf(pair_addr).call().await?._0;

    assert!(pair_balance_a > U256::ZERO);
    assert!(pair_balance_b > U256::ZERO);

    Ok(())
}

#[tokio::test]
async fn test_router_remove_liquidity() -> Result<()> {
    let anvil = Anvil::new().spawn();
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let my_address = signer.address();
    let wallet = EthereumWallet::from(signer);

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(anvil.endpoint().parse()?);

    let factory = Factory::deploy(&provider).await?;
    let factory_addr = *factory.address();

    let router = Router::deploy(&provider, factory_addr).await?;
    let router_addr = *router.address();

    let supply = U256::from(1_000_000u64) * U256::from(10u64).pow(U256::from(18u64));

    let token_a = MockERC20::deploy(
        &provider,
        "TokenA".to_string(),
        "TKA".to_string(),
    ).await?;

    let token_b = MockERC20::deploy(
        &provider,
        "TokenB".to_string(),
        "TKB".to_string(),
    ).await?;

    let token_a_addr = *token_a.address();
    let token_b_addr = *token_b.address();

    let builder = token_a.mint(my_address, supply);
    let pending = builder.send().await?;
    let _ = pending.get_receipt().await?;

    let builder = token_b.mint(my_address, supply);
    let pending = builder.send().await?;
    let _ = pending.get_receipt().await?;

    let builder = factory.createPair(token_a_addr, token_b_addr);
    let pending = builder.send().await?;
    let _ = pending.get_receipt().await?;

    let pair_addr: Address = factory.getPair(token_a_addr, token_b_addr).call().await?._0;
    assert_ne!(pair_addr, Address::ZERO);

    let pair = Pair::new(pair_addr, &provider);

    let amount_a_desired = U256::from(1_000u64) * U256::from(10u64).pow(U256::from(18u64));
    let amount_b_desired = U256::from(2_000u64) * U256::from(10u64).pow(U256::from(18u64));

    let builder = token_a.approve(router_addr, amount_a_desired);
    let pending = builder.send().await?;
    let _ = pending.get_receipt().await?;

    let builder = token_b.approve(router_addr, amount_b_desired);
    let pending = builder.send().await?;
    let _ = pending.get_receipt().await?;

    let deadline = U256::from(9_999_999_999u64);

    let builder = router.addLiquidity(
        token_a_addr,
        token_b_addr,
        amount_a_desired,
        amount_b_desired,
        U256::ZERO,
        U256::ZERO,
        my_address,
        deadline,
    );
    let pending = builder.send().await?;
    let _ = pending.get_receipt().await?;

    let lp_balance_before = pair.balanceOf(my_address).call().await?._0;
    assert!(lp_balance_before > U256::ZERO);

    let token_a_before = token_a.balanceOf(my_address).call().await?._0;
    let token_b_before = token_b.balanceOf(my_address).call().await?._0;

    let builder = pair.approve(router_addr, lp_balance_before);
    let pending = builder.send().await?;
    let _ = pending.get_receipt().await?;

    let builder = router.removeLiquidity(
        token_a_addr,
        token_b_addr,
        lp_balance_before,
        U256::ZERO,
        U256::ZERO,
        my_address,
        deadline,
    );
    let pending = builder.send().await?;
    let _ = pending.get_receipt().await?;

    let lp_balance_after = pair.balanceOf(my_address).call().await?._0;
    assert_eq!(lp_balance_after, U256::ZERO);

    let token_a_after = token_a.balanceOf(my_address).call().await?._0;
    let token_b_after = token_b.balanceOf(my_address).call().await?._0;

    assert!(token_a_after > token_a_before);
    assert!(token_b_after > token_b_before);

    let reserves = pair.getReserves().call().await?;
    assert!(
        U256::from(reserves._0) < U256::from(10_000u64),
        "Reserve 0 must be less than 10000"
    );
    assert!(
        U256::from(reserves._1) < U256::from(10_000u64),
        "Reserve 1 must be less than 10000"
    );

    Ok(())
}

#[tokio::test]
async fn test_router_swap() -> Result<()> {
    let anvil = Anvil::new().spawn();
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let my_address = signer.address();
    let wallet = EthereumWallet::from(signer);

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(anvil.endpoint().parse()?);

    let factory = Factory::deploy(&provider).await?;
    let factory_addr = *factory.address();

    let router = Router::deploy(&provider, factory_addr).await?;
    let router_addr = *router.address();

    let supply = U256::from(1_000_000u64) * U256::from(10u64).pow(U256::from(18u64));

    let token_a = MockERC20::deploy(
        &provider,
        "TokenA".to_string(),
        "TKA".to_string(),
    ).await?;

    let token_b = MockERC20::deploy(
        &provider,
        "TokenB".to_string(),
        "TKB".to_string(),
    ).await?;

    let token_a_addr = *token_a.address();
    let token_b_addr = *token_b.address();

    let builder = token_a.mint(my_address, supply);
    let pending = builder.send().await?;
    let _ = pending.get_receipt().await?;

    let builder = token_b.mint(my_address, supply);
    let pending = builder.send().await?;
    let _ = pending.get_receipt().await?;

    let builder = factory.createPair(token_a_addr, token_b_addr);
    let pending = builder.send().await?;
    let _ = pending.get_receipt().await?;

    let pair_addr: Address = factory.getPair(token_a_addr, token_b_addr).call().await?._0;
    assert_ne!(pair_addr, Address::ZERO);

    let pair = Pair::new(pair_addr, &provider);

    let amount_a_desired = U256::from(10_000u64) * U256::from(10u64).pow(U256::from(18u64));
    let amount_b_desired = U256::from(20_000u64) * U256::from(10u64).pow(U256::from(18u64));

    let builder = token_a.approve(router_addr, amount_a_desired);
    let pending = builder.send().await?;
    let _ = pending.get_receipt().await?;

    let builder = token_b.approve(router_addr, amount_b_desired);
    let pending = builder.send().await?;
    let _ = pending.get_receipt().await?;

    let deadline = U256::from(9_999_999_999u64);

    let builder = router.addLiquidity(
        token_a_addr,
        token_b_addr,
        amount_a_desired,
        amount_b_desired,
        U256::ZERO,
        U256::ZERO,
        my_address,
        deadline,
    );
    let pending = builder.send().await?;
    let _ = pending.get_receipt().await?;

    let amount_in = U256::from(100u64) * U256::from(10u64).pow(U256::from(18u64));

    let token_a_before = token_a.balanceOf(my_address).call().await?._0;
    let token_b_before = token_b.balanceOf(my_address).call().await?._0;

    let builder = token_a.approve(router_addr, amount_in);
    let pending = builder.send().await?;
    let _ = pending.get_receipt().await?;

    let builder = router.swapExactTokensForTokens(
        token_a_addr,
        token_b_addr,
        amount_in,
        U256::ZERO,
        my_address,
        deadline,
    );
    let pending = builder.send().await?;
    let _ = pending.get_receipt().await?;

    let token_a_after = token_a.balanceOf(my_address).call().await?._0;
    let token_b_after = token_b.balanceOf(my_address).call().await?._0;

    assert!(token_a_after < token_a_before);
    assert!(token_b_after > token_b_before);

    let reserves = pair.getReserves().call().await?;
    assert!(reserves._0 > 0);
    assert!(reserves._1 > 0);

    Ok(())
}