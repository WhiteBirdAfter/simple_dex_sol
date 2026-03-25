use alloy::node_bindings::Anvil;
use alloy::primitives::U256;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::sol;
use eyre::Result;
use alloy::network::EthereumWallet;
use alloy::signers::local::PrivateKeySigner;

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