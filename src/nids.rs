use std::{collections::BTreeMap, error::Error, fs::File, io::BufReader, path::Path};

use marked_yaml::{types::MarkedMappingNode, Node};

use anyhow::anyhow;

pub type Nid = u32;
pub type NidName = String;

#[derive(Debug, Clone)]
pub struct VitaFunction {
    pub nid: Nid,
    pub name: String,
}
pub struct VitaLibrary {
    pub nid: Nid,
    pub name: String,
    pub functions: BTreeMap<Nid, VitaFunction>,
}

pub struct VitaModule {
    pub nid: Nid,
    pub name: String,
    pub libraries: BTreeMap<Nid, VitaLibrary>,
}

pub struct NidsDB {
    pub modules: BTreeMap<Nid, VitaModule>,
    pub all_functions: BTreeMap<Nid, VitaFunction>,
}

fn process_nid(node: Option<&Node>) -> Result<u32, Box<dyn Error>> {
    let nid_str = node
        .and_then(Node::as_scalar)
        .ok_or("nid field must be a scalar")?;

    if !nid_str.starts_with("0x") {
        return Err(anyhow!("nid value must start with 0x").into());
    }

    Ok(u32::from_str_radix(nid_str.split_at(2).1, 16)?)
}

impl VitaLibrary {
    pub fn from(
        nids_db: &mut NidsDB,
        key: &str,
        entry: &MarkedMappingNode,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            nid: process_nid(entry.get_node("nid"))?,
            name: key.to_string(),
            functions: {
                let mut funcs = BTreeMap::new();
                if let Err(e) = (|| -> Result<(), Box<dyn Error>> {
                    for (key, node) in entry
                        .get_node("functions")
                        .ok_or("no functions in vita library")?
                        .as_mapping()
                        .ok_or(anyhow!("functions must be a mapping node"))?
                        .iter()
                    {
                        let fnc = VitaFunction {
                            nid: process_nid(Some(node))?,
                            name: key.to_string(),
                        };

                        nids_db.add_function(fnc.clone());

                        funcs.insert(fnc.nid, fnc);
                    }
                    Ok(())
                })() {
                    log::warn!("{}", e.to_string());
                };

                funcs
            },
        })
    }
}
impl VitaModule {
    pub fn from(
        nids_db: &mut NidsDB,
        key: &str,
        entry: &MarkedMappingNode,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            nid: process_nid(entry.get_node("nid"))?,

            name: key.to_string(),
            libraries: {
                let mut libs = BTreeMap::new();

                for (key, node) in entry
                    .get_node("libraries")
                    .ok_or("no libraries node in vitamodule")?
                    .as_mapping()
                    .ok_or("libraries must be a mapping node")?
                    .iter()
                {
                    let lib = VitaLibrary::from(
                        nids_db,
                        key.as_str(),
                        node.as_mapping()
                            .ok_or(anyhow!("library node must be a mapping node"))?,
                    )?;

                    libs.insert(lib.nid, lib);
                }

                libs
            },
        })
    }
}
impl NidsDB {
    pub fn new() -> Self {
        Self {
            modules: Default::default(),
            all_functions: Default::default(),
        }
    }

    pub fn add_module(&mut self, module: VitaModule) {
        log::info!("added module ({}){}", module.nid, module.name);
        self.modules.insert(module.nid, module);
    }

    pub fn add_function(&mut self, function: VitaFunction) {
        log::info!("added function ({}){}", function.nid, function.name);
        self.all_functions.insert(function.nid, function);
    }

    pub fn from(path: &Path) -> Result<Self, Box<dyn Error>> {
        let yaml_data = std::io::read_to_string(BufReader::new(File::open(path)?))?;

        let yaml = marked_yaml::parse_yaml(0, yaml_data.as_str())?;
        let node = yaml
            .as_mapping()
            .ok_or(anyhow!("nids db root node is not a mapping node"))?;

        let mut nids_db = NidsDB::new();

        for module in node
            .get_node("modules")
            .ok_or("no ->modules node in nids db")?
            .as_mapping()
            .ok_or("modules node is not a mapping node")?
            .iter()
        {
            let key = module.0.to_string();
            let val = module.1;
            let module = VitaModule::from(
                &mut nids_db,
                key.as_str(),
                val.as_mapping()
                    .ok_or(anyhow!("module is not a mapping node"))?,
            )?;
            nids_db.add_module(module);
        }

        Ok(nids_db)
    }
}
