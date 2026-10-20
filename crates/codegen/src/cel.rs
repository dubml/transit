use std::env;
use transit::cel::Expression;

pub fn evaluate_command() -> anyhow::Result<()> {
    let mut args: Vec<String> = env::args().collect();

    let ast = args.get(2).is_some_and(|arg| arg == "--ast");
    if ast {
        args.remove(2);
    }

    if args.len() != 3 && (!ast && args.len() != 4) {
        anyhow::bail!(
            "Usage: {} cel [--ast] <expression> [<json>]",
            args.first().map(|s| s.as_str()).unwrap_or("codegen")
        );
    }
    let expression = &args[2];
    let expr = Expression::new(expression)?;
    if ast {
        println!("{:#?}", expr.program());
        return Ok(());
    }

    let input = if args.len() == 4 {
        let json_input = &args[3];
        serde_json::from_str(json_input)?
    } else {
        serde_json::json!({
            "request": {
                "method": "GET",
                "path": "/v1/chat/completions",
                "headers": {
                    "user-agent": "transit/0.1"
                }
            }
        })
    };

    let res = expr.eval_json(&input)?;
    let js = serde_json::to_string_pretty(&res)?;
    println!("{}", js);
    Ok(())
}
