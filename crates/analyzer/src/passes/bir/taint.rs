//! Extended Taint Analysis Pass
//!
//! Propagates the taint sources of each module to a fixpoint along SSA
//! operands and block arguments, through internal calls (arguments to
//! parameters, returned values to call results), and through contract
//! state (stored values to later loads of the same resource). Calls and
//! state are context-insensitive: a parameter carries the labels of every
//! call site's argument, and a resource those of every value stored into
//! any of its slots. The result is a typed `TaintArtifact` (set of taint
//! labels per `OpId`, keyed by module and function).
//!
//! Extended sources: TxOrigin, Timestamp, MsgValue, ExternalCallReturn.
//! Extended sinks:  branch conditions, storage writes, arithmetic operands.

use crate::context::{AnalysisContext, ContextKey};
use crate::passes::base::meta::{PassLevel, PassRepresentation};
use crate::passes::base::{AnalysisPass, Pass, PassResult};
use crate::passes::bir::icfg::ICFGPass;
use scirs::bir::cfg::{BlockId, Function, FunctionId};
use scirs::bir::interfaces::TaintLabel;
use scirs::bir::module::Module;
use scirs::bir::ops::{CallTarget, DialectOp, EvmOp, Op, OpId, OpKind, OpRef, Resource};
use std::any::TypeId;
use std::collections::{HashMap, HashSet};

// ═══════════════════════════════════════════════════════════════════
// Artifact
// ═══════════════════════════════════════════════════════════════════

/// Artifact key for extended taint analysis.
///
/// Maps BIR module id → function → `OpId` → set of `TaintLabel` that reach
/// this op. `OpId`s are only unique within a function.
pub struct TaintArtifact;

impl ContextKey for TaintArtifact {
    type Value = HashMap<String, HashMap<FunctionId, HashMap<OpId, HashSet<TaintLabel>>>>;
    const NAME: &'static str = "taint";
}

// ═══════════════════════════════════════════════════════════════════
// Data Structures
// ═══════════════════════════════════════════════════════════════════

/// The taint state of one module during the fixpoint.
#[derive(Default)]
struct ModuleTaint {
    /// Labels of the arguments passed to each parameter index of each
    /// function, over all internal call sites.
    params: HashMap<FunctionId, HashMap<usize, HashSet<TaintLabel>>>,
    /// Labels each function may return.
    returns: HashMap<FunctionId, HashSet<TaintLabel>>,
    /// Labels stored into each state resource, over all its slots.
    storage: HashMap<Resource, HashSet<TaintLabel>>,
    /// Labels reaching each SSA value of each function.
    values: HashMap<FunctionId, HashMap<OpId, HashSet<TaintLabel>>>,
}

// ═══════════════════════════════════════════════════════════════════
// Pass
// ═══════════════════════════════════════════════════════════════════

/// Extended taint analysis pass.
#[derive(Debug, Default)]
pub struct TaintPass;

impl Pass for TaintPass {
    fn name(&self) -> &'static str {
        "taint"
    }

    fn description(&self) -> &'static str {
        "Extended taint analysis with multiple label types"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Bir
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![TypeId::of::<ICFGPass>()]
    }
}

impl AnalysisPass for TaintPass {
    type Artifact = TaintArtifact;

    fn run(
        &self,
        ctx: &AnalysisContext,
    ) -> PassResult<HashMap<String, HashMap<FunctionId, HashMap<OpId, HashSet<TaintLabel>>>>> {
        let result = ctx
            .bir_units()
            .iter()
            .map(|module| (module.source_module_id.clone(), ModuleTaint::solve(module).values))
            .collect();

        Ok(result)
    }
}

// ═══════════════════════════════════════════════════════════════════
// ModuleTaint Implementations
// ═══════════════════════════════════════════════════════════════════

impl ModuleTaint {
    /// Seed the taint sources of `module` and propagate them to a fixpoint.
    /// Labels only accumulate and are finitely many, so it terminates.
    fn solve(module: &Module) -> Self {
        let mut taint = ModuleTaint::default();
        for func in &module.functions {
            let values = taint.values.entry(func.id.clone()).or_default();
            for op in func.blocks.iter().flat_map(|block| &block.ops) {
                if let Some(label) = op.kind.taint_source() {
                    values.entry(op.id).or_default().insert(label);
                }
            }
        }
        let indices: Vec<HashMap<BlockId, usize>> = module
            .functions
            .iter()
            .map(|func| {
                func.blocks
                    .iter()
                    .enumerate()
                    .map(|(i, b)| (b.id, i))
                    .collect()
            })
            .collect();

        let mut changed = true;
        while changed {
            changed = false;
            for (func, index) in module.functions.iter().zip(&indices) {
                changed |= taint.propagate_in(func, index);
            }
        }
        taint
    }

