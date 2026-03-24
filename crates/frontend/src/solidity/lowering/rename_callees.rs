//! Module to all definitions, such as contract definitions, function
//! definitions, modifier definitions, event definitions, error definitions,
//! enum definitions, struct definitions, etc.
//!
//! The renaming is performed in 2 phases:
//! - Firstly, rename all the function definitions
//! - Secondly, rename function names in all callees.

use crate::solidity::ast::{Name, NamingEnv};
use crate::solidity::{ast::utils::*, ast::*};
use common::{error::Result, fail};

//-------------------------------------------------
// Checking function call type compatibility
//-------------------------------------------------

/// Data structure for comparing compatibility of function call types.
struct TypeChecker {}

impl TypeChecker {
    /// Constructor
    pub fn new() -> Self {
        TypeChecker {}
    }

    /// Check compatibility of function call types.
    pub fn check_call_type_compatibility(
        &mut self,
        callee_type: &Type,
        func_type: &Type,
    ) -> Result<()> {
        self.compare_type(callee_type, func_type)
    }
}

impl Compare<'_> for TypeChecker {
    /// Override `compare_func_type` to compare only parameter and return types.
    fn compare_func_type(&mut self, t1: &FuncType, t2: &FuncType) -> Result<()> {
        if t1.params.len() != t2.params.len() || t1.returns.len() != t2.returns.len() {
            fail!("Different function types: {} vs. {}", t1, t2);
        }
        for (p1, p2) in t1.params.iter().zip(t2.params.iter()) {
            if let Err(err) = self.compare_type(p1, p2) {
                fail!("Different function param: {} vs. {}\nError: {}", p1, p2, err);
            }
        }
        for (r1, r2) in t1.returns.iter().zip(t2.returns.iter()) {
            if let Err(err) = self.compare_type(r1, r2) {
                fail!("Different function return: {} vs. {}\nError: {}", r1, r2, err);
            }
        }
        Ok(())
    }

    /// Override `compare_name` to compare only base name.
    fn compare_name(&mut self, name1: &Name, name2: &Name) -> Result<()> {
        if name1.base != name2.base {
            fail!("Different names: {} vs. {}", name1, name2);
        }
        Ok(())
    }

    /// Override `compare_data_loc` to ignore data location differences.
    /// This is needed because the callee type from the AST may have a
    /// different data location than the function definition's parameter type
    /// (e.g., `calldata` vs `memory` or `default`).
    fn compare_data_loc(&mut self, _dloc1: &DataLoc, _dloc2: &DataLoc) -> Result<()> {
        Ok(())
    }

    /// Override `compare_name_opt` to accept scope mismatches.
    /// This handles cases like `D.s` (scope = Some("D")) vs `s` (scope = None)
    /// which arise in `using for` bound methods.
    fn compare_name_opt(
        &mut self,
        name1: &Option<Name>,
        name2: &Option<Name>,
    ) -> Result<()> {
        match (name1, name2) {
            (Some(n1), Some(n2)) => self.compare_name(n1, n2),
            _ => Ok(()), // Accept: Some vs None or None vs None
        }
    }

    /// Override `compare_struct_type` to ignore pointer (`is_ptr`)
    /// differences. State variable references (storage ref) have `is_ptr =
    /// true` while function parameter types may have `is_ptr = false`.
    fn compare_struct_type(
        &mut self,
        t1: &StructType,
        t2: &StructType,
    ) -> Result<()> {
        self.compare_data_loc(&t1.data_loc, &t2.data_loc)?;
        self.compare_name(&t1.name, &t2.name)?;
        self.compare_name_opt(&t1.scope, &t2.scope)?;
        // Intentionally skip is_ptr comparison
        Ok(())
    }

    /// Override `compare_array_type` to ignore pointer (`is_ptr`)
    /// differences. Storage array references have `is_ptr = true` while
    /// function parameter types may have `is_ptr = false`.
    fn compare_array_type(
        &mut self,
        t1: &ArrayType,
        t2: &ArrayType,
    ) -> Result<()> {
        self.compare_data_loc(&t1.data_loc, &t2.data_loc)?;
        self.compare_type(&t1.base, &t2.base)?;
        if t1.length != t2.length {
            fail!("Different array types: {} vs. {}", t1, t2);
        }
        // Intentionally skip is_ptr comparison
        Ok(())
    }
}

