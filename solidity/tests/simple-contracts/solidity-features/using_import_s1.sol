library L {
    function f(uint256 x) pure public returns (uint256) {
        return x + 2;
    }
}

library L2 {
    function bar(uint256 x) pure public returns (uint256) {
        return x + 2;
    }
}

library L3 {
    function zar(uint256 x) pure public returns (uint256) {
        return x + 2;
    }
}

function g(uint256 x) pure returns (uint256) {
    return x + 8;
}
