use std::path::PathBuf;
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

use crate::{
  canonical_path::CanonicalPath, CatsDependencyDescriptor, Dependencies,
};

pub struct DependencyParser {
  source_map: Lrc<SourceMap>,
  handler: Handler,
}

impl DependencyParser {
  pub fn new() -> DependencyParser {
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

  pub fn get_deps_recursively(
    &self,
    mut file_dep_map: &mut Dependencies,
    file_path: &CanonicalPath,
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
        let p = parent
          .resolve_ts_file(p)
          .expect("unable to resolve TS file");

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
    file_path: &CanonicalPath,
  ) -> Result<Vec<DependencyDescriptor>, std::io::Error> {
    // println!("get_deps: {:#?}", file_path);

    let fm = self.source_map.load_file(file_path.as_ref())?;

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
