mod schema;

use std::env::args;
use anyhow::{Context, Result, bail};

enum Codegen {
	Schema,
}

fn get_tools() -> Result<Codegen> {
	let message = "argument is missing. Example usage: \ncargo codegen schema";
	let arg = args().nth(1).context(message)?;
	match arg.as_str() {
		"schema" => Ok(Codegen::Schema),
		arg => bail!("unknown task: {}", arg),
	}
}

fn main() -> Result<()> {
	match get_tools()? {
		Codegen::Schema => schema::generate_schema(),
	}
}
