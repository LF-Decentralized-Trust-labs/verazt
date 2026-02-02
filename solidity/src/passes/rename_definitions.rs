//! Module to all definitions, such as contract definitions, function
//! definitions, modifier definitions, event definitions, error definitions,
//! enum definitions, struct definitions, etc.
//!
//! The renaming is necessary to eliminate the `import` directives later.

use meta::NamingEnv;

use crate::{ast::utils::*, ast::*};

//-------------------------------------------------
// Rename definitions
//-------------------------------------------------

#[derive(Debug, Clone)]
struct Renamer {
    env: NamingEnv,
}

impl Renamer {
    pub fn new(env: Option<&NamingEnv>) -> Self {
        let env = match env {
            Some(env) => env.clone(),
            None => NamingEnv::new(),
        };
        Renamer { env }
    }

    pub fn rename_definitions(&mut self, source_units: &[SourceUnit]) -> Vec<SourceUnit> {
        // Rename all function definitions and return results.
        self.map_source_units(source_units)
    }
}

/// Implement `Map` utility to rename overloaded functions.
impl Map<'_> for Renamer {
    /// Override `map_source_unit` to capture the current renaming scope.
    fn map_source_unit(&mut self, source_unit: &SourceUnit) -> SourceUnit {
        map::default::map_source_unit(self, source_unit)
    }

    /// Override `map_func_def` to rename function names.
    fn map_func_def(&mut self, func: &FuncDef) -> FuncDef {
        // Save the current naming index before transforming the function definition.
        let current_naming_index = self.env.current_naming_index_map.clone();

        // Rename the function name with a new indexing number.
        let (nname, nenv) = self.env.create_new_name(&func.name.base);
        self.env = nenv;

        // Transform the function definition and update its new name.
        let mut nfunc = map::default::map_func_def(self, func);
        nfunc.name = nname;

        // Restore the current naming index
        self.env.current_naming_index_map = current_naming_index;

        // Return result
        nfunc
    }

    /// Override `map_event_def` to rename Event names.
    fn map_event_def(&mut self, event: &EventDef) -> EventDef {
        // Save the current naming index before transforming the Event definition.
        let current_naming_index = self.env.current_naming_index_map.clone();

        // Rename the Event name with a new indexing number.
        let (nname, nenv) = self.env.create_new_name(&event.name.base);
        self.env = nenv;

        // Transform the Event definition and update its new name.
        let mut nevent = map::default::map_event_def(self, event);
        nevent.name = nname;

        // Restore the current naming index
        self.env.current_naming_index_map = current_naming_index;

        // Return result
        nevent
    }

    /// Override `map_error_def` to rename Error names.
    fn map_error_def(&mut self, error: &ErrorDef) -> ErrorDef {
        // Save the current naming index before transforming the Error definition.
        let current_naming_index = self.env.current_naming_index_map.clone();

        // Rename the Error name with a new indexing number.
        let (nname, nenv) = self.env.create_new_name(&error.name.base);
        self.env = nenv;

        // Transform the Error definition and update its new name.
        let mut nerror = map::default::map_error_def(self, error);
        nerror.name = nname;

        // Restore the current naming index
        self.env.current_naming_index_map = current_naming_index;

        // Return result
        nerror
    }

    /// Override `map_struct_def` to rename Struct names.
    fn map_struct_def(&mut self, struct_: &StructDef) -> StructDef {
        // Save the current naming index before transforming the Struct definition.
        let current_naming_index = self.env.current_naming_index_map.clone();

        // Rename the Struct name with a new indexing number.
        let (nname, nenv) = self.env.create_new_name(&struct_.name.base);
        self.env = nenv;

        // Transform the Struct definition and update its new name.
        let mut nstruct = map::default::map_struct_def(self, struct_);
        nstruct.name = nname;

        // Restore the current naming index
        self.env.current_naming_index_map = current_naming_index;

        // Return result
        nstruct
    }

    /// Override `map_enum_def` to rename Enum names.
    fn map_enum_def(&mut self, enum_: &EnumDef) -> EnumDef {
        // Save the current naming index before transforming the Enum definition.
        let current_naming_index = self.env.current_naming_index_map.clone();

        // Rename the Enum name with a new indexing number.
        let (nname, nenv) = self.env.create_new_name(&enum_.name.base);
        self.env = nenv;

        // Transform the Enum definition and update its new name.
        let mut nenum = map::default::map_enum_def(self, enum_);
        nenum.name = nname;

        // Restore the current naming index
        self.env.current_naming_index_map = current_naming_index;

        // Return result
        nenum
    }

    /// Override `map_user_defined_value_type_def` to rename the type's name.
    fn map_udv_type_def(&mut self, typ: &TypeDef) -> TypeDef {
        // Save the current naming index before transforming the Struct definition.
        let current_naming_index = self.env.current_naming_index_map.clone();

        // Rename the type name with a new indexing number.
        let (nname, nenv) = self.env.create_new_name(&typ.name.base);
        self.env = nenv;

        // Transform the type name definition and update its new name.
        let mut ntyp = map::default::map_udv_type_def(self, typ);
        ntyp.name = nname;

        // Restore the current naming index
        self.env.current_naming_index_map = current_naming_index;

        // Return result
        ntyp
    }

    /// Override `map_contract_type` to rename contract name.
    fn map_contract_type(&mut self, typ: &ContractType) -> ContractType {
        let nname = self.env.get_current_name(typ.name.base.as_str());
        let mut ntyp = map::default::map_contract_type(self, typ);
        ntyp.name = nname.clone();
        ntyp
    }
}

