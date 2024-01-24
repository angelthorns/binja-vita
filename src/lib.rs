use std::error::Error;

use anyhow::anyhow;

use binaryninja::{
    binaryview::BinaryView,
    command::Command,
    interaction::{MessageBoxButtonSet, MessageBoxIcon},
};
use nids::NidsDB;

pub mod nids;

struct NidsImportCmd;

impl NidsImportCmd {
    fn action_body(&self, view: &BinaryView) -> Result<(), Box<dyn Error>> {
        let db = NidsDB::from(
            binaryninja::interaction::get_open_filename_input("Select nids db.yml to use", "*.yml")
                .ok_or(anyhow!("no db.yml path specified"))?
                .as_path(),
        )?;

        log::info!("{:#?}", db.all_functions);

        Ok(())
    }
}
impl Command for NidsImportCmd {
    fn action(&self, view: &BinaryView) {
        if let Err(e) = self.action_body(view) {
            binaryninja::interaction::show_message_box(
                "binja-vita",
                format!("Nids import failed: {}", e).as_str(),
                MessageBoxButtonSet::OKButtonSet,
                MessageBoxIcon::ErrorIcon,
            );
        }
    }

    fn valid(&self, _view: &BinaryView) -> bool {
        true
    }
}

fn main() {
    binaryninja::logger::init(log::LevelFilter::Info).unwrap();

    log::info!("binja-vita loaded");

    binaryninja::command::register(
        "Import vita nids",
        "import nids file db.yml",
        NidsImportCmd {},
    );
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn CorePluginInit() -> bool {
    main();
    true
}
