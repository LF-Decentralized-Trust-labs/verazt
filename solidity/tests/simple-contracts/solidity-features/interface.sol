interface IContractA {
    uint public x;
}

contract A is IContractA {}

contract B {
    function getContractAState() public returns (uint256) {
        A a;
        IContractA contractA = IContractA(a);
        uint256 y = contractA.x();
        return y;
    }
}
