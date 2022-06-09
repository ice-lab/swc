use easy_error::{Error};
use fxhash::FxHashSet;
use std::mem::take;
use swc_common::pass::{Repeat, Repeated};
use swc_common::DUMMY_SP;
use swc_ecmascript::ast::*;
use swc_ecmascript::visit::FoldWith;
use swc_ecmascript::visit::{noop_fold_type, Fold};

/// Note: This paths requires running `resolver` **before** running this.
pub fn remove_export_exprs(remove_exports: Vec<String>) -> impl Fold {
    Repeat::new(RemoveExportsExprs {
        state: State {
            remove_exports: remove_exports,
            ..Default::default()
        },
        in_lhs_of_var: false,
    })
}

/// State of the transforms. Shared by the analyzer and the transform.
#[derive(Debug, Default)]
struct State {
    /// Identifiers referenced by non-data function codes.
    ///
    /// Cleared before running each pass, because we drop ast nodes between the
    /// passes.
    refs_from_other: FxHashSet<Id>,

    /// Identifiers referenced by data functions or derivatives.
    ///
    /// Preserved between runs, because we should remember derivatives of data
    /// functions as the data function itself is already removed.
    refs_from_data_fn: FxHashSet<Id>,

    cur_declaring: FxHashSet<Id>,

    should_run_again: bool,
    remove_exports: Vec<String>,
}

impl State {
    fn should_remove_identifier(&mut self, i: &Ident) -> Result<bool, Error> {
        Ok(self.remove_exports.contains(&String::from(&*i.sym)))
    }
    fn should_remove_default(&mut self) -> bool {
        self.remove_exports.contains(&String::from("default"))
    }
}

struct Analyzer<'a> {
    state: &'a mut State,
    in_lhs_of_var: bool,
    in_data_fn: bool,
}

impl Analyzer<'_> {
    fn add_ref(&mut self, id: Id) {
        tracing::trace!("add_ref({}{:?}, data = {})", id.0, id.1, self.in_data_fn);
        if self.in_data_fn {
            self.state.refs_from_data_fn.insert(id);
        } else {
            if self.state.cur_declaring.contains(&id) {
                return;
            }

            self.state.refs_from_other.insert(id);
        }
    }
}

