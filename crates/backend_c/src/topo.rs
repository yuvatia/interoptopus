//! Topological sorting of types by their dependencies.

use interoptopus::inventory::{RustInventory, TypeId};
use interoptopus::lang::types::{Type, TypeKind, TypePattern, VariantKind};
use std::collections::HashSet;

/// Returns type IDs in dependency order (dependencies before dependents).
pub fn sort_types(inv: &RustInventory) -> Vec<TypeId> {
    let known: HashSet<TypeId> = inv.types.keys().copied().collect();
    let mut visited = HashSet::new();
    let mut order = Vec::new();

    for &tid in inv.types.keys() {
        visit(tid, inv, &known, &mut visited, &mut order);
    }
    order
}

fn visit(tid: TypeId, inv: &RustInventory, known: &HashSet<TypeId>, visited: &mut HashSet<TypeId>, order: &mut Vec<TypeId>) {
    if !known.contains(&tid) || !visited.insert(tid) {
        return;
    }
    for dep in deps(&inv.types[&tid]) {
        visit(dep, inv, known, visited, order);
    }
    order.push(tid);
}

fn deps(ty: &Type) -> Vec<TypeId> {
    match &ty.kind {
        TypeKind::Struct(s) => s.fields.iter().map(|f| f.ty).collect(),
        TypeKind::Enum(e) => e
            .variants
            .iter()
            .filter_map(|v| match &v.kind {
                VariantKind::Tuple(tid) => Some(*tid),
                VariantKind::Unit(_) => None,
            })
            .collect(),
        TypeKind::TypePattern(TypePattern::Slice(t) | TypePattern::SliceMut(t) | TypePattern::Vec(t) | TypePattern::Option(t)) => {
            vec![*t]
        }
        TypeKind::TypePattern(TypePattern::Result(ok, err)) => vec![*ok, *err],
        TypeKind::TypePattern(TypePattern::NamedCallback(sig)) | TypeKind::FnPointer(sig) => {
            let mut d: Vec<_> = sig.arguments.iter().map(|a| a.ty).collect();
            d.push(sig.rval);
            d
        }
        TypeKind::ReadPointer(t) | TypeKind::ReadWritePointer(t) => vec![*t],
        _ => vec![],
    }
}
