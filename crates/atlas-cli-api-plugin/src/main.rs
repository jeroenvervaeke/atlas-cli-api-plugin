use anyhow::{bail, Context, Result};
use hierarchy::Hierarchy;
use serde_yaml;
use openapiv3::OpenAPI;

mod hierarchy;

fn main() -> Result<()> {
    let data = include_str!("../../../raw-spec.yaml");
    let spec: OpenAPI = serde_yaml::from_str(data)
        .expect("Could not deserialize input");
    

    let hierarchy = Hierarchy::from_openapi_spec("/api/atlas/v2/", &spec).context("parsing hierarchy")?;
    //let hierarchy_json = serde_json::to_string(&hierarchy).context("failed to serialize")?;
    //println!("{hierarchy_json}");
    //bail!("exit");

    let cli: clap::Command = (&hierarchy).into();
    let _matches = cli.get_matches();

    println!("todo: implement command");
    
    Ok(())
}