//-------------------------------------------------
// Public functions
//-------------------------------------------------

/// Rename definitions of contracts, functions, enums, structs, events, errors.
pub fn rename_definitions(
    source_units: &[SourceUnit],
    env: Option<&NamingEnv>,
) -> (Vec<SourceUnit>, NamingEnv) {
    println!("Normalize AST: renaming definitions");
    let mut renamer = Renamer::new(env);
    let nsource_units = renamer.rename_definitions(source_units);
    (nsource_units, renamer.env)
}

//-------------------------------------------------
// Unit tests
//-------------------------------------------------

/// Unit tests
#[cfg(test)]
mod tests {
    use super::rename_definitions;
    use crate::{
        ast::utils::syntactic_comparer::compare_source_units,
        compile::compile_solidity_source_code, passes::utils::configure_unit_test_env,
    };
    use indoc::indoc;

    /// Test renaming definitions.
    ///
    /// Note that this unit test doesn't rename callee functions.
    #[test]
    fn test_rename_definitions() {
        let _ = configure_unit_test_env();

        // Input contract
        let input_contract = indoc! {r###"
            contract C {
                function g() public returns (uint, uint) {
                    return (2, 3);
                }

                function g(uint x) public returns (uint) {
                    return x;
                }

                function z(uint x) public returns (uint) {
                    uint a = 2;
                    return a;
                }
            }

            function f(uint) returns (uint) {
                return 2;
            }

            function f(string memory) returns (uint) {
                return 3;
            }

            function g(bool) returns (uint) {
                return 1;
            }"###};

        // Expected output contract
        let expected_contract = indoc! {r###"
            contract C {
                function g_0() public returns (uint, uint) {
                    return (2, 3);
                }

                function g_1(uint x) public returns (uint) {
                    return x;
                }

                function z_0(uint x) public returns (uint) {
                    uint a = 2;
                    return a;
                }
            }

            function f_0(uint) returns (uint) {
                return 2;
            }

            function f_1(string memory) returns (uint) {
                return 3;
            }

            function g_2(bool) returns (uint) {
                return 1;
            }"###};

        let input_sunits = match compile_solidity_source_code(input_contract, "0.8.15") {
            Ok(sunits) => sunits,
            Err(err) => panic!("Failed to parse input source unit: {}", err),
        };
        let (output_sunits, _) = rename_definitions(&input_sunits, None);

        let expected_sunits = match compile_solidity_source_code(expected_contract, "0.8.15") {
            Ok(sunits) => sunits,
            Err(err) => panic!("Failed to parse expected source unit: {}", err),
        };

        if let Err(err) = compare_source_units(&output_sunits, &expected_sunits) {
            panic!("Failed to flatten definitions: {}", err)
        }
    }
}
