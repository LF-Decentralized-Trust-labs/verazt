// SPDX-License-Identifier: MIT
pragma solidity 0.7.6;

// pragma experimental ABIEncoderV2;
contract BugSample {
    uint256 uzero = 0;
    uint256 umax = 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;
    uint256 guards = 1;
    address winner;
    address owner;
    bool reentrancy_guard = false;

    constructor() {
        owner = msg.sender;
    }

    function overflow_add(uint256 b) public view returns (uint256) {
        // <yes> <report> ARITHMETIC_BUG
        return umax + b;
    }

    function underflow_minus(uint256 b) public view returns (uint256) {
        // <yes> <report> ARITHMETIC_BUG
        return uzero - b;
    }
    function reentrancy_withdraw() public {
        // <yes> <report> REENTRANCY
        require(guards > 0, "Must have some guards left");
        (bool success, ) = payable(msg.sender).call{value: 1 ether}(""); // REENTRANCY, UNCHECKED_SEND
        require(success);
        guards = 0;
    }

    function assert_failure(uint256 _umax) public payable {
        // <yes> <report> ASSERTION_FAILURE
        assert(_umax > 0);
        umax = _umax;
    }

    function guess(uint256 i) public payable {
        require(msg.value > 0, "Must pay to play");
        // <yes> <report> BLOCK_DEPENDENCY
        if (block.timestamp % i == 89) {
            winner = msg.sender;
        }
        // <yes> <report> BLOCK_DEPENDENCY
        if (block.number % i == 97) {
            winner = msg.sender;
        }
    }

    function exception(address addr) public {
        // <yes> <report> UNHANDLED_EXCEPTION
        address(addr).call("0x1234"); // EXCEPTION_DISORDER, ADDRESS_VALIDATION
    }

    function kill() public {
        // <yes> <report> UNSAFE_SELFDESTRUCT
        selfdestruct(msg.sender); // UNPROTECTED_SELFDESTRUCT
    }

    bool lock1;
    bool lock2;

    function unlock1 () public {
      lock1 = true;
    }

    function unlock2 () public {
      lock2 = true;
    }

    function three_step_bug() public {
        require(lock1);
        require(lock2);
      // <yes> <report> UNSAFE_SELFDESTRUCT
      selfdestruct(msg.sender); // UNCHECKED_SEND
    }

    function test() public {
        require(lock1);
        require(lock2);
        unlock1();
        unlock2();
    }
}
