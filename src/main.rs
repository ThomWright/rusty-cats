use canonical_path::CanonicalPath;
use error::CatsError;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};
use swc_ecma_dep_graph::DependencyDescriptor;
use parsing::{DependencyParser};

mod canonical_path;
mod error;
mod parsing;

type Dependencies = BTreeMap<CanonicalPath, Vec<CatsDependencyDescriptor>>;

fn main() -> Result<(), CatsError> {
    let parser = DependencyParser::new();

    // TODO: replace with CLI flag
    let curr_dir = std::env::current_dir()?;
    let file_path =
        CanonicalPath::try_from(&curr_dir.join(Path::new("test/index.ts")))?;

    println!("Root file: {}", file_path);

    let mut dependencies: Dependencies = BTreeMap::default();

    parser.get_deps_recursively(&mut dependencies, &file_path)?;

    print_overview(
        dependencies,
        &file_path.parent().expect("root path has no parent"),
    );

    Ok(())
}

#[derive(Debug, Clone)]
pub struct CatsDependencyDescriptor {
    descriptor: DependencyDescriptor,
    path: CanonicalPath,
}

fn print_overview(dependencies: Dependencies, root_path: &CanonicalPath) {
    println!("Root path: {}", root_path);

    for (k, v) in dependencies.iter() {
        // FIXME: I expect this might happen in normal use...
        let file = k
            .strip_prefix(&root_path)
            .expect("dependency is not below the root");

        let deps: Vec<PathBuf> = v
            .iter()
            .map(|d| {
                d.path
                    .strip_prefix(&root_path)
                    .expect("dependency is not below the root")
                    .to_owned()
            })
            .collect();

        println!("{:#?} {:#?}", file, deps);
    }
}