/// Check compatibility of function call types.
pub fn check_call_type(callee_type: &Type, def_type: &Type) -> bool {
    let mut checker = TypeChecker::new();
    let res = checker.check_call_type_compatibility(callee_type, def_type);
    res.is_ok()
}

//-------------------------------------------------
// Rename callee expressions
//-------------------------------------------------

/// Data structure to rename overloaded functions.
#[derive(Debug, Clone)]
struct Renamer<'a> {
    /// Name of the current source unit whose elements are being renamed.
    pub current_source_unit: Option<&'a SourceUnit>,

    /// Name of the current contract whose elements are being renamed.
    pub current_contract: Option<ContractDef>,

    /// Base contracts of the current contract.
    pub current_base_contracts: Vec<ContractDef>,

    /// List of all source units to be renamed.
    source_units: &'a [SourceUnit],

    /// Naming environment.
    env: NamingEnv,
}

impl<'a> Renamer<'a> {
    /// Constructor.
    pub fn new(source_units: &'a [SourceUnit], env: Option<&NamingEnv>) -> Self {
        let env = match env {
            Some(env) => env.clone(),
            None => NamingEnv::new(),
        };
        Renamer {
            current_source_unit: None,
            current_contract: None,
            current_base_contracts: vec![],
            source_units,
            env,
        }
    }

    // /// Rename a name scope, which is a `NamePath` containing source unit alias
    // and contract or /// library names, such as `A.B.C`.
    // fn rename_scope(&mut self, scope: &NamePath) -> NamePath {
    //     let mut new_names = vec![];
    //     for n in scope.names.iter() {
    //         new_names.push(self.rename_name(n));
    //     }
    // }

    /// Rename callees.
    pub fn rename_callees(&mut self, source_units: &[SourceUnit]) -> Vec<SourceUnit> {
        // Rename and return result.
        self.map_source_units(source_units)
    }

    /// Find contracts (libraries) bound via `using L for T` directives that
    /// contain a function matching the given member name.
    fn find_using_for_contracts(&self, member: &Name) -> Vec<ContractDef> {
        let mut contracts = vec![];

        // Helper closure: search using directives in a list of contract elements.
        let search_using_dirs = |elems: &[ContractElem],
                                 source_unit: Option<&SourceUnit>,
                                 out: &mut Vec<ContractDef>| {
            for elem in elems {
                if let ContractElem::Using(using_dir) = elem {
                    match &using_dir.kind {
                        UsingKind::UsingLib(ulib) => {
                            if let Some(sunit) = source_unit {
                                if let Some(lib) =
                                    sunit.find_contract_def_by_base_name(&ulib.lib_name)
                                {
                                    // Check if the library has a function with the member name.
                                    let has_func = lib.body.iter().any(|e| match e {
                                        ContractElem::Func(f) => f.name.base == member.base,
                                        _ => false,
                                    });
                                    if has_func {
                                        out.push(lib.clone());
                                    }
                                }
                            }
                        }
                        UsingKind::UsingFunc(_) => {
                            // UsingFunc attaches individual free functions; they
                            // are handled through normal identifier renaming.
                        }
                    }
                }
            }
        };

        // Search in the current contract's body.
        if let Some(contract) = &self.current_contract {
            search_using_dirs(
                &contract.body,
                self.current_source_unit,
                &mut contracts,
            );
        }

        // Search in source-unit-level `using` directives.
        if let Some(sunit) = self.current_source_unit {
            for elem in &sunit.elems {
                if let SourceUnitElem::Using(using_dir) = elem {
                    match &using_dir.kind {
                        UsingKind::UsingLib(ulib) => {
                            if let Some(lib) = sunit.find_contract_def_by_base_name(&ulib.lib_name)
                            {
                                let has_func = lib.body.iter().any(|e| match e {
                                    ContractElem::Func(f) => f.name.base == member.base,
                                    _ => false,
                                });
                                if has_func {
                                    contracts.push(lib.clone());
                                }
                            }
                        }
                        UsingKind::UsingFunc(_) => {}
                    }
                }
            }
        }

        contracts
    }
}

