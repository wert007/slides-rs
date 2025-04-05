use std::env;
use std::fs;
use std::path::Path;

struct FunctionDefinition {
    name: String,
    parameters: Vec<String>,
    return_type: String,
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("global_functions.rs");
    let source_code = fs::read_to_string("src/compiler/evaluator/functions.rs").unwrap();
    let mut functions = Vec::new();
    for line in source_code.lines() {
        let line = line.trim();
        let Some(line) = line.strip_prefix("pub fn ") else {
            continue;
        };
        let Some((name, line)) = line.split_once('(') else {
            continue;
        };
        let Some((parameters, line)) = line.split_once(')') else {
            continue;
        };
        let parameters = parse_parameters(parameters);
        let return_type = line
            .trim()
            .trim_start_matches("->")
            .trim_end_matches("{")
            .trim();
        functions.push(FunctionDefinition {
            name: name.into(),
            parameters,
            return_type: return_type.into(),
        });
    }

    let args = |f: &FunctionDefinition| {
        f.parameters
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let type_name = match p.as_str() {
                    "i64" => "integer".to_owned(),
                    "f64" => "float".to_owned(),
                    "PathBuf" => "path".to_owned(),
                    "String" | "Background" | "Color" | "Label" | "Image" => p.to_ascii_lowercase(),
                    "StringArray" => "string_array".to_owned(),
                    _ => unreachable!("Unexpected type {p}"),
                };
                format!("args[{i}].as_{type_name}().clone()")
            })
            .collect::<Vec<String>>()
            .join(", ")
    };

    let functions_as_array = functions
        .iter()
        .map(|f| {
            format!(
                "    FunctionDefinition {{
        name: {:?},
        parameters: &[{}],
        return_type: Type::from_rust_string({:?}).unwrap(),
        call: |mut args| {{
        assert_eq!(args.len(), {});
            {}({}).into()
        }}
    }}",
                f.name,
                f.parameters
                    .iter()
                    .map(|p| format!("Type::from_rust_string({p:?}).unwrap()"))
                    .collect::<Vec<String>>()
                    .join(", "),
                f.return_type,
                f.parameters.len(),
                f.name,
                args(f),
            )
        })
        .collect::<Vec<String>>()
        .join(",\n");

    fs::write(
        &dest_path,
        format!(
            "
use crate::compiler::binder::{{Type, Value}};
use crate::compiler::evaluator::functions::*;

pub struct FunctionDefinition {{
    pub name: &'static str,
    pub parameters: &'static [Type],
    pub return_type: Type,
    pub call: fn(Vec<Value>) -> Value,
}}

pub const FUNCTIONS: [FunctionDefinition; {}] = [
{functions_as_array}
];",
            functions.len(),
        ),
    )
    .unwrap();
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=src/compiler/evaluator/functions.rs");
}

fn parse_parameters(parameters: &str) -> Vec<String> {
    parameters
        .split(',')
        .map(|p| p.trim().split_once(':').unwrap().1.trim().to_owned())
        .collect()
}
