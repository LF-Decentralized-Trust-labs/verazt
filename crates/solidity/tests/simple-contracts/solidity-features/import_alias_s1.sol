uint256 constant a = 89;

enum Color {
    Red,
    Blue
}

struct Editor {
    string name;
    uint256 version;
}

function fre() pure returns (uint256) {
    return a;
}

contract A {
    function foo() public pure returns (uint256) {
        return 1;
    }
}