/// Implement `Map` utility to rename callees.
impl<'a> Map<'_> for Renamer<'a> {
    /// Override `map_source_unit` to capture the current renaming scope.
    fn map_source_unit(&mut self, source_unit: &SourceUnit) -> SourceUnit {
        self.current_source_unit = self
            .source_units
            .iter()
            .find(|unit| unit.path == source_unit.path);

        // Reset contract scope
        self.current_contract = None;
        let nsource_unit = map::default::map_source_unit(self, source_unit);

        // Clear the source unit scope and return result.
        self.current_source_unit = None;
        nsource_unit
    }

    // /// Override `map_using_lib` to rename the used libraries with new naming
    // indices. fn map_using_lib(&mut self, ulib: &UsingLib) -> UsingLib {
    //     let nulib = map::default::map_using_lib(self, ulib);
    //     if let Some(source_unit) = self.current_source_unit {
    //         if let Some(lib) =
    // source_unit.find_contract_def_by_base_name_path(&ulib.name_path) {
    //             return UsingLib { name_path: lib.name.clone(), ..nulib };
    //         }
    //     }
    //     nulib
    // }

    // /// Override `map_using_func`
    // fn map_using_func(&mut self, using: &'a UsingFunc) -> UsingFunc {

    // }

    /// Override `map_contract_def` to rename the base contracts with new naming
    /// indices.
    fn map_contract_def(&mut self, contract: &ContractDef) -> ContractDef {
        // Update contract scope
        if let Some(sunit) = self.current_source_unit {
            self.current_contract = sunit.find_contract_def(&contract.name).cloned();
            if let Some(contract) = self.current_contract.clone() {
                self.current_base_contracts = contract
                    .base_contracts
                    .iter()
                    .flat_map(|base| sunit.find_contract_def_by_base_name(&base.name).cloned())
                    .collect();
            };
        };

        // Continue renaming
        let ncontract = map::default::map_contract_def(self, contract);

        // Clear the contract scope and return result.
        self.current_contract = None;
        self.current_base_contracts = vec![];
        ncontract
    }

    /// Override `map_base_contract` to update indexing of the base name.
    fn map_base_contract(&mut self, base: &BaseContract) -> BaseContract {
        let nbase = map::default::map_base_contract(self, base);

        if let Some(source_unit) = self.current_source_unit {
            if let Some(contract) = source_unit.find_contract_def_by_base_name(&base.name) {
                return BaseContract { name: contract.name.clone(), ..nbase };
            }
        }

        nbase
    }

    /// Override `map_func_def`
    fn map_func_def(&mut self, func: &FuncDef) -> FuncDef {
        let nfunc = map::default::map_func_def(self, func);

        // Rename overrides
        let overriding = match (&self.current_source_unit, &func.overriding) {
            (Some(source_unit), Overriding::Some(contract_names)) => {
                let mut new_cnames = vec![];
                for cname in contract_names.iter() {
                    match source_unit.find_contract_def_by_base_name(cname) {
                        Some(contract) => new_cnames.push(contract.name.clone()),
                        None => new_cnames.push(cname.clone()),
                    }
                }
                Overriding::Some(new_cnames)
            }
            _ => func.overriding.clone(),
        };

        FuncDef { overriding, ..nfunc }
    }

    /// Override `map_var_decl`.
    fn map_var_decl(&mut self, vdecl: &VarDecl) -> VarDecl {
        let nvdecl = map::default::map_var_decl(self, vdecl);

        // Rename overrides
        let noverrides = match (&self.current_source_unit, &vdecl.overriding) {
            (Some(source_unit), Overriding::Some(names)) => {
                let mut nnames = vec![];
                for name in names.iter() {
                    match source_unit.find_contract_def_by_base_name(name) {
                        Some(contract) => nnames.push(contract.name.clone()),
                        None => nnames.push(name.clone()),
                    }
                }
                Overriding::Some(nnames)
            }
            _ => vdecl.overriding.clone(),
        };

        VarDecl { overriding: noverrides, ..nvdecl }
    }

    /// Override `map_member_expr`.
    fn map_member_expr(&mut self, expr: &MemberExpr) -> MemberExpr {
        let nexpr = map::default::map_member_expr(self, expr);

        // Skip if the member access expression which is not a function.
        if !nexpr.typ.is_func_type() || nexpr.member.index.is_some() {
            // For `using for` bound methods, the member type might not be
            // a function type in the AST (e.g. for struct members accessed
            // via bound library functions). Try `using for` lookup anyway.
            if nexpr.member.index.is_none() {
                let using_contracts = self.find_using_for_contracts(&nexpr.member);
                if !using_contracts.is_empty() {
                    // With non-func type, we can't do type-aware matching.
                    // If only one function matches by base name, use it.
                    let candidates: Vec<Name> = using_contracts
                        .iter()
                        .flat_map(|c| c.body.iter())
                        .filter_map(|e| match e {
                            ContractElem::Func(f) if f.name.base == nexpr.member.base => {
                                Some(f.name.clone())
                            }
                            _ => None,
                        })
                        .collect();
                    if candidates.len() == 1 {
                        return MemberExpr {
                            member: candidates.into_iter().next().unwrap(),
                            ..nexpr
                        };
                    }
                }
            }
            return nexpr;
        }

        // Construct the list of contracts to search for the identifier definition.
        let mut contracts = vec![];
        let base_typ = nexpr.base.typ().clone();

        // Case 1: base is `this` — look up member in current contract.
        let is_this = nexpr.base.to_string() == keywords::THIS;
        if is_this {
            if let Some(contract) = self.current_contract.clone() {
                contracts = find_contract_scopes(&contract.name, self.current_source_unit);
            }
        }

        // Case 2: base type is a contract or library type (including meta/magic types).
        if contracts.is_empty()
            && (base_typ.is_contract_type()
                || base_typ.is_magic_contract_type()
                || base_typ.is_contract_library_type()
                || base_typ.is_magic_contract_library_type())
        {
            if let Some(contract_name) = base_typ.name() {
                contracts = find_contract_scopes(&contract_name, self.current_source_unit);
            }
        }

        // Case 3: `using L for T` bound method calls — when base type is not a
        // contract type, look for `using` directives that bind a library to the
        // base type.
        if contracts.is_empty() {
            contracts = self.find_using_for_contracts(&nexpr.member);
        }

        let (nmember, _) = find_call_definition_name(&nexpr.member, &nexpr.typ, &contracts, None);
        MemberExpr { member: nmember, ..nexpr }
    }

    fn map_type_name_expr(&mut self, expr: &TypeNameExpr) -> TypeNameExpr {
        map::default::map_type_name_expr(self, expr)
    }

    /// Override `map_ident` to rename calls to overloaded functions.
    fn map_ident(&mut self, ident: &Identifier) -> Identifier {
        // Construct the list of contracts to search for the identifier definition.
        let mut contracts = vec![];
        if let Some(contract) = self.current_contract.clone() {
            contracts.push(contract);
            contracts.extend(self.current_base_contracts.clone());
        };
        let (nname, _) = find_call_definition_name(
            &ident.name,
            &ident.typ,
            &contracts,
            self.current_source_unit,
        );
        Identifier { name: nname, ..ident.clone() }
    }

    fn map_struct_type(&mut self, typ: &StructType) -> StructType {
        let ntyp = map::default::map_struct_type(self, typ);
        let contracts = match &typ.scope {
            Some(contract_name) => find_contract_scopes(contract_name, self.current_source_unit),
            _ => vec![],
        };
        let (nname, nscope) = find_call_definition_name(
            &typ.name,
            &Type::from(typ.clone()),
            &contracts,
            self.current_source_unit,
        );
        StructType { name: nname, scope: nscope, ..ntyp }
    }

    fn map_enum_type(&mut self, typ: &EnumType) -> EnumType {
        let contracts = match &typ.scope {
            Some(contract_name) => find_contract_scopes(contract_name, self.current_source_unit),
            _ => vec![],
        };
        let (nname, nscope) = find_call_definition_name(
            &typ.name,
            &Type::from(typ.clone()),
            &contracts,
            self.current_source_unit,
        );
        EnumType { name: nname, scope: nscope }
    }

    /// Override `map_contract_type`
    fn map_contract_type(&mut self, typ: &ContractType) -> ContractType {
        let ntyp = map::default::map_contract_type(self, typ);
        if let Some(contract) = find_contract_by_base_name(&typ.name, self.current_source_unit) {
            return ContractType { name: contract.name.clone(), ..ntyp };
        }
        ntyp
    }

    fn map_type_name(&mut self, typ: &UserDefinedType) -> UserDefinedType {
        let contracts = match &typ.scope {
            Some(contract_name) => find_contract_scopes(contract_name, self.current_source_unit),
            _ => vec![],
        };
        let (nname, nscope) = find_call_definition_name(
            &typ.name,
            &Type::from(typ.clone()),
            &contracts,
            self.current_source_unit,
        );
        UserDefinedType { name: nname, scope: nscope }
    }
}

