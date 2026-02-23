
use crate::migrator::Migration;
pub mod m_2025_01_15_1945_create_users;
// #[placeholder-add-mig-mods] DO NOT REMOVE THIS COMMENT, OTHERWISE AUTOMATIC ADD WILL BREAK

pub fn get_all_migrations() -> Vec<Box<dyn Migration>> {
    // may need do it auto next time
    vec![
        Box::new(m_2025_01_15_1945_create_users::CreateUsers {}),
        // #[placeholder-add-mig-trait] DO NOT REMOVE THIS COMMENT, OTHERWISE AUTOMATIC ADD WILL BREAK
    ]
}
