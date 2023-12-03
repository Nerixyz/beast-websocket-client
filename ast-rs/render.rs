use std::{
    env,
    io::Write,
    process::{Command, Stdio},
};

use minijinja::{context, path_loader, Environment};

use crate::model::{MemberTypeGlobal, StructDecl};

pub fn render_decls(decls: &[StructDecl]) -> anyhow::Result<(String, String)> {
    let mut env = Environment::new();
    env.add_global(
        "MemberType",
        minijinja::Value::from_struct_object(MemberTypeGlobal),
    );
    env.set_loader(path_loader("ast/lib/templates"));
    let def = env.get_template("struct-definition.tmpl").unwrap();
    let imp = env.get_template("struct-implementation.tmpl").unwrap();

    let (defs, impls) = decls.iter().try_fold(
        (String::new(), String::new()),
        |(mut defs, mut impls), decl| {
            let ctx = context!(struct => decl);

            if !defs.is_empty() {
                defs.push_str("\n\n");
            }
            defs.push_str(&def.render(&ctx)?);

            if !impls.is_empty() {
                impls.push_str("\n\n");
            }
            impls.push_str(&imp.render(&ctx)?);

            Ok::<_, anyhow::Error>((defs, impls))
        },
    )?;

    let (defs, impls) = rayon::join(|| clang_format(&defs), || clang_format(&impls));

    Ok((defs?, impls?))
}

fn clang_format(input: &str) -> anyhow::Result<String> {
    let clang_binary = env::var("CLANG_FORMAT_BINARY").unwrap_or("clang-format".to_string());
    let mut child = Command::new(clang_binary.as_str())
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    {
        let mut stdin = child.stdin.take().expect("no stdin");
        write!(stdin, "{}", input)?;
    }

    let output = child.wait_with_output()?;
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        Err(anyhow::anyhow!(
            "clang-tidy: bad status code - {:?}",
            output.status.code()
        ))
    }
}
