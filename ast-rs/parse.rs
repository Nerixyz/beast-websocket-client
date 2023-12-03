use std::path::PathBuf;

use clang::{Clang, Entity, EntityKind, EntityVisitResult, Index, Type};

use crate::model::{MemberType, StructDecl, StructMember};

pub fn parse_decls(
    header_path: impl Into<PathBuf>,
    include_dirs: &[String],
) -> anyhow::Result<Vec<StructDecl>> {
    let clang = Clang::new().unwrap();

    let index = Index::new(&clang, false, false);

    let mut args = vec!["-std=c++17"];
    args.reserve(include_dirs.len() * 2);
    for dir in include_dirs {
        args.push("-I");
        args.push(dir);
    }

    let tu = index
        .parser(header_path)
        .skip_function_bodies(true)
        .incomplete(true)
        .arguments(&args)
        .parse()?;

    for diag in tu.get_diagnostics() {
        eprintln!("{diag}");
    }

    let mut decls = vec![];

    tu.get_entity().visit_children(|entity, _| {
        if !entity.get_location().is_some_and(|l| l.is_in_main_file()) {
            return EntityVisitResult::Continue;
        }
        match entity.get_kind() {
            EntityKind::StructDecl => {
                if let Some(decl) = try_parse_struct(entity) {
                    decls.push(decl);
                }
                EntityVisitResult::Continue
            }
            EntityKind::Namespace => entity
                .get_name()
                .filter(|n| !n.starts_with("std") && !n.starts_with("boost"))
                .map_or_else(
                    || EntityVisitResult::Continue,
                    |_| EntityVisitResult::Recurse,
                ),
            _ => EntityVisitResult::Continue,
        }
    });

    Ok(decls)
}

fn try_parse_struct(entity: Entity<'_>) -> Option<StructDecl> {
    // TODO: support nested types
    // let name = entity
    //     .get_type()
    //     .map(|ty| ty.get_display_name())
    //     .filter(|n| !n.starts_with('_'))?;
    let name = entity.get_name().filter(|n| !n.starts_with('_'))?;

    let mut decl = StructDecl {
        full_name: name,
        inner_root: String::new(),
        members: vec![],
    };

    entity.visit_children(|e, _| {
        if e.get_kind() == EntityKind::FieldDecl {
            decl.members
                .push(try_parse_entity(e).expect("Failed to parse member"));
        }

        EntityVisitResult::Continue
    });

    if let Some(comment) = entity.get_comment() {
        decl.apply_commands(parse_comment_args(&comment));
    }

    Some(decl)
}

fn try_parse_entity(entity: Entity<'_>) -> Option<StructMember> {
    let name = entity.get_name()?;
    let ty = entity.get_type()?;
    let mut member = StructMember {
        name: name.clone(),
        member_type: MemberType::Basic,
        type_name: get_type_name(&ty),
        json_name: name,
        tag: None,
        dont_fail_on_deserialization: false,
        fixed_name: false,
    };

    let args = ty.get_template_argument_types();
    if let Some(Some(arg)) = args.as_ref().and_then(|a| a.first()) {
        member.type_name = arg.get_display_name();
        entity.visit_children(|e, _| {
            match e.get_kind() {
                EntityKind::TemplateRef => match e.get_name().as_deref() {
                    Some("optional") => {
                        assert!(member.member_type == MemberType::Basic);
                        member.member_type = MemberType::Optional
                    }
                    Some("vector") => match member.member_type {
                        MemberType::Basic => member.member_type = MemberType::Vector,
                        MemberType::Optional => member.member_type = MemberType::OptionalVector,
                        _ => panic!("bad member type"),
                    },
                    _ => (),
                },
                EntityKind::TypeRef => {
                    if let Some(ty) = e.get_type() {
                        member.type_name = get_type_name(&ty);
                    }
                }
                _ => (),
            };
            EntityVisitResult::Continue
        });
    }

    if let Some(comment) = entity.get_comment() {
        member.apply_commands(parse_comment_args(&comment));
    }

    Some(member)
}

fn get_type_name(ty: &Type<'_>) -> String {
    let mut type_name = ty.get_display_name();
    if ty.is_const_qualified() {
        type_name = type_name.trim_start_matches("const ").to_owned();
    }
    type_name
}

fn parse_comment_args(comment: &str) -> impl Iterator<Item = (&'_ str, &'_ str)> {
    comment
        .lines()
        .filter_map(|l| l.trim_start_matches("///").trim().split_once('='))
}
