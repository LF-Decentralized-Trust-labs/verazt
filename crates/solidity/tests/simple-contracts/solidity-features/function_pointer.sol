pragma abicoder v2;

contract C {
    bool sideEffectRan = false;

    function (uint256, string memory) external fPointer;

    function fExternal(uint256 p, string memory t) external {}

    function fLocalPointerCall() public returns (bytes memory) {
        function (uint256, string memory) external localFunctionPointer = this.fExternal;
        return abi.encodeCall(localFunctionPointer, (1, hex"313233"));
    }

}
