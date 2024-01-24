use std::{error::Error, mem::size_of};

use anyhow::anyhow;

use binaryninja::{
    binaryview::{BinaryView, BinaryViewBase, BinaryViewExt},
    command::Command,
    interaction::{MessageBoxButtonSet, MessageBoxIcon},
    symbol::{SymbolBuilder, SymbolType},
};
use nids::{Nid, NidsDB};

pub mod nids;

/* documentation: https://www.psdevwiki.com/vita/index.php?title=PRX_File_Format#Imports */
#[repr(C)]
#[derive(Debug)]
pub struct LibraryStubTable {
    pub ssize: u8,
    pub _1: u8,
    pub ver: u16,
    pub attr: u16,
    pub func_count: u16,
    pub var_count: u16,
    pub tlsvar_count: u16,
    pub _2: u32,

    //prx2arm
    pub libname_nid: Nid,
    pub libname: u32,
    pub sdk_ver: u32,
    pub func_nidtable: u32,
    pub func_table: u32,
    pub var_nidtable: u32,
    pub var_table: u32,
    pub tls_nidtable: u32,
    pub tls_table: u32,
}
struct NidsImportCmd;

impl NidsImportCmd {
    fn action_body(&self, view: &BinaryView) -> Result<(), Box<dyn Error>> {
        let db = NidsDB::from(
            binaryninja::interaction::get_open_filename_input("Select nids db.yml to use", "*.yml")
                .ok_or(anyhow!("no db.yml path specified"))?
                .as_path(),
        )?;

        /* documentation: https://www.psdevwiki.com/vita/index.php?title=PRX_File_Format */
        let import_stub_table_loc = view.start() + view.entry_point() + 44;
        log::info!("reading stub from {:#10x}", import_stub_table_loc);

        let mut data: [u8; 8] = [0; 8]; // we want to get both the start and the end
        view.read(&mut data, import_stub_table_loc);

        let mut stub_start = view.start() + u32::from_le_bytes(data[0..4].try_into()?) as u64;
        let stub_end = view.start() + u32::from_le_bytes(data[4..8].try_into()?) as u64;

        log::info!(
            "import table beginning and end found at {:#10x} to {:#10x}",
            stub_start,
            stub_end
        );

        loop {
            let mut table = [0u8; size_of::<LibraryStubTable>()];
            view.read(&mut table, stub_start);
            let tb: LibraryStubTable = unsafe { std::mem::transmute(table) };

            log::debug!("stub table {:#?}", tb);
            stub_start = stub_start + (tb.ssize as u64);

            let func_nidtable = tb.func_nidtable as u64;
            let func_table = tb.func_table as u64;

            log::debug!(
                "nid table: {:#10x}, table: {:#10x}",
                func_nidtable,
                func_table
            );

            for i in 0..tb.func_count {
                let nid = u32::from_le_bytes(
                    view.read_vec(func_nidtable + ((i as u64) * 4), 4)[0..4].try_into()?,
                );

                let addr = u32::from_le_bytes(
                    view.read_vec(func_table + ((i as u64) * 4), 4)[0..4].try_into()?,
                );

                let name = db
                    .all_functions
                    .get(&nid)
                    .map(|v| v.name.clone())
                    .unwrap_or(format!("nid_{}", nid));

                log::info!("located nid symbol {} at {:#10x}", name, addr);
                view.define_auto_symbol(
                    &(SymbolBuilder::new(SymbolType::LibraryFunction, name.as_str(), addr as u64)
                        .create()),
                );
            }

            if stub_start >= stub_end {
                break;
            }
        }
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
