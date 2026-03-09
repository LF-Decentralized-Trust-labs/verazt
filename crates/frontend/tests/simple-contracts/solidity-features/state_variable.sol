enum Direction {
    A,
    B,
    C
}

contract A {
    // State variable
    Direction public a;

    function f(Direction b) public view returns (bool) {
        return (a == b);
    }
}

contract B {
    Direction internal b;
    A internal c;

    function f(Direction w) public view returns (bool) {
        // Access to the state variable of contract `c` through the getter
        if (c.a() == w) {
            return true;
        }

        if (c.f(b)) {
            return true;
        }

        return (b == w);
    }
}
