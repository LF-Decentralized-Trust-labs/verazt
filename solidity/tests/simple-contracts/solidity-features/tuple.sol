pragma solidity ^0.8.0;

contract C {
    function test_tuple() internal pure {
        uint a;
        uint b;

        (a, b) = (1, 2);
    }

}