/// Find contract definition by base name.
fn find_contract_by_base_name<'a>(
    name: &Name,
    source_unit: Option<&'a SourceUnit>,
) -> Option<&'a ContractDef> {
    if let Some(source_unit) = source_unit {
        source_unit.find_contract_def_by_base_name(name)
    } else {
        None
    }
}

/// Find all contract definitions in the inheritance hierarchy of a contract,
/// including the contract itself, its direct base contracts, and transitively
/// inherited contracts.
fn find_contract_scopes(name: &Name, source_unit: Option<&SourceUnit>) -> Vec<ContractDef> {
    let mut contracts: Vec<ContractDef> = vec![];
    if let Some(source_unit) = source_unit {
        if let Some(contract) = source_unit.find_contract_def_by_base_name(name) {
            let mut visited: Vec<String> = vec![];
            let mut queue: Vec<&ContractDef> = vec![contract];
            while let Some(current) = queue.pop() {
                if visited.contains(&current.name.base) {
                    continue;
                }
                visited.push(current.name.base.clone());
                contracts.push(current.clone());
                for base in current.base_contracts.iter() {
                    if let Some(c) = source_unit.find_contract_def_by_base_name(&base.name) {
                        if !visited.contains(&c.name.base) {
                            queue.push(c);
                        }
                    }
                }
            }
        }
    }
    contracts
}

