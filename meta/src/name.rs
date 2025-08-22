use std::{
    collections::HashMap,
    fmt::{self, Display},
};

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct Name {
    pub base: String,
    pub index: Option<usize>,
}

impl Name {
    pub fn new(base: String, index: Option<usize>) -> Self {
        Self { base, index }
    }

    pub fn is_empty(&self) -> bool {
        self.base.is_empty()
    }

    pub fn set_index(&mut self, index: Option<usize>) {
        self.index = index
    }

    pub fn original_name(&self) -> &str {
        self.base.as_str()
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.index {
            Some(index) => write!(f, "{}_{}", self.base, index),
            None => write!(f, "{}", self.base),
        }
    }
}

impl From<String> for Name {
    fn from(value: String) -> Self {
        Name::from(value.as_str())
    }
}

impl From<&String> for Name {
    fn from(value: &String) -> Self {
        Name::from(value.as_str())
    }
}

impl From<&str> for Name {
    fn from(value: &str) -> Self {
        Name::new(value.to_string(), None)
    }
}

/// Data structure represent a name path, such as `A.B.C`, where `A`, `B`, and
/// `C` are names or aliases of source units, contracts, or libraries, etc.
///
/// See more about `identifier-path` in Solidity's language specification:
/// <https://docs.soliditylang.org/en/latest/grammar.html#a4.SolidityParser.identifierPath>
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct NamePath {
    pub names: Vec<Name>,
}

impl NamePath {
    pub fn new(names: Vec<Name>) -> Self {
        Self { names }
    }
}

impl From<&str> for NamePath {
    fn from(name_path: &str) -> Self {
        let names = name_path.split('.').map(Name::from).collect();
        Self { names }
    }
}

impl From<&[Name]> for NamePath {
    fn from(names: &[Name]) -> Self {
        Self { names: names.to_vec() }
    }
}

impl Display for NamePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self
            .names
            .iter()
            .map(|name| name.to_string())
            .collect::<Vec<String>>()
            .join(".");
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone)]
pub struct NamingEnv {
    pub current_naming_index_map: HashMap<String, usize>,
    pub naming_index_counter_map: HashMap<String, usize>,
}

impl NamingEnv {
    pub fn new() -> Self {
        NamingEnv {
            current_naming_index_map: HashMap::new(),
            naming_index_counter_map: HashMap::new(),
        }
    }

    pub fn get_current_index(&self, base_name: &str) -> Option<usize> {
        self.current_naming_index_map.get(base_name).cloned()
    }

    pub fn get_current_name(&self, base: &str) -> Name {
        let idx = self.get_current_index(base);
        Name::new(base.to_string(), idx)
    }

    pub fn create_new_name(&self, base: &str) -> (Name, NamingEnv) {
        // Create the new environment
        let mut nenv = self.clone();

        // New index
        let nidx = match self.naming_index_counter_map.get(base) {
            None => 0,
            Some(idx) => *idx + 1,
        };

        // Update current index
        nenv.current_naming_index_map.insert(base.to_string(), nidx);

        // Update index counter
        nenv.naming_index_counter_map.insert(base.to_string(), nidx);

        // // Fine-tune and return result
        // let final_idx = match nidx == 0 {
        //     true => None,
        //     false => Some(nidx),
        // };

        let nname = Name::new(base.to_string(), Some(nidx));
        (nname, nenv)
    }
}

impl Default for NamingEnv {
    fn default() -> Self {
        Self::new()
    }
}
