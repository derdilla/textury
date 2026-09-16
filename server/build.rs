use std::{env, fs, path::PathBuf, io::{Read, Write}};
use typify::{TypeSpace, TypeSpaceSettings};

fn main() {
    let content = fs::read_to_string("../openapi.yaml").unwrap();
    let schema = serde_yaml::from_str::<openapiv3::OpenAPI>(&content).unwrap();

    let mut root_schema = schemars::schema::RootSchema::default();

    // Convert components/schemas into Rust types
    for (name, schema) in schema.components.unwrap().schemas {
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

    println!("cargo::rerun-if-changed=../openapi.yaml");
    println!("cargo::rerun-if-changed=build.rs");
}

fn rustfmt(unformatted: &str) -> Result<Vec<u8>, std::io::Error> {
    let process = std::process::Command::new("rustfmt")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;
    process.stdin.unwrap().write_all(unformatted.as_bytes())?;
    let mut formatted = Vec::new();
    process.stdout.unwrap().read_to_end(&mut formatted)?;
    Ok(formatted)
}
