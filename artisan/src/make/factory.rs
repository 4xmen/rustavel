use std::time::Instant;
use clap::Args;
use illuminate_str::Str;
use minijinja::Environment;
use rustavel_core::facades::file_content::FileContent;
use rustavel_core::facades::terminal_ui::{operation, Status};
use crate::general::lib::register_mod_file;
use crate::make::make_error::MakeError;

#[derive(Args, Debug)]
#[command(about = "Create a new factory file")]
pub struct NewFactoryArgs {
    /// The name of the factory
    pub name: String,

    /// model name
    #[arg(long)]
    pub model: Option<String>,
}

#[derive(serde::Serialize)]
struct FactoryContext {
    name: String,
    model: String,
    has_model: bool,
}

const FACTORY_TEMPLATE: &str = include_str!("templates/factory.rs.j2");
pub async fn factory(args: &NewFactoryArgs) -> Result<(), MakeError> {
    let start = Instant::now();
    let factory_name = Str::ucfirst(&Str::singular(&args.name));

    let factory_name = if factory_name.ends_with("Factory"){
        factory_name
    }else{
        format!("{}Factory", factory_name)
    };



    let model_name = match &args.model {
        Some(model) => Str::ucfirst(model).to_string(),
        None => "".to_string(),
    };


    // add template and render
    let mut env = Environment::new();
    env.add_template("factory", FACTORY_TEMPLATE)?;

    let ctx = FactoryContext {
        name: factory_name.clone(),
        model: model_name.clone(),
        has_model: model_name.len() > 0,
    };
    let rendered = env.get_template("factory")?.render(ctx)?;

    let base = std::env::current_dir()?.join("database/src/factories/");
    let generated_path = base.join(format!("{}.rs", Str::snake(&factory_name,"_")));

    // Write controller file
    FileContent::put(generated_path.to_str().unwrap(), &rendered).await?;


    // Register in mod.rs
    let mod_file = base.join("mod.rs");

    register_mod_file(&mod_file, &Str::snake(&factory_name, "_")).await?;

    operation(&format!("factory made: {:?}", factory_name),start.elapsed(),Status::Done);

    Ok(())
}