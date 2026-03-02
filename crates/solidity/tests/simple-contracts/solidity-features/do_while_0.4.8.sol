pragma solidity 0.4.8;

contract C {
    function f() public {
        uint x;
        do {
            break;
            x = 1;
        } while (x == 0);
    }
}
