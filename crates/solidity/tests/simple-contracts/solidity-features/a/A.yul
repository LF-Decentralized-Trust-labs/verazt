
/// @use-src 0:"contract_to_base_override.sol"
object "A_9" {
    code {
        /// @src 0:40:129  "contract A {..."
        mstore(64, memoryguard(128))
        if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }

        constructor_A_9()

        let _1 := allocate_unbounded()
        codecopy(_1, dataoffset("A_9_deployed"), datasize("A_9_deployed"))

        return(_1, datasize("A_9_deployed"))

        function allocate_unbounded() -> memPtr {
            memPtr := mload(64)
        }

        function revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() {
            revert(0, 0)
        }

        /// @src 0:40:129  "contract A {..."
        function constructor_A_9() {

            /// @src 0:40:129  "contract A {..."

        }
        /// @src 0:40:129  "contract A {..."

    }
    /// @use-src 0:"contract_to_base_override.sol"
    object "A_9_deployed" {
        code {
            /// @src 0:40:129  "contract A {..."
            mstore(64, memoryguard(128))

            if iszero(lt(calldatasize(), 4))
            {
                let selector := shift_right_224_unsigned(calldataload(0))
                switch selector

                case 0xc2985578
                {
                    // foo()

                    external_fun_foo_8()
                }

                default {}
            }

            revert_error_42b3090547df1d2001c96683413b8cf91c1b902ef5e3cb8d9f6f304cf7446f74()

            function shift_right_224_unsigned(value) -> newValue {
                newValue :=

                shr(224, value)

            }

            function allocate_unbounded() -> memPtr {
                memPtr := mload(64)
            }

            function revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() {
                revert(0, 0)
            }

            function revert_error_dbdddcbe895c83990c08b3492a0e83918d802a52331272ac6fdb6a7c4aea3b1b() {
                revert(0, 0)
            }

            function abi_decode_tuple_(headStart, dataEnd)   {
                if slt(sub(dataEnd, headStart), 0) { revert_error_dbdddcbe895c83990c08b3492a0e83918d802a52331272ac6fdb6a7c4aea3b1b() }

            }

            function cleanup_t_uint256(value) -> cleaned {
                cleaned := value
            }

            function abi_encode_t_uint256_to_t_uint256_fromStack(value, pos) {
                mstore(pos, cleanup_t_uint256(value))
            }

            function abi_encode_tuple_t_uint256__to_t_uint256__fromStack(headStart , value0) -> tail {
                tail := add(headStart, 32)

                abi_encode_t_uint256_to_t_uint256_fromStack(value0,  add(headStart, 0))

            }

            function external_fun_foo_8() {

                if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }
                abi_decode_tuple_(4, calldatasize())
                let ret_0 :=  fun_foo_8()
                let memPos := allocate_unbounded()
                let memEnd := abi_encode_tuple_t_uint256__to_t_uint256__fromStack(memPos , ret_0)
                return(memPos, sub(memEnd, memPos))

            }

            function revert_error_42b3090547df1d2001c96683413b8cf91c1b902ef5e3cb8d9f6f304cf7446f74() {
                revert(0, 0)
            }

            function zero_value_for_split_t_uint256() -> ret {
                ret := 0
            }

            function cleanup_t_rational_1_by_1(value) -> cleaned {
                cleaned := value
            }

            function identity(value) -> ret {
                ret := value
            }

            function convert_t_rational_1_by_1_to_t_uint256(value) -> converted {
                converted := cleanup_t_uint256(identity(cleanup_t_rational_1_by_1(value)))
            }

            /// @ast-id 8
            /// @src 0:57:127  "function foo() virtual public returns (uint) {..."
            function fun_foo_8() -> var__3 {
                /// @src 0:96:100  "uint"
                let zero_t_uint256_1 := zero_value_for_split_t_uint256()
                var__3 := zero_t_uint256_1

                /// @src 0:119:120  "1"
                let expr_5 := 0x01
                /// @src 0:112:120  "return 1"
                var__3 := convert_t_rational_1_by_1_to_t_uint256(expr_5)
                leave

            }
            /// @src 0:40:129  "contract A {..."

        }

        data ".metadata" hex"a26469706673582212201be97bdc94ca60404eadb6ab3f6a23b638ad79c2b91c9e1dbf1d009cb568dc0264736f6c63430008130033"
    }

}

