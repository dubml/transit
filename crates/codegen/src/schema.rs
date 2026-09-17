use std::io::Write;
use anyhow::Result;

pub fn generate_schema() -> Result<()> {
	let codegen_path = std::env::var("CARGO_MANIFEST_DIR")?;
	let schema_dir = format!("{codegen_path}/../../schema");
	fs_err::create_dir_all(&schema_dir)?;

	let config_schema = schemars::schema_for!(transit::RuntimeConfig);
	let schema_json = serde_json::to_string_pretty(&config_schema)?;
	let rule_path = format!("{schema_dir}/config.json");
	let mut file = fs_err::File::create(rule_path)?;
	file.write_all(schema_json.as_bytes())?;

	Ok(())
}
