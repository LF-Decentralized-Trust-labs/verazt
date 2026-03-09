
/// @use-src 0:"contract_to_base_override.sol"
object "C_84" {
    code {
        /// @src 0:323:508  "contract C {..."
        mstore(64, memoryguard(128))
        if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }

        constructor_C_84()

        let _1 := allocate_unbounded()
        codecopy(_1, dataoffset("C_84_deployed"), datasize("C_84_deployed"))

        return(_1, datasize("C_84_deployed"))

        function allocate_unbounded() -> memPtr {
            memPtr := mload(64)
        }

        function revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() {
            revert(0, 0)
        }

        /// @src 0:323:508  "contract C {..."
        function constructor_C_84() {

            /// @src 0:323:508  "contract C {..."

        }
        /// @src 0:323:508  "contract C {..."

    }
    /// @use-src 0:"contract_to_base_override.sol"
    object "C_84_deployed" {
        code {
            /// @src 0:323:508  "contract C {..."
            mstore(64, memoryguard(128))

            if iszero(lt(calldatasize(), 4))
            {
                let selector := shift_right_224_unsigned(calldataload(0))
                switch selector

                case 0x26121ff0
                {
                    // f()

                    external_fun_f_83()
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

            function abi_encode_tuple__to__fromStack(headStart ) -> tail {
                tail := add(headStart, 0)

            }

            function external_fun_f_83() {

                if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }
                abi_decode_tuple_(4, calldatasize())
                fun_f_83()
                let memPos := allocate_unbounded()
                let memEnd := abi_encode_tuple__to__fromStack(memPos  )
                return(memPos, sub(memEnd, memPos))

            }

            function revert_error_42b3090547df1d2001c96683413b8cf91c1b902ef5e3cb8d9f6f304cf7446f74() {
                revert(0, 0)
            }

            function panic_error_0x41() {
                mstore(0, 35408467139433450592217433187231851964531694900788300625387963629091585785856)
                mstore(4, 0x41)
                revert(0, 0x24)
            }

            function revert_forward_1() {
                let pos := allocate_unbounded()
                returndatacopy(pos, 0, returndatasize())
                revert(pos, returndatasize())
            }

            function cleanup_t_uint160(value) -> cleaned {
                cleaned := and(value, 0xffffffffffffffffffffffffffffffffffffffff)
            }

            function identity(value) -> ret {
                ret := value
            }

            function convert_t_uint160_to_t_uint160(value) -> converted {
                converted := cleanup_t_uint160(identity(cleanup_t_uint160(value)))
            }

            function convert_t_uint160_to_t_contract$_A_$9(value) -> converted {
                converted := convert_t_uint160_to_t_uint160(value)
            }

            function convert_t_contract$_B_$31_to_t_contract$_A_$9(value) -> converted {
                converted := convert_t_uint160_to_t_contract$_A_$9(value)
            }

            function convert_t_uint160_to_t_address(value) -> converted {
                converted := convert_t_uint160_to_t_uint160(value)
            }

            function convert_t_contract$_A_$9_to_t_address(value) -> converted {
                converted := convert_t_uint160_to_t_address(value)
            }

            function revert_error_0cc013b6b3b6beabea4e3a74a6d380f0df81852ca99887912475e1f66b2a2c20() {
                revert(0, 0)
            }

            function round_up_to_mul_of_32(value) -> result {
                result := and(add(value, 31), not(31))
            }

            function finalize_allocation(memPtr, size) {
                let newFreePtr := add(memPtr, round_up_to_mul_of_32(size))
                // protect against overflow
                if or(gt(newFreePtr, 0xffffffffffffffff), lt(newFreePtr, memPtr)) { panic_error_0x41() }
                mstore(64, newFreePtr)
            }

            function shift_left_224(value) -> newValue {
                newValue :=

                shl(224, value)

            }

            function revert_error_c1322bf8034eace5e0b5c7295db60986aa89aae5e0ea0873e4689e076861a5db() {
                revert(0, 0)
            }

            function cleanup_t_uint256(value) -> cleaned {
                cleaned := value
            }

            function validator_revert_t_uint256(value) {
                if iszero(eq(value, cleanup_t_uint256(value))) { revert(0, 0) }
            }

            function abi_decode_t_uint256_fromMemory(offset, end) -> value {
                value := mload(offset)
                validator_revert_t_uint256(value)
            }

            function abi_decode_tuple_t_uint256_fromMemory(headStart, dataEnd) -> value0 {
                if slt(sub(dataEnd, headStart), 32) { revert_error_dbdddcbe895c83990c08b3492a0e83918d802a52331272ac6fdb6a7c4aea3b1b() }

                {

                    let offset := 0

                    value0 := abi_decode_t_uint256_fromMemory(add(headStart, offset), dataEnd)
                }

            }

            function convert_t_contract$_B_$31_to_t_address(value) -> converted {
                converted := convert_t_uint160_to_t_address(value)
            }

            /// @ast-id 83
            /// @src 0:338:506  "function f() public {..."
            function fun_f_83() {

                /// @src 0:371:378  "new A()"
                let _1 := allocate_unbounded()
                let _2 := add(_1, datasize("A_9"))
                if or(gt(_2, 0xffffffffffffffff), lt(_2, _1)) { panic_error_0x41() }
                datacopy(_1, dataoffset("A_9"), datasize("A_9"))
                _2 := abi_encode_tuple__to__fromStack(_2)

                let expr_40_address := create(0, _1, sub(_2, _1))

                if iszero(expr_40_address) { revert_forward_1() }

                /// @src 0:364:378  "A a1 = new A()"
                let var_a1_36_address := expr_40_address
                /// @src 0:391:398  "new B()"
                let _3 := allocate_unbounded()
                let _4 := add(_3, datasize("B_31"))
                if or(gt(_4, 0xffffffffffffffff), lt(_4, _3)) { panic_error_0x41() }
                datacopy(_3, dataoffset("B_31"), datasize("B_31"))
                _4 := abi_encode_tuple__to__fromStack(_4)

                let expr_48_address := create(0, _3, sub(_4, _3))

                if iszero(expr_48_address) { revert_forward_1() }

                /// @src 0:384:398  "A a2 = new B()"
                let var_a2_44_address := convert_t_contract$_B_$31_to_t_contract$_A_$9(expr_48_address)
                /// @src 0:410:417  "new B()"
                let _5 := allocate_unbounded()
                let _6 := add(_5, datasize("B_31"))
                if or(gt(_6, 0xffffffffffffffff), lt(_6, _5)) { panic_error_0x41() }
                datacopy(_5, dataoffset("B_31"), datasize("B_31"))
                _6 := abi_encode_tuple__to__fromStack(_6)

                let expr_56_address := create(0, _5, sub(_6, _5))

                if iszero(expr_56_address) { revert_forward_1() }

                /// @src 0:404:417  "B b = new B()"
                let var_b_52_address := expr_56_address
                /// @src 0:430:431  "b"
                let _7_address := var_b_52_address
                let expr_61_address := _7_address
                /// @src 0:423:431  "A a3 = b"
                let var_a3_60_address := convert_t_contract$_B_$31_to_t_contract$_A_$9(expr_61_address)
                /// @src 0:444:445  "b"
                let _8_address := var_b_52_address
                let expr_66_address := _8_address
                /// @src 0:437:445  "B b2 = b"
                let var_b2_65_address := expr_66_address
                /// @src 0:461:463  "a3"
                let _9_address := var_a3_60_address
                let expr_70_address := _9_address
                /// @src 0:461:467  "a3.foo"
                let expr_71_address := convert_t_contract$_A_$9_to_t_address(expr_70_address)
                let expr_71_functionSelector := 0xc2985578
                /// @src 0:461:469  "a3.foo()"

                // storage for arguments and returned data
                let _10 := allocate_unbounded()
                mstore(_10, shift_left_224(expr_71_functionSelector))
                let _11 := abi_encode_tuple__to__fromStack(add(_10, 4) )

                let _12 := call(gas(), expr_71_address,  0,  _10, sub(_11, _10), _10, 32)

                if iszero(_12) { revert_forward_1() }

                let expr_72
                if _12 {

                    let _13 := 32

                    if gt(_13, returndatasize()) {
                        _13 := returndatasize()
                    }

                    // update freeMemoryPointer according to dynamic return size
                    finalize_allocation(_10, _13)

                    // decode return parameters from external try-call into retVars
                    expr_72 :=  abi_decode_tuple_t_uint256_fromMemory(_10, add(_10, _13))
                }
                /// @src 0:452:469  "uint x = a3.foo()"
                let var_x_69 := expr_72
                /// @src 0:484:486  "b2"
                let _14_address := var_b2_65_address
                let expr_76_address := _14_address
                /// @src 0:484:490  "b2.bar"
                let expr_77_address := convert_t_contract$_B_$31_to_t_address(expr_76_address)
                let expr_77_functionSelector := 0xfebb0f7e
                /// @src 0:484:492  "b2.bar()"

                // storage for arguments and returned data
                let _15 := allocate_unbounded()
                mstore(_15, shift_left_224(expr_77_functionSelector))
                let _16 := abi_encode_tuple__to__fromStack(add(_15, 4) )

                let _17 := call(gas(), expr_77_address,  0,  _15, sub(_16, _15), _15, 32)

                if iszero(_17) { revert_forward_1() }

                let expr_78
                if _17 {

                    let _18 := 32

                    if gt(_18, returndatasize()) {
                        _18 := returndatasize()
                    }

                    // update freeMemoryPointer according to dynamic return size
                    finalize_allocation(_15, _18)

                    // decode return parameters from external try-call into retVars
                    expr_78 :=  abi_decode_tuple_t_uint256_fromMemory(_15, add(_15, _18))
                }
                /// @src 0:475:492  "uint y = b2.bar()"
                let var_y_75 := expr_78
                /// @src 0:499:501  "a1"
                let _19_address := var_a1_36_address
                let expr_80_address := _19_address

            }
            /// @src 0:323:508  "contract C {..."

        }

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

        /// @use-src 0:"contract_to_base_override.sol"
        object "B_31" {
            code {
                /// @src 0:131:321  "contract B is A {..."
                mstore(64, memoryguard(128))
                if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }

                constructor_B_31()

                let _1 := allocate_unbounded()
                codecopy(_1, dataoffset("B_31_deployed"), datasize("B_31_deployed"))

                return(_1, datasize("B_31_deployed"))

                function allocate_unbounded() -> memPtr {
                    memPtr := mload(64)
                }

                function revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() {
                    revert(0, 0)
                }

                /// @src 0:131:321  "contract B is A {..."
                function constructor_B_31() {

                    /// @src 0:131:321  "contract B is A {..."
                    constructor_A_9()

                }
                /// @src 0:131:321  "contract B is A {..."

                /// @src 0:40:129  "contract A {..."
                function constructor_A_9() {

                    /// @src 0:40:129  "contract A {..."

                }
                /// @src 0:131:321  "contract B is A {..."

            }
            /// @use-src 0:"contract_to_base_override.sol"
            object "B_31_deployed" {
                code {
                    /// @src 0:131:321  "contract B is A {..."
                    mstore(64, memoryguard(128))

                    if iszero(lt(calldatasize(), 4))
                    {
                        let selector := shift_right_224_unsigned(calldataload(0))
                        switch selector

                        case 0xc2985578
                        {
                            // foo()

                            external_fun_foo_22()
                        }

                        case 0xfebb0f7e
                        {
                            // bar()

                            external_fun_bar_30()
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

                    function external_fun_foo_22() {

                        if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }
                        abi_decode_tuple_(4, calldatasize())
                        let ret_0 :=  fun_foo_22()
                        let memPos := allocate_unbounded()
                        let memEnd := abi_encode_tuple_t_uint256__to_t_uint256__fromStack(memPos , ret_0)
                        return(memPos, sub(memEnd, memPos))

                    }

                    function external_fun_bar_30() {

                        if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }
                        abi_decode_tuple_(4, calldatasize())
                        let ret_0 :=  fun_bar_30()
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

                    /// @ast-id 22
                    /// @src 0:153:251  "function foo() override public returns (uint) {..."
                    function fun_foo_22() -> var__15 {
                        /// @src 0:193:197  "uint"
                        let zero_t_uint256_1 := zero_value_for_split_t_uint256()
                        var__15 := zero_t_uint256_1

                        /// @src 0:237:244  "A.foo()"
                        let expr_19 := fun_foo_8()
                        /// @src 0:230:244  "return A.foo()"
                        var__15 := expr_19
                        leave

                    }
                    /// @src 0:131:321  "contract B is A {..."

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
                        let zero_t_uint256_2 := zero_value_for_split_t_uint256()
                        var__3 := zero_t_uint256_2

                        /// @src 0:119:120  "1"
                        let expr_5 := 0x01
                        /// @src 0:112:120  "return 1"
                        var__3 := convert_t_rational_1_by_1_to_t_uint256(expr_5)
                        leave

                    }
                    /// @src 0:131:321  "contract B is A {..."

                    function cleanup_t_rational_2_by_1(value) -> cleaned {
                        cleaned := value
                    }

                    function convert_t_rational_2_by_1_to_t_uint256(value) -> converted {
                        converted := cleanup_t_uint256(identity(cleanup_t_rational_2_by_1(value)))
                    }

                    /// @ast-id 30
                    /// @src 0:257:319  "function bar() public returns (uint) {..."
                    function fun_bar_30() -> var__25 {
                        /// @src 0:288:292  "uint"
                        let zero_t_uint256_3 := zero_value_for_split_t_uint256()
                        var__25 := zero_t_uint256_3

                        /// @src 0:311:312  "2"
                        let expr_27 := 0x02
                        /// @src 0:304:312  "return 2"
                        var__25 := convert_t_rational_2_by_1_to_t_uint256(expr_27)
                        leave

                    }
                    /// @src 0:131:321  "contract B is A {..."

                }

                data ".metadata" hex"a2646970667358221220c1d9ae2ea50e1417087d2057c29738a904959f40bb790507624ff0d3d62ce21f64736f6c63430008130033"
            }

        }

        data ".metadata" hex"a2646970667358221220c3c63b24626481036a582f9d334ebc505ed129df897a568849ae731044efa02464736f6c63430008130033"
    }

}

