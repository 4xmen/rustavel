use minijinja::{ Environment};
use rustavel_core::facades::file_content::FileContent;
use crate::make::make_error::MakeError;
use rustavel_core::db::schema::ModelData;
const MODEL_GENERATED_TEMPLATE: &str = include_str!("../make/templates/model_generated.rs.j2");


#[derive(serde::Serialize)]
struct ModelContext {
    name: String,
    fields: String,
    table: String,
    pkey: String,
    field_list: String,
}

pub async fn save_generated_model(model_name: String, data: Option<ModelData>)  -> Result<(), MakeError>{

    let mut env = Environment::new();
    env.add_template("model_generated", MODEL_GENERATED_TEMPLATE)?;
    let base = std::env::current_dir()?.join("app/src/models");
    let generated_path = base.join(format!("{}_generated.rs", model_name.to_lowercase()));

    let ctx = if let Some(model_data) = data  {
        ModelContext {
            name: model_name,
            fields: model_data.model.trim().to_owned(),
            field_list: model_data.columns,
            table: model_data.table,
            pkey: model_data.primary_key,
        }
    }else{
        ModelContext {
            name: model_name,
            fields: "".to_string(),
            field_list: "".to_string(),
            table: "".to_string(),
            pkey: "".to_string(),
        }
    };

    let rendered = env.get_template("model_generated")?.render(ctx)?;

    // println!("{}", rendered); // debug print

    FileContent::put(generated_path.to_str().unwrap(), &rendered).await?;

    Ok(())
}