use std::{env, path::PathBuf};

use rayon::iter::{ParallelBridge, ParallelIterator};

mod apply;
mod cmake;
mod model;
mod parse;
mod render;

fn main() -> anyhow::Result<()> {
    if env::args().len() < 2 {
        eprintln!("Usage: {} <path-to-dir>", env::args().next().unwrap());
        std::process::exit(1); // could've used exit code
    }
    let include_dirs = cmake::get_include_dirs()?;
    eprintln!("Include dirs: {include_dirs:?}");

    let dir = env::args().nth(1).unwrap();
    let dirpath = format!("include/eventsub/{}", &dir);
    std::fs::read_dir(dirpath)?
        .map_while(Result::ok)
        .filter(|it| it.file_type().is_ok_and(|f| f.is_file()))
        .filter(|it| it.path().extension().is_some_and(|e| "hpp" == e))
        .filter_map(|it| {
            let header_path = it.path();
            let mut source_path = PathBuf::from("src");
            source_path.push(&dir);
            source_path.push(it.file_name());
            source_path.set_extension("cpp");

            if source_path.is_file() {
                Some((header_path, source_path))
            } else {
                None
            }
        })
        .map(|(header_path, source_path)| {
            eprintln!("-> {}", header_path.display());
            (
                header_path.clone(),
                source_path,
                parse::parse_decls(header_path, &include_dirs).expect("clang error"),
            )
        })
        .par_bridge()
        .try_for_each(|(header_path, source_path, decls)| {
            let (defs, impls) = render::render_decls(&decls)?;
            apply::definition(header_path, &defs)?;
            apply::implementation(source_path, &impls)?;
            Ok::<_, anyhow::Error>(())
        })?;

    Ok(())
}
