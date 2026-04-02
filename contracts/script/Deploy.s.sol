// SPDX-License-Identifier: MIT
pragma solidity ^0.8.33;

import "forge-std/Script.sol";
import "../src/Factory.sol";
import "../src/Router.sol";
import "../src/MockERC20.sol";

contract DeployScript is Script {
    function run() external{
        // anvil[0] key here
        uint256 deployerPrivateKey = 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80;
        address deployerAddress = vm.addr(deployerPrivateKey);
        vm.startBroadcast(deployerPrivateKey);
        Factory factory = new Factory();
        console.log("Factory: ", address(factory));

        Router router = new Router(address(factory));
        console.log("Router: ", address(router));

        uint256 supply = 1_000_000 * 10**18;
        MockERC20 tokenA = new MockERC20("Token A", "TKA");
        MockERC20 tokenB = new MockERC20("Token B", "TKB");

        tokenA.mint(deployerAddress, supply);
        tokenB.mint(deployerAddress, supply);

        console.log("Token A: ", address(tokenA));
        console.log("Token B: ", address(tokenB));

        vm.stopBroadcast();

    }
}