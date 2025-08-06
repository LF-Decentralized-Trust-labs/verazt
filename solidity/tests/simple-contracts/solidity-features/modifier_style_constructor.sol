contract A {
    constructor() {}
}

contract B is A {
    // A is a modifier invocation.
    constructor() A {}
}
