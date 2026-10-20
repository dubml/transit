use anyhow::Result;
use std::io::Write;
use transit::cel::ExecutorSerde;

pub fn generate_schema() -> Result<()> {
    let codegen_path = std::env::var("CARGO_MANIFEST_DIR")?;
    let schema_dir = format!("{codegen_path}/../../schema");
    fs_err::create_dir_all(&schema_dir)?;

    // 1. Export schema/config.json
    let config_schema = schemars::schema_for!(transit::Config);
    let schema_json = serde_json::to_string_pretty(&config_schema)?;
    let rule_path = format!("{schema_dir}/config.json");
    let mut file = fs_err::File::create(rule_path)?;
    file.write_all(schema_json.as_bytes())?;

    // 2. Export schema/cel.json
    let cel_schema = schemars::schema_for!(ExecutorSerde);
    let cel_schema_json = serde_json::to_string_pretty(&cel_schema)?;
    let cel_path = format!("{schema_dir}/cel.json");
    let mut cel_file = fs_err::File::create(cel_path)?;
    cel_file.write_all(cel_schema_json.as_bytes())?;

    Ok(())
}
