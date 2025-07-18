use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::Serializer;
use std::collections::{HashMap, HashSet};
use swc_core::{
    common::BytePos,
    ecma::{
        ast::*,
        parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax},
        visit::{Visit, VisitWith},
    },
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn parse_tsx(code: &str) -> Result<Module, String> {
    let input = StringInput::new(code, BytePos(0), BytePos(code.len() as u32));

    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..Default::default()
        }),
        EsVersion::EsNext,
        input,
        None,
    );

    let mut parser = Parser::new_from(lexer);
    parser
        .parse_module()
        .map_err(|err| format!("Parse error: {:?}", err))
}

#[derive(Default)]
struct DeclExportCollector {
    exports: HashSet<String>,
    decls: HashMap<String, String>,
}

impl Visit for DeclExportCollector {
    fn visit_module_decl(&mut self, n: &ModuleDecl) {
        match n {
            ModuleDecl::ExportDecl(ExportDecl { decl, .. }) => match decl {
                Decl::Var(v) => {
                    for d in &v.decls {
                        if let Pat::Ident(BindingIdent { id, .. }) = &d.name {
                            self.exports.insert(id.sym.to_string());
                            self.decls.insert(id.sym.to_string(), format!("{:?}", d));
                        }
                    }
                }
                Decl::Fn(f) => {
                    self.exports.insert(f.ident.sym.to_string());
                    self.decls
                        .insert(f.ident.sym.to_string(), format!("{:?}", f));
                }
                Decl::Class(c) => {
                    self.exports.insert(c.ident.sym.to_string());
                    self.decls
                        .insert(c.ident.sym.to_string(), format!("{:?}", c));
                }
                _ => {}
            },
            ModuleDecl::ExportNamed(named) => {
                for spec in &named.specifiers {
                    match spec {
                        ExportSpecifier::Named(named) => {
                            let export_name = named.exported.as_ref().unwrap_or(&named.orig);
                            let name = match export_name {
                                ModuleExportName::Ident(ident) => ident.sym.to_string(),
                                ModuleExportName::Str(str_) => str_.value.to_string(),
                            };
                            self.exports.insert(name);
                        }
                        ExportSpecifier::Default(default) => {
                            self.exports.insert(default.exported.sym.to_string());
                        }
                        ExportSpecifier::Namespace(ns) => {
                            let name = match &ns.name {
                                ModuleExportName::Ident(ident) => ident.sym.to_string(),
                                ModuleExportName::Str(str_) => str_.value.to_string(),
                            };
                            self.exports.insert(name);
                        }
                    }
                }
            }
            ModuleDecl::ExportDefaultDecl(_) | ModuleDecl::ExportDefaultExpr(_) => {
                self.exports.insert("default".into());
            }
            ModuleDecl::ExportAll(export_all) => {
                // 只能记录它是 export *
                self.exports.insert("*".into());
            }

            _ => {}
        }
        n.visit_children_with(self);
    }

    fn visit_var_decl(&mut self, n: &VarDecl) {
        for decl in &n.decls {
            if let Pat::Ident(BindingIdent { id, .. }) = &decl.name {
                self.decls.insert(id.sym.to_string(), format!("{:?}", decl));
            }
        }
    }

    fn visit_fn_decl(&mut self, n: &FnDecl) {
        self.decls
            .insert(n.ident.sym.to_string(), format!("{:?}", n));
    }

    fn visit_class_decl(&mut self, n: &ClassDecl) {
        self.decls
            .insert(n.ident.sym.to_string(), format!("{:?}", n));
    }
}

#[derive(Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Removed,
    Modified,
}

#[derive(Serialize, Deserialize)]
pub struct DiffResult {
    pub name: String,
    pub change: ChangeType,
}

#[wasm_bindgen]
pub fn analyze_diff(old_code: &str, new_code: &str) -> Result<JsValue, JsValue> {
    let old_ast = parse_tsx(old_code).map_err(|e| JsValue::from_str(&e))?;
    let new_ast = parse_tsx(new_code).map_err(|e| JsValue::from_str(&e))?;

    let mut old_coll = DeclExportCollector::default();
    old_ast.visit_with(&mut old_coll);

    let mut new_coll = DeclExportCollector::default();
    new_ast.visit_with(&mut new_coll);

    let mut results = vec![];

    for export in new_coll.exports.union(&old_coll.exports) {
        let old_decl = old_coll.decls.get(export);
        let new_decl = new_coll.decls.get(export);

        let change = match (old_decl, new_decl) {
            (Some(old), Some(new)) => {
                if old != new {
                    Some(ChangeType::Modified)
                } else {
                    None
                }
            }
            (None, Some(_)) => Some(ChangeType::Added),
            (Some(_), None) => Some(ChangeType::Removed),
            _ => None,
        };

        if let Some(change) = change {
            results.push(DiffResult {
                name: export.clone(),
                change,
            });
        }
    }

    let serializer = Serializer::new().serialize_maps_as_objects(true);
    results
        .serialize(&serializer)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {:?}", e)))
}
