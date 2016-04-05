use std::collections::HashMap;
use std::io::prelude::*;
use std::fs::File;
use yaml_rust::{YamlLoader, Yaml};

pub type Kwargs = HashMap<String, String>;
pub type Inventory = HashMap<String, Vec<String>>;

#[derive(Debug)]
pub struct Property {
    pub name: String,
    pub module: String,
    pub params: HashMap<String, String>
}

#[derive(Debug)]
pub struct Check {
    pub inventory_name: String,
    pub properties: Vec<Property>
}


pub struct CheckSuite {
    pub inventory: Inventory,
    pub checks: Vec<Check>,
}

impl CheckSuite {

    pub fn read_from_file(filename: &str) -> Option<CheckSuite> {
        let INVENTORY = Yaml::from_str("inventory");
        let HOSTS = Yaml::from_str("hosts");
        let PROPERTIES = Yaml::from_str("properties");
        let NAME = Yaml::from_str("name");

        let mut f = File::open(filename).unwrap();
        let mut yaml_str = String::new();
        let _ = f.read_to_string(&mut yaml_str);
        let docs = YamlLoader::load_from_str(&yaml_str).unwrap();

    /*
    [Array([
           Hash({
               String("inventory"): Hash({
                   String("raspi"): Array([
                        String("heimbot.fritz.box")])})}),
           Hash({
               String("hosts"): String("raspi"),
               String("properties"): Array([
                 Hash({
                     String("name"): String("Checking SSH"),
                     String("ssh"): Hash({
                         String("port"): Integer(22),
                         String("software"): String("OpenSSH.*"),
                         String("version"): Real("2.0")})})])})])]
    */

        // We assume only one document in the file which consists of a list of hashes
        let mut inventory = Inventory::new();
        let mut checks: Vec<Check> = Vec::new();

        for hash in docs[0].as_vec().unwrap() {
            let map = hash.as_hash().unwrap();
            if map.contains_key(&INVENTORY) {
                debug!("Found inventory: {:?}", hash);
                let inventory_yaml = map.get(&INVENTORY).unwrap().as_hash().unwrap();
                for hosts_name_yaml in inventory_yaml.keys() {
                    let hosts_name = hosts_name_yaml.as_str().unwrap().to_string();
                    let mut hosts: Vec<String> = Vec::new();
                    for host in inventory_yaml.get(hosts_name_yaml).unwrap().as_vec().unwrap() {
                        let h = host.as_str().unwrap().to_string();
                        debug!("Host: {}", h);
                        hosts.push(h);
                    }
                    debug!("- - Inventory name: '{:?}'", hosts_name);
                    debug!("- - Inventory hosts: '{:?}'", hosts);
                    inventory.insert(hosts_name.to_string(), hosts);
                }
            }
        }

        for hash in docs[0].as_vec().unwrap() {
            let map = hash.as_hash().unwrap();
            if map.contains_key(&INVENTORY) {
            } else {
                if map.contains_key(&HOSTS) && map.contains_key(&PROPERTIES) {
                    debug!("- Found check: {:?}", hash);
                    let inventory_name = map.get(&HOSTS).unwrap().as_str().unwrap();
                    let mut properties = Vec::new();

                    let properties_yaml = map.get(&PROPERTIES).unwrap().as_vec().unwrap();
                    for property_yml in properties_yaml {
                        let mut name: Option<String> = None;
                        let mut params = HashMap::new();
                        let mut module: Option<String> = None;
                        for elem in property_yml.as_hash().unwrap() {
                            if elem.0 == &NAME {
                                name = Some(elem.1.as_str().unwrap().to_string());
                            } else {
                                module = Some(elem.0.as_str().unwrap().to_string());
                            }
                        }
                        if module.is_some() {
                            let params_yaml = property_yml.as_hash().unwrap().get(&Yaml::from_str(module.as_ref().unwrap())).unwrap().as_hash().unwrap();
                            for kv in params_yaml {
                                let value: String = match *kv.1 {
                                    Yaml::Integer(i) => i.to_string(),
                                    Yaml::Real(ref r) => r.to_string(),
                                    Yaml::String(ref string) => string.to_string(),
                                    _ => "<could not translate YAML value>".to_string(),
                                };
                                params.insert(kv.0.as_str().unwrap().to_string(), value);
                            }
                        }
                        properties.push( Property { name: name.unwrap(), module: module.unwrap(), params: params } );
                    }

                    let check = Check { inventory_name: inventory_name.to_string(), properties: properties };
                    debug!("- Created a check: {:?}", check);
                    checks.push(check);
                }

            }
        }
        info!("* Inventory: {:?}", inventory);
        info!("* Checks: {:?}", checks);

        let suite = CheckSuite { inventory: inventory, checks: checks };
        return Some(suite);
    }
}

