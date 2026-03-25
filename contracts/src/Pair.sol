// SPDX-License-Identifier: MIT
pragma solidity ^0.8.33;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract Pair is ERC20 {
    using SafeERC20 for IERC20;

    event Mint(address indexed sender, uint256 amount0, uint256 amount1);
    event Burn(address indexed sender, uint256 amount0, uint256 amount1, address indexed to);
    event Swap(
        address indexed sender,
        uint256 amount0In,
        uint256 amount1In,
        uint256 amount0Out,
        uint256 amount1Out,
        address indexed to
    );
    event Sync(uint112 reserve0, uint112 reserve1);

    address constant DEAD = address(0xdead);

    address public immutable factory;

    address public token0;
    address public token1;

    uint112 private reserve0;
    uint112 private reserve1;

    uint256 public constant MINIMUM_LIQUIDITY = 10**3;

    bool private initialized;

    constructor() ERC20("Simple LP Token", "SLP") {
        factory = msg.sender;
    }

    modifier onlyFactory() {
        require(msg.sender == factory, "PAIR: FORBIDDEN");
        _;
    }

    function initialize(address _token0, address _token1) external onlyFactory {
        require(!initialized, "PAIR: ALREADY_INITIALIZED");
        require(_token0 != _token1, "PAIR: IDENTICAL_ADDRESSES");
        require(_token0 != address(0) && _token1 != address(0), "PAIR: ZERO_ADDRESS");

        token0 = _token0;
        token1 = _token1;
        initialized = true;
    }

    function getReserves() public view returns (uint112, uint112) {
        return (reserve0, reserve1);
    }

    function _update(uint256 balance0, uint256 balance1) private {
        require(balance0 <= type(uint112).max, "PAIR: RESERVE0_OVERFLOW");
        require(balance1 <= type(uint112).max, "PAIR: RESERVE1_OVERFLOW");

        reserve0 = uint112(balance0);
        reserve1 = uint112(balance1);

        emit Sync(reserve0, reserve1);
    }

    function _min(uint256 x, uint256 y) private pure returns (uint256) {
        return x < y ? x : y;
    }

    function _sqrt(uint256 y) private pure returns (uint256 z) {
        if (y > 3) {
            z = y;
            uint256 x = y / 2 + 1;
            while (x < z) {
                z = x;
                x = (y / x + x) / 2;
            }
        } else if (y != 0) {
            z = 1;
        }
    }

    function mint(address to) external returns (uint256 liquidity) {
        require(to != address(0), "PAIR: ZERO_TO");

        (uint112 _reserve0, uint112 _reserve1) = getReserves();

        uint256 balance0 = IERC20(token0).balanceOf(address(this));
        uint256 balance1 = IERC20(token1).balanceOf(address(this));

        uint256 amount0 = balance0 - _reserve0;
        uint256 amount1 = balance1 - _reserve1;

        uint256 _totalSupply = totalSupply();

        if (_totalSupply == 0) {
            uint256 rootK = _sqrt(amount0 * amount1);
            require(rootK > MINIMUM_LIQUIDITY, "PAIR: INSUFFICIENT_INITIAL_LIQUIDITY");

            liquidity = rootK - MINIMUM_LIQUIDITY;

            _mint(address(DEAD), MINIMUM_LIQUIDITY);
        } else {
            uint256 liquidity0 = (amount0 * _totalSupply) / _reserve0;
            uint256 liquidity1 = (amount1 * _totalSupply) / _reserve1;
            liquidity = _min(liquidity0, liquidity1);
        }

        require(liquidity > 0, "PAIR: INSUFFICIENT_LIQUIDITY_MINTED");

        _mint(to, liquidity);
        _update(balance0, balance1);

        emit Mint(msg.sender, amount0, amount1);
    }


    function burn(address to) external returns (uint256 amount0, uint256 amount1) {
        require(to != address(0), "PAIR: ZERO_TO");


        uint256 balance0 = IERC20(token0).balanceOf(address(this));
        uint256 balance1 = IERC20(token1).balanceOf(address(this));

        uint256 liquidity = balanceOf(address(this));
        uint256 _totalSupply = totalSupply();

        require(liquidity > 0, "PAIR: NO_LIQUIDITY_TO_BURN");
        require(_totalSupply > 0, "PAIR: ZERO_TOTAL_SUPPLY");

        amount0 = (liquidity * balance0) / _totalSupply;
        amount1 = (liquidity * balance1) / _totalSupply;

        require(amount0 > 0 && amount1 > 0, "PAIR: INSUFFICIENT_LIQUIDITY_BURNED");

        _burn(address(this), liquidity);

        IERC20(token0).safeTransfer(to, amount0);
        IERC20(token1).safeTransfer(to, amount1);

        balance0 = IERC20(token0).balanceOf(address(this));
        balance1 = IERC20(token1).balanceOf(address(this));

        _update(balance0, balance1);

        emit Burn(msg.sender, amount0, amount1, to);
    }

    function swap(uint256 amount0Out, uint256 amount1Out, address to) external {
        require(amount0Out > 0 || amount1Out > 0, "PAIR: INSUFFICIENT_OUTPUT_AMOUNT");
        require(to != address(0), "PAIR: ZERO_TO");

        (uint112 _reserve0, uint112 _reserve1) = getReserves();

        require(amount0Out < _reserve0 && amount1Out < _reserve1, "PAIR: INSUFFICIENT_LIQUIDITY");

        require(to != token0 && to != token1, "PAIR: INVALID_TO");

        if (amount0Out > 0) IERC20(token0).safeTransfer(to, amount0Out);
        if (amount1Out > 0) IERC20(token1).safeTransfer(to, amount1Out);

        uint256 balance0 = IERC20(token0).balanceOf(address(this));
        uint256 balance1 = IERC20(token1).balanceOf(address(this));

        uint256 amount0In = balance0 > (_reserve0 - amount0Out)
            ? balance0 - (_reserve0 - amount0Out)
            : 0;
        uint256 amount1In = balance1 > (_reserve1 - amount1Out)
            ? balance1 - (_reserve1 - amount1Out)
            : 0;

        require(amount0In > 0 || amount1In > 0, "PAIR: INSUFFICIENT_INPUT_AMOUNT");

        uint256 balance0Adjusted = (balance0 * 1000) - (amount0In * 3);
        uint256 balance1Adjusted = (balance1 * 1000) - (amount1In * 3);

        require(
            balance0Adjusted * balance1Adjusted >= uint256(_reserve0) * uint256(_reserve1) * 1000**2,
            "PAIR: K"
        );

        _update(balance0, balance1);

        emit Swap(msg.sender, amount0In, amount1In, amount0Out, amount1Out, to);
    }


}