impl Fold for Analyzer<'_> {
    // This is important for reducing binary sizes.
    noop_fold_type!();

    fn fold_binding_ident(&mut self, i: BindingIdent) -> BindingIdent {
        if !self.in_lhs_of_var || self.in_data_fn {
            self.add_ref(i.id.to_id());
        }

        i
    }

    fn fold_export_named_specifier(&mut self, s: ExportNamedSpecifier) -> ExportNamedSpecifier {
        if let ModuleExportName::Ident(id) = &s.orig {
            if !self.state.remove_exports.contains(&String::from(&*id.sym)) {
                self.add_ref(id.to_id());
            }
        }

        s
    }

    fn fold_export_decl(&mut self, s: ExportDecl) -> ExportDecl {
        if let Decl::Var(d) = &s.decl {
            if d.decls.is_empty() {
                return s;
            }

            if let Pat::Ident(id) = &d.decls[0].name {
                if !self.state.remove_exports.contains(&String::from(&*id.id.sym)) {
                    self.add_ref(id.to_id());
                }
            }
        }

        s.fold_children_with(self)
    }

    fn fold_expr(&mut self, e: Expr) -> Expr {
        let e = e.fold_children_with(self);

        if let Expr::Ident(i) = &e {
            self.add_ref(i.to_id());
        }

        e
    }

    fn fold_jsx_element(&mut self, jsx: JSXElement) -> JSXElement {
        fn get_leftmost_id_member_expr(e: &JSXMemberExpr) -> Id {
            match &e.obj {
                JSXObject::Ident(i) => i.to_id(),
                JSXObject::JSXMemberExpr(e) => get_leftmost_id_member_expr(e),
            }
        }

        match &jsx.opening.name {
            JSXElementName::Ident(i) => {
                self.add_ref(i.to_id());
            }
            JSXElementName::JSXMemberExpr(e) => {
                self.add_ref(get_leftmost_id_member_expr(e));
            }
            _ => {}
        }

        jsx.fold_children_with(self)
    }

    fn fold_fn_decl(&mut self, f: FnDecl) -> FnDecl {
        let old_in_data = self.in_data_fn;

        self.state.cur_declaring.insert(f.ident.to_id());

        if let Ok(should_remove_identifier) = self.state.should_remove_identifier(&f.ident) {
            self.in_data_fn |= should_remove_identifier;
        } else {
            return f;
        }
        tracing::trace!(
            "remove_export_exprs: Handling `{}{:?}`; in_data_fn = {:?}",
            f.ident.sym,
            f.ident.span.ctxt,
            self.in_data_fn
        );

        let f = f.fold_children_with(self);

        self.state.cur_declaring.remove(&f.ident.to_id());

        self.in_data_fn = old_in_data;

        f
    }

    fn fold_fn_expr(&mut self, f: FnExpr) -> FnExpr {
        let f = f.fold_children_with(self);

        if let Some(id) = &f.ident {
            self.add_ref(id.to_id());
        }

        f
    }

    /// Drops [ExportDecl] if all specifiers are removed.
    fn fold_module_item(&mut self, s: ModuleItem) -> ModuleItem {
        match s {
            ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(e)) if !e.specifiers.is_empty() => {
                let e = e.fold_with(self);

                if e.specifiers.is_empty() {
                    return ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP }));
                }

                return ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(e));
            }
            _ => {}
        };

        // Visit children to ensure that all references is added to the scope.
        let s = s.fold_children_with(self);

        if let ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(e)) = &s {
            match &e.decl {
                Decl::Fn(f) => {
                    if let Ok(should_remove_identifier) = self.state.should_remove_identifier(&f.ident) {
                        if should_remove_identifier {
                            return ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP }));
                        }
                    } else {
                        return s;
                    }
                }

                Decl::Var(d) => {
                    if d.decls.is_empty() {
                        return ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP }));
                    }
                }
                _ => {}
            }
        }

        s
    }

    fn fold_named_export(&mut self, mut n: NamedExport) -> NamedExport {
        if n.src.is_some() {
            n.specifiers = n.specifiers.fold_with(self);
        }

        n
    }

    // fn fold_default_decl(&mut self, d: DefaultDecl) -> DefaultDecl {
    //     let old_in_data = self.in_data_fn;

    //     if self.state.should_remove_default() {
    //         self.in_data_fn = true;

    //         let d = d.fold_children_with(self);

    //         self.in_data_fn = old_in_data;

    //         return d
    //     }

    //     d
    // }

    // fn fold_export_default_expr(&mut self, e: ExportDefaultExpr) -> ExportDefaultExpr {
    //     let old_in_data = self.in_data_fn;

    //     if self.state.should_remove_default() {
    //         self.in_data_fn = true;

    //         let e = e.fold_children_with(self);

    //         self.in_data_fn = old_in_data;

    //         return e
    //     }

    //     e
    // }

    fn fold_prop(&mut self, p: Prop) -> Prop {
        let p = p.fold_children_with(self);

        if let Prop::Shorthand(i) = &p {
            self.add_ref(i.to_id());
        }

        p
    }

    fn fold_var_declarator(&mut self, mut v: VarDeclarator) -> VarDeclarator {
        let old_in_data = self.in_data_fn;

        if let Pat::Ident(name) = &v.name {
            if let Ok(should_remove_identifier) = self.state.should_remove_identifier(&name.id) {
                if should_remove_identifier {
                    self.in_data_fn = true;
                }
            } else {
                return v;
            }
        }

        let old_in_lhs_of_var = self.in_lhs_of_var;

        self.in_lhs_of_var = true;
        v.name = v.name.fold_with(self);

        self.in_lhs_of_var = false;
        v.init = v.init.fold_with(self);

        self.in_lhs_of_var = old_in_lhs_of_var;

        self.in_data_fn = old_in_data;

        v
    }
}

/// Actual implementation of the transform.
struct RemoveExportsExprs {
    pub state: State,
    in_lhs_of_var: bool,
}

impl RemoveExportsExprs {
    fn should_remove(&self, id: Id) -> bool {
        self.state.refs_from_data_fn.contains(&id) && !self.state.refs_from_other.contains(&id)
    }

    /// Mark identifiers in `n` as a candidate for removal.
    fn mark_as_candidate<N>(&mut self, n: N) -> N
    where
        N: for<'aa> FoldWith<Analyzer<'aa>>,
    {
        tracing::debug!("mark_as_candidate");

        // Analyzer never change `in_data_fn` to false, so all identifiers in `n` will
        // be marked as referenced from a data function.
        let mut v = Analyzer {
            state: &mut self.state,
            in_lhs_of_var: false,
            in_data_fn: true,
        };

        let n = n.fold_with(&mut v);
        self.state.should_run_again = true;
        n
    }
}

impl Repeated for RemoveExportsExprs {
    fn changed(&self) -> bool {
        self.state.should_run_again
    }

