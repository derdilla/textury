use std::{env, fs, path::PathBuf, io::{Read, Write}};
use typify::{TypeSpace, TypeSpaceSettings};

fn main() {
    let content = fs::read_to_string("../openapi.yaml").unwrap();
    let mut schema = serde_yaml::from_str::<openapiv3::OpenAPI>(&content).unwrap();

    let mut root_schema = schemars::schema::RootSchema::default();

    // Convert components/schemas into Rust types
    for (name, schema) in schema.components.clone().unwrap().schemas {
        let schema = serde_yaml::from_str::<schemars::schema::Schema>(
            &serde_yaml::to_string(&schema).unwrap()
        ).unwrap();
        root_schema.definitions.insert(name, schema);
    }


    let mut type_space = TypeSpace::new(TypeSpaceSettings::default().with_conversion(
        schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::String.into()),
            format: Some("date-time".to_string()),
            ..Default::default()
        },
        "::time::OffsetDateTime",
        [
            typify::TypeSpaceImpl::Display,
            typify::TypeSpaceImpl::FromStr,
        ].into_iter(),
    ));
    type_space.add_root_schema(root_schema).expect("Failed to add root schema to typify");

    let contents = type_space.to_stream().to_string();

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    if let Ok(formatted) = rustfmt(&contents) {
        fs::write(out_dir.join("api_types.rs"), formatted).unwrap();
    } else {
        // Maybe rustfmt isn’t installed
        fs::write(out_dir.join("api_types.rs"), contents).unwrap();
    }

    // Add missing operationIds for test client generation
    // https://github.com/oxidecomputer/progenitor/issues/1105
    let mut new_paths = openapiv3::Paths::default();
    for (path, path_item) in schema.paths {
        if let openapiv3::ReferenceOr::Item(mut item) = path_item {
            let operations = [
                ("get", &mut item.get),
                ("post", &mut item.post),
                ("put", &mut item.put),
                ("delete", &mut item.delete),
                ("options", &mut item.options),
                ("head", &mut item.head),
                ("patch", &mut item.patch),
                ("trace", &mut item.trace),
            ];

            for (method, op) in operations {
                if let Some(op) = op {
                    if op.operation_id.is_none() {
                        let clean = path.replace('/', "_").replace(['{', '}'], "");
                        op.operation_id = Some(format!("{method}_{clean}"));
                    }
                }
            }
            new_paths.paths.insert(path, openapiv3::ReferenceOr::Item(item));
        } else {
            new_paths.paths.insert(path, path_item);
        }
    }
    schema.paths = new_paths;

    // Generate test client
    let mut settings = progenitor::GenerationSettings::default();
    for (name, _) in schema.components.clone().unwrap().schemas {
        settings.with_replacement(&name, format!("server::api::types::{}", name), [].into_iter());
    }
    let mut generator = progenitor::Generator::new(&settings);
    let result = generator.generate_tokens(&schema).unwrap();

    let out_path = out_dir.join("api_client.rs");
    let contents = result.to_string();
    if let Ok(formatted) = rustfmt(&contents) {
        fs::write(&out_path, formatted).unwrap();
    } else {
        fs::write(&out_path, contents).unwrap();
    }


    println!("cargo::rerun-if-changed=../openapi.yaml");
    println!("cargo::rerun-if-changed=build.rs");
}

fn rustfmt(unformatted: &str) -> Result<Vec<u8>, std::io::Error> {
    let process = std::process::Command::new("rustfmt")
        .arg("--edition").arg("2024")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;
    process.stdin.unwrap().write_all(unformatted.as_bytes())?;
    let mut formatted = Vec::new();
    process.stdout.unwrap().read_to_end(&mut formatted)?;
    Ok(formatted)
}
