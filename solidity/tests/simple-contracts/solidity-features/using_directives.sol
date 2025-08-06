library SomeLibrary {
    function add(uint256 a, uint256 b) public returns (uint256) {
        return a + b;
    }
}

function add2(uint256 a, uint256 b) returns (uint256) {
    return a + b;
}

contract SomeContract {
    using SomeLibrary for *;
    using { add2  } for uint256;

    function add3(uint256 number) public returns (uint256) {
        uint16 b = 1;
        b.add(3);

        uint256 a = number.add(3);
        return a.add2(3);
    }
}
