// SPDX-License-Identifier: MIT

pragma solidity ^0.8;

contract Counter {
    uint public number;

    function increment() external {
        number += 1;
    }
}

contract CounterCaller {
    Counter public myCounter;

    constructor(address counterAddress) {
        myCounter = Counter(counterAddress);
    }

    function counterIncrement() external {
        myCounter.increment();
    }
}
