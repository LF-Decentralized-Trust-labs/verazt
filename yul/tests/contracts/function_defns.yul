/// Yul contract for testing function definitions.

object "Simple" {
  code {
    mstore(64, 128)

    let a := allocate_unbounded()

    function allocate_unbounded() -> memPtr {
      memPtr := mload(64)
    }

    function func_no_params_no_returns() {
      memPtr := mload(64)
    }

    function func_with_params_with_returns(a, b) -> c {
      c := add(a, b)
    }

    function func_with_params_no_returns(a, b) {
      let c := sub(a, b)
    }

    function func_no_params_with_return() memPtr {
      memPtr := mload(64)
    }
  }
}
