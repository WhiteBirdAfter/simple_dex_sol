// SPDX-License-Identifier: MIT
pragma solidity ^0.8.33;
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "./Factory.sol";
import "./Pair.sol";

contract Router{
    using SafeERC20 for IERC20;
    address private immutable factory;

    constructor(address _factory){
        require(_factory != address(0), "ROUTER: FACTORY_ADDRESS_IS_NULL");
        factory = _factory;
    }

    error Expired();
    error PairNotFound();
    error InsufficientAAmount();
    error InsufficientBAmount();

    function quote(
        uint256 amountA,
        uint256 reserveA,
        uint256 reserveB
    ) public pure returns (uint256 amountB){
        require(amountA > 0, "ROUTER: INSUFFICIENT_AMOUNT");
        require(reserveA > 0 && reserveB > 0, "ROUTER: INSUFFICIENT_LIQUIDITY");
        amountB = (amountA * reserveB) / reserveA;
    }

    function getAmountOut(
        uint256 amountIn,
        uint256 reserveIn,
        uint256 reserveOut
    ) public pure returns(uint256 amountOut){
        require(amountIn > 0, "ROUTER: INSUFFICIENT_INPUT_AMOUNT");
        require(reserveIn > 0 && reserveOut > 0, "ROUTER: INSUFFICIENT_LIQUIDITY");
        uint256 amountInWithFee = amountIn * 997;
        uint256 numerator = amountInWithFee * reserveOut;
        uint256 denominator = (reserveIn * 1000) + amountInWithFee;
        amountOut = numerator / denominator;
    }

    function _getPair(address tokenA, address tokenB) internal view returns(address pair){
        pair = Factory(factory).getPair(tokenA, tokenB);
        if (pair == address(0)) revert PairNotFound();
    }

    function _sortTokens(address tokenA, address tokenB) internal pure returns (address token0, address token1) {
        require(tokenA != tokenB, "ROUTER: IDENTICAL_ADDRESSES");
        (token0, token1) = tokenA < tokenB ? (tokenA, tokenB) : (tokenB, tokenA);
    }
    function _getReserves(address tokenA, address tokenB) internal view returns (uint256 reserveA, uint256 reserveB) {
        address pair = _getPair(tokenA, tokenB);
        (uint112 reserve0, uint112 reserve1) = Pair(pair).getReserves();
        (address token0,) = _sortTokens(tokenA, tokenB);
        (reserveA, reserveB) = tokenA == token0
            ? (reserve0, reserve1)
            : (reserve1, reserve0);
    }
    function _addLiquidity(
        address tokenA,
        address tokenB,
        uint256 amountADesired,
        uint256 amountBDesired,
        uint256 amountAMin,
        uint256 amountBMin
    ) internal view returns (uint256 amountA, uint256 amountB) {
        (uint256 reserveA, uint256 reserveB) = _getReserves(tokenA, tokenB);

        if (reserveA == 0 && reserveB == 0) {

            (amountA, amountB) = (amountADesired, amountBDesired);

        } else {
            uint256 amountBOptimal = quote(amountADesired, reserveA, reserveB);
            
            if (amountBOptimal <= amountBDesired) {
                if (amountBOptimal < amountBMin) revert InsufficientBAmount();
                (amountA, amountB) = (amountADesired, amountBOptimal);
            } else {
                uint256 amountAOptimal = quote(amountBDesired, reserveB, reserveA);
                if (amountAOptimal < amountAMin) revert InsufficientAAmount();
                (amountA, amountB) = (amountAOptimal, amountBDesired);
            }
        }
    }


    function addLiquidity(
        address tokenA,
        address tokenB,
        uint256 amountADesired,
        uint256 amountBDesired,
        uint256 amountAMin,
        uint256 amountBMin,
        address to,
        uint256 deadline
    ) external returns (uint256 amountA, uint256 amountB, uint256 liquidity) {
        if (block.timestamp > deadline) revert Expired();
        require(to != address(0), "ROUTER: ZERO_TO");

        address pair = _getPair(tokenA, tokenB);

        (amountA, amountB) = _addLiquidity(
            tokenA, tokenB, amountADesired, amountBDesired, amountAMin, amountBMin
        );

        IERC20(tokenA).safeTransferFrom(msg.sender, pair, amountA);
        IERC20(tokenB).safeTransferFrom(msg.sender, pair, amountB);

        liquidity = Pair(pair).mint(to);
    }
    function removeLiquidity(
        address tokenA,
        address tokenB,
        uint256 liquidity,
        uint256 amountAMin,
        uint256 amountBMin,
        address to,
        uint256 deadline
    ) external returns (uint256 amountA, uint256 amountB) {
        if (block.timestamp > deadline) revert Expired();
        require(to != address(0), "ROUTER: ZERO_TO");

        address pair = _getPair(tokenA, tokenB);

        IERC20(pair).safeTransferFrom(msg.sender, pair, liquidity);

        (uint256 amount0, uint256 amount1) = Pair(pair).burn(to);

        (address token0,) = _sortTokens(tokenA, tokenB);
        (amountA, amountB) = tokenA == token0
            ? (amount0, amount1)
            : (amount1, amount0);

        require(amountA >= amountAMin, "ROUTER: INSUFFICIENT_A_AMOUNT");
        require(amountB >= amountBMin, "ROUTER: INSUFFICIENT_B_AMOUNT");
    }
    function swapExactTokensForTokens(
        address tokenIn,
        address tokenOut,
        uint256 amountIn,
        uint256 amountOutMin,
        address to,
        uint256 deadline
    ) external returns (uint256 amountOut) {
        if (block.timestamp > deadline) revert Expired();
        require(to != address(0), "ROUTER: ZERO_TO");

        address pair = _getPair(tokenIn, tokenOut);
        (uint256 reserveIn, uint256 reserveOut) = _getReserves(tokenIn, tokenOut);

        amountOut = getAmountOut(amountIn, reserveIn, reserveOut);
        
        require(amountOut >= amountOutMin, "ROUTER: INSUFFICIENT_OUTPUT_AMOUNT");

        IERC20(tokenIn).safeTransferFrom(msg.sender, pair, amountIn);

        (address token0,) = _sortTokens(tokenIn, tokenOut);

        uint256 amount0Out;
        uint256 amount1Out;

        if (tokenIn == token0) {
            amount0Out = 0;
            amount1Out = amountOut;
        } else {
            amount0Out = amountOut;
            amount1Out = 0;
        }
        Pair(pair).swap(amount0Out, amount1Out, to);
    }

}