/// Find the definition of a function call.
///
/// Return the name of the function definition and the contract scope where it
/// is defined.
fn find_call_definition_name(
    callee_name: &Name,
    callee_typ: &Type,
    contracts: &[ContractDef],
    source_unit: Option<&SourceUnit>,
) -> (Name, Option<Name>) {
    // First look for function definitions in the list of given contracts.
    // Collect all candidates matching by base name for the fallback.
    let mut candidates: Vec<(Name, Option<Name>)> = vec![];
    for contract in contracts.iter() {
        let scope = Some(contract.name.clone());
        for elem in contract.body.iter() {
            match elem {
                ContractElem::Func(f) if f.name.base == callee_name.base => {
                    if f.kind == FuncKind::Modifier || check_call_type(callee_typ, &f.typ()) {
                        return (f.name.clone(), scope);
                    }
                    candidates.push((f.name.clone(), scope.clone()));
                }
                ContractElem::Var(v) if v.name.base == callee_name.base => {
                    // Call to a getter function of a contract variable.
                    return (v.name.clone(), scope);
                }
                ContractElem::Error(e) if e.name.base == callee_name.base => {
                    if check_call_type(callee_typ, &e.get_type()) {
                        return (e.name.clone(), scope);
                    }
                    candidates.push((e.name.clone(), scope.clone()));
                }
                ContractElem::Event(e) if e.name.base == callee_name.base => {
                    return (e.name.clone(), scope);
                }
                ContractElem::Struct(s) if s.name.base == callee_name.base => {
                    return (s.name.clone(), scope);
                }
                ContractElem::Enum(e) if e.name.base == callee_name.base => {
                    return (e.name.clone(), scope);
                }
                ContractElem::Type(t) if t.name.base == callee_name.base => {
                    return (t.name.clone(), scope);
                }
                _ => {}
            }
        }
    }

    // If not found, look for free functions in the given source unit.
    if let Some(sunit) = source_unit {
        for elem in sunit.elems.iter() {
            match elem {
                SourceUnitElem::Contract(c) if c.name.base == callee_name.base => {
                    return (c.name.clone(), None);
                }
                SourceUnitElem::Func(f) if f.name.base == callee_name.base => {
                    if f.kind == FuncKind::Modifier || check_call_type(callee_typ, &f.typ()) {
                        return (f.name.clone(), None);
                    }
                    candidates.push((f.name.clone(), None));
                }
                SourceUnitElem::Error(e) if e.name.base == callee_name.base => {
                    if check_call_type(callee_typ, &e.get_type()) {
                        return (e.name.clone(), None);
                    }
                    candidates.push((e.name.clone(), None));
                }
                SourceUnitElem::Struct(s) if s.name.base == callee_name.base => {
                    return (s.name.clone(), None);
                }
                SourceUnitElem::Enum(e) if e.name.base == callee_name.base => {
                    return (e.name.clone(), None);
                }
                SourceUnitElem::UserType(t) if t.name.base == callee_name.base => {
                    return (t.name.clone(), None);
                }
                _ => {}
            }
        }
    }

    // Fallback: if exactly one candidate matched by base name but failed the
    // strict type check, use it. This handles cases where type comparison
    // fails due to data location qualifiers or incomplete type info.
    if candidates.len() == 1 {
        return candidates.into_iter().next().unwrap();
    }

    (callee_name.clone(), None)
}