    fn reset(&mut self) {
        self.state.refs_from_other.clear();
        self.state.cur_declaring.clear();
        self.state.should_run_again = false;
    }
}

/// `VisitMut` is faster than [Fold], but we use [Fold] because it's much easier
/// to read.
///
/// Note: We don't implement `fold_script` because next.js doesn't use it.
impl Fold for RemoveExportsExprs {
    // This is important for reducing binary sizes.
    noop_fold_type!();

    // Remove import expression
    fn fold_import_decl(&mut self, mut i: ImportDecl) -> ImportDecl {
        // Imports for side effects.
        if i.specifiers.is_empty() {
            return i;
        }

        i.specifiers.retain(|s| match s {
            ImportSpecifier::Named(ImportNamedSpecifier { local, .. })
            | ImportSpecifier::Default(ImportDefaultSpecifier { local, .. })
            | ImportSpecifier::Namespace(ImportStarAsSpecifier { local, .. }) => {
                if self.should_remove(local.to_id()) {
                    tracing::trace!(
                        "Dropping import `{}{:?}` because it should be removed",
                        local.sym,
                        local.span.ctxt
                    );

                    self.state.should_run_again = true;
                    false
                } else {
                    true
                }
            }
        });

        i
    }

    fn fold_module(&mut self, mut m: Module) -> Module {
        tracing::info!("remove_export_exprs: Start");
        {
            // Fill the state.
            let mut v = Analyzer {
                state: &mut self.state,
                in_lhs_of_var: false,
                in_data_fn: false,
            };
            m = m.fold_with(&mut v);
        }

        m.fold_children_with(self)
    }

    fn fold_module_items(&mut self, mut items: Vec<ModuleItem>) -> Vec<ModuleItem> {
        items = items.fold_children_with(self);

        // Drop nodes.
        items.retain(|s| !matches!(s, ModuleItem::Stmt(Stmt::Empty(..))));

        items
    }