    /// One sweep over the ops and block arguments of `func`, whose block
    /// indices are `index`; returns `true` if any label was new.
    fn propagate_in(&mut self, func: &Function, index: &HashMap<BlockId, usize>) -> bool {
        let mut changed = false;
        for block in &func.blocks {
            for op in &block.ops {
                changed |= self.transfer_op(&func.id, op);
            }
            // Block parameters receive the labels of their arguments
            for call in block.term.block_calls() {
                let Some(&target) = index.get(&call.block) else {
                    continue;
                };
                for (param, arg) in func.blocks[target].params.iter().zip(&call.args) {
                    let labels = self.labels_of(&func.id, &[*arg]);
                    changed |= self.taint_value(&func.id, param.id, labels);
                }
            }
        }
        changed
    }

    /// Propagate the labels flowing through `op` of function `func`;
    /// returns `true` if any label was new.
    fn transfer_op(&mut self, func: &FunctionId, op: &Op) -> bool {
        match &op.kind {
            // Values computed from their operands carry their labels
            OpKind::BinOp { .. }
            | OpKind::UnOp { .. }
            | OpKind::Opaque { .. }
            | OpKind::Dialect(DialectOp::Evm(EvmOp::Builtin(_))) => {
                let labels = self.labels_of(func, &op.kind.operands());
                self.taint_value(func, op.id, labels)
            }
            OpKind::Param { index } => {
                let params = self.params.get(func).and_then(|params| params.get(index));
                let labels = params.cloned().unwrap_or_default();
                self.taint_value(func, op.id, labels)
            }
            OpKind::Call(call) => match &call.target {
                CallTarget::Internal(callee) => {
                    let mut changed = false;
                    for (i, arg) in call.args.iter().enumerate() {
                        let labels = self.labels_of(func, &[*arg]);
                        let param = self.params.entry(callee.clone()).or_default().entry(i);
                        changed |= extend_labels(param.or_default(), labels);
                    }
                    let labels = self.returns.get(callee).cloned().unwrap_or_default();
                    changed | self.taint_value(func, op.id, labels)
                }
                // External results are seeded as `ExternalReturn` sources.
                CallTarget::External(_) => false,
            },
            OpKind::Load(load) => {
                let labels = self
                    .storage
                    .get(&load.resource)
                    .cloned()
                    .unwrap_or_default();
                self.taint_value(func, op.id, labels)
            }
            OpKind::Store(store) => match store.value {
                Some(value) => {
                    let labels = self.labels_of(func, &[value]);
                    extend_labels(self.storage.entry(store.resource.clone()).or_default(), labels)
                }
                None => false,
            },
            OpKind::Return(values) => {
                let labels = self.labels_of(func, values);
                extend_labels(self.returns.entry(func.clone()).or_default(), labels)
            }
            // Sources (seeded), constants, and effects without a result
            OpKind::Assert { .. }
            | OpKind::Const(_)
            | OpKind::Dialect(_)
            | OpKind::Emit(_)
            | OpKind::Env(_)
            | OpKind::ExprStmt { .. }
            | OpKind::Symbol { .. } => false,
        }
    }

    /// The union of the labels of `refs` in function `func`.
    fn labels_of(&self, func: &FunctionId, refs: &[OpRef]) -> HashSet<TaintLabel> {
        let Some(values) = self.values.get(func) else {
            return HashSet::new();
        };
        refs.iter()
            .filter_map(|OpRef(id)| values.get(id))
            .flatten()
            .copied()
            .collect()
    }

    /// Add `labels` to the SSA value `id` of function `func`; returns `true`
    /// if any label was new.
    fn taint_value(&mut self, func: &FunctionId, id: OpId, labels: HashSet<TaintLabel>) -> bool {
        if labels.is_empty() {
            return false;
        }
        let values = self.values.entry(func.clone()).or_default();
        extend_labels(values.entry(id).or_default(), labels)
    }
}

/// Add `labels` to `target`; returns `true` if any label was new.
fn extend_labels(target: &mut HashSet<TaintLabel>, labels: HashSet<TaintLabel>) -> bool {
    let before = target.len();
    target.extend(labels);
    target.len() != before
}

