use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use swc_common::{
    comments::SingleThreadedComments,
    errors::{ColorConfig, Handler},
    sync::Lrc,
    SourceMap,
};
use swc_ecma_dep_graph::{
    analyze_dependencies, DependencyDescriptor, DependencyKind,
};
use swc_ecma_parser::{lexer::Lexer, Capturing, Parser, StringInput, Syntax};

type Dependencies = BTreeMap<PathBuf, Vec<CatsDependencyDescriptor>>;

fn main() -> Result<(), std::io::Error> {
    let parser = DependencyParser::new();

    // TODO: replace with CLI flag
    let curr_dir = std::env::current_dir()?;
    let file_path = curr_dir.join(Path::new("test/index.ts")).canonicalize()?;
    println!("{:#?}", file_path);

    let mut dependencies: Dependencies = BTreeMap::default();

    parser.get_deps_recursively(&mut dependencies, &file_path)?;

    print_overview(
        dependencies,
        &file_path.parent().expect("root path has no parent"),
    );

    Ok(())
}

#[derive(Debug, Clone)]
struct CatsDependencyDescriptor {
    descriptor: DependencyDescriptor,
    path: PathBuf,
}

struct DependencyParser {
    source_map: Lrc<SourceMap>,
    handler: Handler,
}

impl DependencyParser {
    fn new() -> DependencyParser {
        let source_map: Lrc<SourceMap> = Default::default();

        let handler = Handler::with_tty_emitter(
            ColorConfig::Auto,
            true,
            false,
            Some(source_map.clone()),
        );

        DependencyParser {
            source_map,
            handler,
        }
    }

    fn get_deps_recursively(
        &self,
        mut file_dep_map: &mut Dependencies,
        file_path: &PathBuf,
    ) -> Result<(), std::io::Error> {
        // println!("get_deps_recursively: {:#?}", file_path);

        if file_dep_map.contains_key(file_path) {
            return Ok(());
        }

        let parent = file_path
            .parent()
            .expect("file should have parent directory");

        let deps = self.get_deps(&file_path)?;

        let deps: Vec<CatsDependencyDescriptor> = deps
            .iter()
            .filter(|dep| {
                dep.kind == DependencyKind::Import
                    && (&dep.specifier as &str).starts_with('.')
            })
            .map(|dep| {
                let p: PathBuf = (&dep.specifier as &str).into();
                let mut p = parent.join(p);
                if p.is_dir() {
                    p = p.join("index");
                }
                if p.extension().is_none() {
                    p.set_extension("ts");
                }
                // println!("Aliased import: {:#?}", p);

                let p = p.canonicalize().expect("unable to canocicalise path");

                CatsDependencyDescriptor {
                    descriptor: dep.clone(),
                    path: p,
                }
            })
            .collect();

        file_dep_map.insert(file_path.clone(), deps.clone());

        for dep in deps {
            self.get_deps_recursively(&mut file_dep_map, &dep.path)?;
        }

        Ok(())
    }

    fn get_deps(
        &self,
        file_path: &Path,
    ) -> Result<Vec<DependencyDescriptor>, std::io::Error> {
        // println!("get_deps: {:#?}", file_path);

        let fm = self.source_map.load_file(&file_path)?;

        let comments = SingleThreadedComments::default();
        let lexer = Lexer::new(
            Syntax::Typescript(Default::default()),
            Default::default(),
            StringInput::from(&*fm),
            Some(&comments),
        );

        let capturing = Capturing::new(lexer);

        let mut parser = Parser::new_from(capturing);

        for e in parser.take_errors() {
            e.into_diagnostic(&self.handler).emit();
        }

        let module = parser
            .parse_typescript_module()
            .map_err(|e| e.into_diagnostic(&self.handler).emit())
            .expect("Failed to parse module.");

        Ok(analyze_dependencies(&module, &self.source_map, &comments))
    }
}

fn print_overview(dependencies: Dependencies, root_path: &Path) {
    println!("{:#?}", root_path);

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