    fn fold_module_item(&mut self, i: ModuleItem) -> ModuleItem {
        if let ModuleItem::ModuleDecl(ModuleDecl::Import(i)) = i {
            let is_for_side_effect = i.specifiers.is_empty();
            let i = i.fold_with(self);

            if !is_for_side_effect && i.specifiers.is_empty() {
                return ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP }));
            }

            return ModuleItem::ModuleDecl(ModuleDecl::Import(i));
        }

        let i = i.fold_children_with(self);

        match &i {
            ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(e)) if e.specifiers.is_empty() => {
                return ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP }))
            }
            _ => {}
        }

        i
    }

    fn fold_named_export(&mut self, mut n: NamedExport) -> NamedExport {
        n.specifiers = n.specifiers.fold_with(self);

        n.specifiers.retain(|s| {
            let preserve = match s {
                ExportSpecifier::Namespace(ExportNamespaceSpecifier {
                    name: ModuleExportName::Ident(exported),
                    ..
                })
                | ExportSpecifier::Default(ExportDefaultSpecifier { exported, .. })
                | ExportSpecifier::Named(ExportNamedSpecifier {
                    exported: Some(ModuleExportName::Ident(exported)),
                    ..
                }) => self
                    .state
                    .should_remove_identifier(exported)
                    .map(|should_remove_identifier| !should_remove_identifier),
                ExportSpecifier::Named(ExportNamedSpecifier {
                    orig: ModuleExportName::Ident(orig),
                    ..
                }) => self
                    .state
                    .should_remove_identifier(orig)
                    .map(|should_remove_identifier| !should_remove_identifier),
                _ => Ok(true),
            };

            match preserve {
                Ok(false) => {
                    tracing::trace!("Dropping a export specifier because it's a data identifier");

                    if let ExportSpecifier::Named(ExportNamedSpecifier {
                        orig: ModuleExportName::Ident(orig),
                        ..
                    }) = s
                    {
                        self.state.should_run_again = true;
                        self.state.refs_from_data_fn.insert(orig.to_id());
                    }

                    false
                }
                Ok(true) => true,
                Err(_) => false,
            }
        });

        n
    }

    fn fold_default_decl(&mut self, d: DefaultDecl) -> DefaultDecl {
        if self.state.should_remove_default() {
            // Replace with an empty function
            return DefaultDecl::Fn(FnExpr {
                ident: None,
                function: Function {
                    params: vec![],
                    body: Some(BlockStmt {
                        span: DUMMY_SP,
                        stmts: vec![]
                    }),
                    span: DUMMY_SP,
                    is_generator: false,
                    is_async: false,
                    decorators: vec![],
                    return_type: None,
                    type_params: None,
                }
            })
        }
        d
    }

    fn fold_export_default_expr(&mut self, n: ExportDefaultExpr) -> ExportDefaultExpr {
        if self.state.should_remove_default() {
            // Replace with an empty function
            return ExportDefaultExpr {
                span: DUMMY_SP,
                expr: Box::new(Expr::Fn(FnExpr {
                    ident: None,
                    function: Function {
                        params: vec![],
                        body: Some(BlockStmt {
                            span: DUMMY_SP,
                            stmts: vec![]
                        }),
                        span: DUMMY_SP,
                        is_generator: false,
                        is_async: false,
                        decorators: vec![],
                        return_type: None,
                        type_params: None,
                    }
                }))
            };
        }
        n
    }

    /// This methods returns [Pat::Invalid] if the pattern should be removed.
    fn fold_pat(&mut self, mut p: Pat) -> Pat {
        p = p.fold_children_with(self);

        if self.in_lhs_of_var {
            match &mut p {
                Pat::Ident(name) => {
                    if self.should_remove(name.id.to_id()) {
                        self.state.should_run_again = true;
                        tracing::trace!(
                            "Dropping var `{}{:?}` because it should be removed",
                            name.id.sym,
                            name.id.span.ctxt
                        );

                        return Pat::Invalid(Invalid { span: DUMMY_SP });
                    }
                } 
                Pat::Array(arr) => {
                    if !arr.elems.is_empty() {
                        arr.elems.retain(|e| !matches!(e, Some(Pat::Invalid(..))));

                        if arr.elems.is_empty() {
                            return Pat::Invalid(Invalid { span: DUMMY_SP });
                        }
                    }
                }
                Pat::Object(obj) => {
                    if !obj.props.is_empty() {
                        obj.props = take(&mut obj.props)
                            .into_iter()
                            .filter_map(|prop| match prop {
                                ObjectPatProp::KeyValue(prop) => {
                                    if prop.value.is_invalid() {
                                        None
                                    } else {
                                        Some(ObjectPatProp::KeyValue(prop))
                                    }
                                }
                                ObjectPatProp::Assign(prop) => {
                                    if self.should_remove(prop.key.to_id()) {
                                        self.mark_as_candidate(prop.value);

                                        None
                                    } else {
                                        Some(ObjectPatProp::Assign(prop))
                                    }
                                }
                                ObjectPatProp::Rest(prop) => {
                                    if prop.arg.is_invalid() {
                                        None
                                    } else {
                                        Some(ObjectPatProp::Rest(prop))
                                    }
                                }
                            })
                            .collect();

                        if obj.props.is_empty() {
                            return Pat::Invalid(Invalid { span: DUMMY_SP });
                        }
                    }
                }
                Pat::Rest(rest) => {
                    if rest.arg.is_invalid() {
                        return Pat::Invalid(Invalid { span: DUMMY_SP });
                    }
                }
                _ => {}
            }
        }

        p
    }

    #[allow(clippy::single_match)]
    fn fold_stmt(&mut self, mut s: Stmt) -> Stmt {
        match s {
            Stmt::Decl(Decl::Fn(f)) => {
                if self.should_remove(f.ident.to_id()) {
                    self.mark_as_candidate(f.function);
                    return Stmt::Empty(EmptyStmt { span: DUMMY_SP });
                }

                s = Stmt::Decl(Decl::Fn(f));
            }
            _ => {}
        }

        let s = s.fold_children_with(self);
        match s {
            Stmt::Decl(Decl::Var(v)) if v.decls.is_empty() => {
                return Stmt::Empty(EmptyStmt { span: DUMMY_SP });
            }
            _ => {}
        }

        s
    }

    /// This method make `name` of [VarDeclarator] to [Pat::Invalid] if it
    /// should be removed.
    fn fold_var_declarator(&mut self, mut d: VarDeclarator) -> VarDeclarator {
        let old = self.in_lhs_of_var;
        self.in_lhs_of_var = true;
        let name = d.name.fold_with(self);

        self.in_lhs_of_var = false;
        if name.is_invalid() {
            d.init = self.mark_as_candidate(d.init);
        }
        let init = d.init.fold_with(self);
        self.in_lhs_of_var = old;

        VarDeclarator { name, init, ..d }
    }

    fn fold_var_declarators(&mut self, mut decls: Vec<VarDeclarator>) -> Vec<VarDeclarator> {
        decls = decls.fold_children_with(self);
        decls.retain(|d| !d.name.is_invalid());

        decls
    }
}
