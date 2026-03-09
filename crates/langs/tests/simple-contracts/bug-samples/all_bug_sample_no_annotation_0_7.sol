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

    function add_max(uint256 b) public view returns (uint256) {
        return umax + b;
    }

    function zero_minus(uint256 b) public view returns (uint256) {
        return uzero - b;
    }
    function withdraw() public {
        require(guards > 0, "Must have some guards left");
        (bool success, ) = payable(msg.sender).call{value: 1 ether}("");
        require(success);
        guards = 0;
    }

    function assert_failure(uint256 _umax) public payable {
        assert(_umax > 0);
        umax = _umax;
    }

    function guess(uint256 i) public payable {
        require(msg.value > 0, "Must pay to play");
        if (block.timestamp % i == 89) {
            winner = msg.sender;
        }
        if (block.number % i == 97) {
            winner = msg.sender;
        }
    }

    function exception(address addr) public {
        address(addr).call("0x1234");
    }

    function kill() public {
        selfdestruct(msg.sender);
    }

    bool lock1;
    bool lock2;

    function unlock1 () public {
      lock1 = true;
    }

    function unlock2 () public {
      lock2 = true;
    }

    function destruct() public {
        require(lock1);
        require(lock2);
        selfdestruct(msg.sender);
    }

    function test() public {
        require(lock1);
        require(lock2);
        unlock1();
        unlock2();
    }
}
