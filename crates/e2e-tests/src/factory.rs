use alloy::network::EthereumWallet;
use alloy::node_bindings::Anvil;
use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use eyre::Result;

alloy::sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    Factory,
    "../../contracts/out/Factory.sol/Factory.json"
);

alloy::sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    MockERC20,
    "../../contracts/out/MockERC20.sol/MockERC20.json"
);

#[tokio::test]
async fn test_factory_core_logic() -> Result<()> {
    let anvil = Anvil::new().spawn();
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet = EthereumWallet::from(signer);

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(anvil.endpoint().parse()?);

    let factory = Factory::deploy(&provider).await?;

    let token_a = MockERC20::deploy(&provider, "Token A".into(), "TKNA".into()).await?;
    let token_b = MockERC20::deploy(&provider, "Token B".into(), "TKNB".into()).await?;

    let addr_a = *token_a.address();
    let addr_b = *token_b.address();

    let initial_length = factory.allPairsLength().call().await?._0;
    assert_eq!(initial_length, U256::ZERO, "Fabric must be empty");

    let receipt = factory.createPair(addr_a, addr_b)
        .send()
        .await?
        .get_receipt()
        .await?;
    assert!(receipt.status(), "Transaction createPair err");

    let length_after = factory.allPairsLength().call().await?._0;
    assert_eq!(length_after, U256::from(1), "There is no new pool in allPairs ");

    let pair_address = factory.allPairs(U256::ZERO).call().await?._0;
    assert_ne!(pair_address, Address::ZERO, "Pool address is zero");

    let pair_ab = factory.getPair(addr_a, addr_b).call().await?._0;
    let pair_ba = factory.getPair(addr_b, addr_a).call().await?._0;

    assert_eq!(pair_ab, pair_address, "Pair A->B broken");
    assert_eq!(pair_ba, pair_address, "Pair B->A broken");

    // try to add duplicate pool
    let call_builder = factory.createPair(addr_a, addr_b);
    let duplicate_tx = call_builder.send().await;



    assert!(
        duplicate_tx.is_err() || !duplicate_tx.unwrap().get_receipt().await?.status(),
        "The factory allowed the creation of a duplicate pool. Err"
    );

    println!(
        "Success. Pool for {} and {} created here: {}",
        addr_a, addr_b, pair_address
    );

    Ok(())
}