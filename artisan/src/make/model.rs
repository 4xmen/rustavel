use clap::Args;
use illuminate_string::Str;
use minijinja::{context, Environment};
use rustavel_core::facades::file_content::FileContent;
use crate::make::make_error::MakeError;
use crate::make::migration::{migrate, NewMigArgs};

const MODEL_GENERATED_TEMPLATE: &str = include_str!("templates/model_generated.rs.j2");
const MODEL_TEMPLATE: &str = include_str!("templates/model.rs.j2");
#[derive(Args, Debug)]
#[command(about = "Create a new model file")]
pub struct NewModelArgs {
    /// The name of the model
    pub name: String,
    /// has migration
    #[arg(short = 'm')]
    pub has_migration: bool,

    /// has controller
    #[arg(short = 'c')]
    pub has_controller: bool,
}


#[derive(serde::Serialize)]
struct ModelContext {
    name: String,
    fields: String,
    table: String,
    pkey: String,
    field_list: String,
}


pub async fn model(args: &NewModelArgs) -> Result<(), MakeError> {
    let model_name = Str::ucfirst( &Str::singular(&args.name) );

    let mut env = Environment::new();
    env.add_template("model_generated", MODEL_GENERATED_TEMPLATE)?;

    let ctx = ModelContext {
        name: model_name.clone(),
        fields: "".to_string(),
        field_list: "".to_string(),
        table: "".to_string(),
        pkey: "".to_string(),
    };

    let rendered = env.get_template("model_generated")?.render(ctx)?;


    let base = std::env::current_dir()?.join("app/src/models");
    let generated_path = base.join(format!("{}_generated.rs", model_name.to_lowercase()));
    let model_path = base.join(format!("{}.rs", model_name.to_lowercase()));
    FileContent::put(generated_path.to_str().unwrap(), &rendered).await?;
    env = Environment::new();
    env.add_template("model_generated", MODEL_TEMPLATE)?;
    let rendered = env.get_template("model_generated")?.render(context! {
        name => model_name.clone(),
    })?;
    FileContent::put(model_path.to_str().unwrap(), &rendered).await?;


    if args.has_migration {
        let migration_name = format!("{}Create", &model_name);
        let table = Str::plural_studly(&model_name,3).to_lowercase();
        println!("Creating migration {}", table);
        let mig_args = NewMigArgs{
            name: migration_name,
            create: Some(table),
            path: None,
            table: None,
            realpath: false,
        };
        _ = migrate(&mig_args).await?;
    }

    // WIP: create controller
    if args.has_migration {
        // create controller here
    }
    Ok(())
}