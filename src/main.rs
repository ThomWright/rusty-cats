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

fn main() -> Result<(), std::io::Error> {
    let parser = DependencyParser::new();

    // TODO: replace with CLI flag
    let curr_dir = std::env::current_dir()?;
    let file_path = curr_dir.join(Path::new("test/index.ts")).canonicalize()?;
    println!("{:#?}", file_path);

    let mut all_deps: BTreeMap<PathBuf, Vec<DependencyDescriptor>> =
        BTreeMap::default();

    parser.get_deps_recursively(&mut all_deps, &file_path)?;

    println!("{:#?}", all_deps);

    Ok(())
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
        mut file_dep_map: &mut BTreeMap<PathBuf, Vec<DependencyDescriptor>>,
        file_path: &PathBuf,
    ) -> Result<(), std::io::Error> {
        // println!("get_deps_recursively: {:#?}", file_path);

        if file_dep_map.contains_key(file_path) {
            return Ok(());
        }

        let deps = self.get_deps(&file_path)?;

        // println!("Got deps for: {:#?}", file_path);

        file_dep_map.insert(file_path.clone(), deps.clone());

        let parent = file_path
            .parent()
            .expect("file should have parent directory");

        // println!("Parent: {:#?}", parent);

        for dep in deps {
            match dep.kind {
                DependencyKind::Import => {
                    let specifier = String::from(&dep.specifier as &str);
                    // println!("Import specifier: {:#?}", specifier);

                    if !specifier.starts_with('.') {
                        continue;
                    }

                    let p: PathBuf = specifier.into();
                    let mut p = parent.join(p);
                    if p.is_dir() {
                        p = p.join("index");
                    }
                    if p.extension().is_none() {
                        p.set_extension("ts");
                    }
                    // println!("Aliased import: {:#?}", p);

                    let p = p.canonicalize()?;
                    // println!("Canonicalised: {:#?}", p);

                    self.get_deps_recursively(&mut file_dep_map, &p)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn get_deps(
        &self,
        file_path: &Path,
    ) -> Result<Vec<DependencyDescriptor>, std::io::Error> {
        // println!("get_deps: {:#?}", file_path);

        let fm = self.source_map.load_file(&file_path)?;

        // println!("Loaded: {:#?}", file_path);

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