// ========================================================================
// Tests
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::AnalysisConfig;
    use scirs::bir::cfg::{BasicBlock, Terminator};
    use scirs::bir::ops::{CallOp, EnvVar, LoadOp, StoreOp};
    use scirs::sir::{BoolLit, Lit, UnOp};

    type Taint = HashMap<FunctionId, HashMap<OpId, HashSet<TaintLabel>>>;

    /// A function `name` with a single block of `ops`, `%i` being `ops[i]`.
    fn function(name: &str, ops: Vec<OpKind>) -> Function {
        let mut func = Function::new(FunctionId(name.into()), true);
        let mut entry = BasicBlock::new(BlockId(0));
        entry.ops = ops
            .into_iter()
            .enumerate()
            .map(|(i, kind)| Op::new(OpId(i), kind))
            .collect();
        entry.term = Terminator::TxnExit { reverted: false };
        func.blocks = vec![entry];
        func
    }

    /// The taint of the module `m` holding `functions`.
    fn analyze(functions: Vec<Function>) -> Taint {
        let mut module = Module::new("m".into());
        module.functions = functions;
        let mut ctx = AnalysisContext::new(vec![], AnalysisConfig::default());
        ctx.set_bir_units(vec![module]);
        TaintPass.run(&ctx).unwrap().remove("m").unwrap()
    }

    /// The labels of `%id` in function `name`.
    fn labels(taint: &Taint, name: &str, id: usize) -> HashSet<TaintLabel> {
        taint[&FunctionId(name.into())]
            .get(&OpId(id))
            .cloned()
            .unwrap_or_default()
    }

    fn sender() -> OpKind {
        OpKind::Env(EnvVar::Caller)
    }

    fn constant() -> OpKind {
        OpKind::Const(Lit::Bool(BoolLit::new(true, None)))
    }

    fn call(callee: &str, arg: usize) -> OpKind {
        let target = CallTarget::Internal(FunctionId(callee.into()));
        OpKind::Call(CallOp { target, args: vec![OpRef(OpId(arg))], value: None })
    }

    /// `id(x) { return x; }`, as function `name`.
    fn identity(name: &str) -> Function {
        function(
            name,
            vec![
                OpKind::Param { index: 0 },
                OpKind::Return(vec![OpRef(OpId(0))]),
            ],
        )
    }

    fn state(name: &str) -> Resource {
        Resource::StateVar(name.into())
    }

    fn store(resource: Resource, value: usize) -> OpKind {
        OpKind::Store(StoreOp { resource, keys: vec![], value: Some(OpRef(OpId(value))) })
    }

    fn load(resource: Resource) -> OpKind {
        OpKind::Load(LoadOp { resource, keys: vec![] })
    }

    #[test]
    fn test_taint_pass_seed_propagation() {
        let taint = analyze(vec![function("f", vec![sender()])]);
        assert!(labels(&taint, "f", 0).contains(&TaintLabel::UserControlled));
    }

    #[test]
    fn test_taint_does_not_leak_between_functions() {
        // f: %0 = env Caller. g: %0 = const, %1 = unop not %0.
        let not = OpKind::UnOp { op: UnOp::Not, operand: OpRef(OpId(0)) };
        let taint = analyze(vec![
            function("f", vec![sender()]),
            function("g", vec![constant(), not]),
        ]);
        assert!(labels(&taint, "g", 0).is_empty() && labels(&taint, "g", 1).is_empty());
        assert!(!labels(&taint, "f", 0).is_empty());
    }

    #[test]
    fn test_internal_call_returns_tainted_argument() {
        // g: %0 = env Caller, %1 = call @id(%0).
        let taint = analyze(vec![identity("id"), function("g", vec![sender(), call("id", 0)])]);
        assert!(labels(&taint, "g", 1).contains(&TaintLabel::UserControlled));
    }

    #[test]
    fn test_stored_taint_reaches_later_load() {
        // set: @owner = msg.sender. get: %0 = load @owner.
        let set = function("set", vec![sender(), store(state("owner"), 0)]);
        let get = function("get", vec![load(state("owner"))]);
        let taint = analyze(vec![set, get]);
        assert!(labels(&taint, "get", 0).contains(&TaintLabel::UserControlled));
    }

    #[test]
    fn test_untainted_call_and_state_stay_clean() {
        // g: %1 = call @id(const); @flag = const; %3 = load @flag, while
        // msg.sender only flows into another function and resource.
        let ops = vec![
            constant(),
            call("id", 0),
            store(state("flag"), 0),
            load(state("flag")),
            sender(),
            store(state("owner"), 4),
        ];
        let taint = analyze(vec![identity("id"), function("g", ops)]);
        assert!(labels(&taint, "g", 1).is_empty());
        assert_eq!(labels(&taint, "g", 3), HashSet::from([TaintLabel::StorageLoaded]));
    }
}
