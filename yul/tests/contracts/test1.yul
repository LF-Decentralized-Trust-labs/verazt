/// @use-src 0:"dao_solidity_v0_8.sol"
object "SimpleDAO_62" {
  code {
    /// @src 0:162:618  "contract SimpleDAO {..."
    mstore(64, 128)
    let a := allocate_unbounded()
    {
      let a := allocate_unbounded()
    }

    let b := a

    a := allocate_unbounded()

    let c := b

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

    function func_no_params_with_return() -> memPtr {
      memPtr := mload(64)
    }
  }
}