// /// TODO: rename function name
// fn rename_function(ident: Ident, func: &FuncDef) -> Option<Ident> {
//     if func.name.base_name == ident.name.base_name {
//         let func_type = normalize_data_location(&func.data_type());
//         if check_function_call_type_compatibility(&ident_type, &func_type) {
//             return Ident {
//                 name: func.name.clone(),
//                 ..ident.clone()
//             };
//         }
//     }
// }

//-------------------------------------------------
// Public functions
//-------------------------------------------------

/// Rename definitions of contracts, functions, enums, structs, events, errors.
pub fn rename_callees(
    source_units: &[SourceUnit],
    env: Option<&NamingEnv>,
) -> (Vec<SourceUnit>, NamingEnv) {
    let mut renamer = Renamer::new(source_units, env);
    let nsource_units = renamer.rename_callees(source_units);
    (nsource_units, renamer.env)
}

//-------------------------------------------------
// Unit tests
//-------------------------------------------------

/// Unit tests
#[cfg(test)]
mod tests {
    use super::rename_callees;
    use crate::solidity::{
        ast::utils::syntactic_comparer::compare_source_units,
        lowering::{rename_defs, utils::configure_unit_test_env},
        parsing::parse_solidity_source_code,
    };
    use indoc::indoc;

    /// Test renaming callees.
    #[test]
    fn test_rename_callees() {
        let _ = configure_unit_test_env();

        // Input contract
        let input_contract = indoc! {r###"
            contract C {
                function g() public returns (uint, uint) {
                    return (f("abc"), f(2));
                }

                function g(uint x) public returns (uint) {
                    return x;
                }

                function z(uint x) public returns (uint) {
                    uint a = g(x);
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
                    return (f_1("abc"), f_0(2));
                }

                function g_1(uint x) public returns (uint) {
                    return x;
                }

                function z(uint x) public returns (uint) {
                    uint a = g_1(x);
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

        let input_sunits = match parse_solidity_source_code(input_contract, "0.8.15") {
            Ok(input_sunits) => input_sunits,
            Err(err) => panic!("Failed to parse input source unit: {}", err),
        };

        let (output_sunits, env) = rename_defs(&input_sunits, None);
        let (output_sunits, _) = rename_callees(&output_sunits, Some(&env));

        let expected_sunits = match parse_solidity_source_code(expected_contract, "0.8.15") {
            Ok(sunits) => sunits,
            Err(err) => panic!("Failed to parse expected source unit: {}", err),
        };

        if let Err(err) = compare_source_units(&output_sunits, &expected_sunits) {
            panic!("Failed to rename callees: {}", err)
        }
    }
}
