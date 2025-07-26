contract C {
    function h() view public {
        assembly {
            function func_with_params_with_returns(a, b) -> c {
                c := add(a, b)
            }
        }
    }
}
