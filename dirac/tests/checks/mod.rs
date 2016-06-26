use dirac::checks::*;

static CHECK_SUITE_YAML: &'static str = r##"
---
- inventory:
    all:
      - fritz.box
      - esel.fritz.box
    fritz_box:
      - fritz.box
    esel:
      - esel.fritz.box


- hosts: all
  properties:
    - name: Check SSH
      ssh: { port: 22, version: 2.0, software: "OpenSSH.*" }


- hosts: fritz_box
  properties:
    - name: DNS TCP
      connect_tcp:
        port: 53


- hosts: esel
  properties:
    - name: NetBios TCP Port
      connect_tcp:
        port: 139
"##;

#[test]
pub fn check_suite_yml_read_in_test() {
    parse_check_suite(CHECK_SUITE_YAML);
}

#[test]
pub fn check_suite_yml_inventory_test() {
    let check_suite = parse_check_suite(CHECK_SUITE_YAML);
    let inventory = &check_suite.inventory;

    assert!(inventory.contains_key("all"));
    assert!(inventory.contains_key("fritz_box"));
    assert!(inventory.contains_key("esel"));
    assert_eq!(inventory.len(), 3);

    let all_group = inventory.get("all").unwrap();
    assert!(all_group.contains(&"fritz.box".to_string()));
    assert!(all_group.contains(&"esel.fritz.box".to_string()));
    assert_eq!(all_group.len(), 2);

    let fritz_box_group = inventory.get("fritz_box").unwrap();
    assert!(fritz_box_group.contains(&"fritz.box".to_string()));
    assert_eq!(fritz_box_group.len(), 1);

    let esel_group = inventory.get("esel").unwrap();
    assert!(esel_group.contains(&"esel.fritz.box".to_string()));
    assert_eq!(esel_group.len(), 1);
}

#[test]
pub fn check_suite_yml_checks_test() {
    let check_suite = parse_check_suite(CHECK_SUITE_YAML);
    let checks = &check_suite.checks;
    assert_eq!(checks.len(), 3);

    {
        let check = &checks[0];
        assert_eq!(check.inventory_name, "all".to_string());

        let properties = &check.properties;
        assert_eq!(properties.len(), 1);

        let property = &properties[0];
        assert_eq!(property.name, "Check SSH".to_string());
        assert_eq!(property.module, "ssh".to_string());

        let prop_params = &property.params;
        assert_eq!(prop_params["port"], "22".to_string());
        assert_eq!(prop_params["version"], "2.0".to_string());
        assert_eq!(prop_params["software"], "OpenSSH.*".to_string());
        assert_eq!(prop_params.len(), 3);
    }

    {
        let check = &checks[1];
        assert_eq!(check.inventory_name, "fritz_box".to_string());
        let properties = &check.properties;
        assert_eq!(properties.len(), 1);

        let property = &properties[0];
        assert_eq!(property.name, "DNS TCP".to_string());
        assert_eq!(property.module, "connect_tcp".to_string());

        let prop_params = &property.params;
        assert_eq!(prop_params["port"], "53".to_string());
        assert_eq!(prop_params.len(), 1);
    }

    {
        let check = &checks[2];
        assert_eq!(check.inventory_name, "esel".to_string());
        let properties = &check.properties;
        assert_eq!(properties.len(), 1);

        let property = &properties[0];
        assert_eq!(property.name, "NetBios TCP Port".to_string());
        assert_eq!(property.module, "connect_tcp".to_string());

        let prop_params = &property.params;
        assert_eq!(prop_params["port"], "139".to_string());
        assert_eq!(prop_params.len(), 1);
    }
}

fn parse_check_suite(check_suite_yaml: &str) -> CheckSuite {
    let check_suite_op = CheckSuite::read_from_string(check_suite_yaml);
    assert!(check_suite_op.is_some());
    check_suite_op.unwrap()
}
