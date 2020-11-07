use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

use indexmap::IndexMap;

const OBJMASK_INFO: [(char, &str); 32] = [
    ('A', "Flag_A_OBJMASK_ARMORED"),
    ('B', "Flag_B_OBJMASK_BOMBARD"),
    ('C', "Flag_C_OBJMASK_CIVILIAN"),
    ('D', "Flag_D_OBJMASK_MUSKET_INF"),
    ('E', "Flag_E_OBJMASK_ELEPHANT"),
    ('F', "Flag_F_OBJMASK_FOOT"),
    ('G', "Flag_G_OBJMASK_GUN"),
    ('H', "Flag_H_OBJMASK_HEAVY_INF"),
    ('I', "Flag_I_OBJMASK_MODERN_INF"),
    ('J', "Flag_J_OBJMASK_CARRY_AIR"),
    ('K', "Flag_K_OBJMASK_FOOT_ARCHER"),
    ('L', "Flag_L_OBJMASK_LARGE"),
    ('M', "Flag_M_OBJMASK_MOUNTED"),
    ('N', "Flag_N_OBJMASK_NAVAL"),
    ('O', "Flag_O_OBJMASK_HORSE_ARCHER"),
    ('P', "Flag_P_OBJMASK_SPARSE"),
    ('Q', "Flag_Q_OBJMASK_LIGHT_INF"),
    ('R', "Flag_R_OBJMASK_ARCHERY"),
    ('S', "Flag_S_OBJMASK_SIEGE"),
    ('T', "Flag_T_OBJMASK_WAR_MACHINE"),
    ('U', "Flag_U_OBJMASK_ARMORPIERCE"),
    ('V', "Flag_V_OBJMASK_VEHICLE"),
    ('W', "Flag_W_OBJMASK_MELEE"),
    ('X', "Flag_X_OBJMASK_EXPLOSIVE"),
    ('Y', "Flag_Y_OBJMASK_HEAVY_CAV"),
    ('Z', "Flag_Z_OBJMASK_DETECT"),
    ('1', "Flag_1_OBJMASK_UNUSED"),
    ('2', "Flag_2_OBJMASK_MISSILE"),
    ('3', "Flag_3_OBJMASK_AIR"),
    ('4', "Flag_4_OBJMASK_LIGHT_CAV"),
    ('5', "Flag_5_OBJMASK_PIKE"),
    ('6', "Flag_6_OBJMASK_ANTI_AIR"),
];

fn char_to_attrib_str(c: char) -> Option<&'static str> {
    OBJMASK_INFO.iter().find(|(c2, _)| c2 == &c).map(|(_, attrib)| *attrib)
}

#[derive(Clone, Debug, Default)]
struct UnitBalance {
    entries: IndexMap<String, UnitBalanceEntry>,
}

#[derive(Clone, Debug, Default)]
struct UnitBalanceEntry {
    modifiers: IndexMap<String, f32>,
}

fn main() {
    let ron_dir = std::env::args().nth(1);

    if ron_dir.is_none() {
        print_usage();
        return;
    }

    let ron_dir = ron_dir.unwrap();

    if ron_dir == "-h" || ron_dir == "--help" {
        print_usage();
        return;
    }

    match run(&ron_dir) {
        Ok(_) => {
            println!("Complete");
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

fn print_usage() {
    println!("Rise of Nations: Extended Edition OBJ_MASK bug workaround");
    println!("");
    println!("USAGE:");
    println!("    ron-objmask-workaround [ron data directory]");
    println!("");
    println!("OPTION:");
    println!("    -h, --help  Print this help information");
}

fn run(ron_dir: &str) -> Result<(), String> {
    let unit_objmask_map = parse_unitrules(&ron_dir)?;
    let old_unit_balance = parse_balance(&ron_dir)?;
    let new_unit_balance = calculate_new_balance(&unit_objmask_map, &old_unit_balance);

    write_new_balance(&ron_dir, &new_unit_balance)
        .map_err(|e| format!("Failed to write new balance.xml file: {}", e))?;

    Ok(())
}

fn parse_unitrules(ron_dir: &str) -> Result<IndexMap<String, HashSet<&'static str>>, String> {
    let unitrules_xml_file = File::open(PathBuf::from(ron_dir).join("unitrules.xml"))
        .map_err(|e| format!("Failed to open unitrules.xml: {}", e))?;
    let unitrules_xml_reader = BufReader::new(unitrules_xml_file);

    let mut unitrules_xml_document = Reader::from_reader(unitrules_xml_reader);

    println!("Processing unitrules.xml");

    let mut unit_objmask_map: IndexMap<String, HashSet<&'static str>> = IndexMap::new();

    let mut buf = Vec::new();
    let mut in_unit_element = false;
    let mut in_name_element = false;
    let mut in_obj_mask_element = false;
    let mut cur_unit_name = String::new();
    let mut cur_obj_mask = String::new();
    loop {
        let event = unitrules_xml_document.read_event(&mut buf)
            .map_err(|e| format!("Failed to read unitrules.xml: {}", e))?;
        match event {
            Event::Start(e) if e.name() == b"UNIT" => {
                in_unit_element = true;
            }
            Event::Start(e) if e.name() == b"NAME" && in_unit_element => {
                in_name_element = true;
            }
            Event::Start(e) if e.name() == b"OBJ_MASK" && in_unit_element => {
                in_obj_mask_element = true;
            }
            Event::Text(e) if in_name_element => {
                cur_unit_name = e.unescape_and_decode(&unitrules_xml_document)
                    .map_err(|e| format!("Failed to get unit name: {}", e))?;
                cur_unit_name = cur_unit_name.replace(" ", "_").replace("'", "");
            }
            Event::Text(e) if in_obj_mask_element => {
                cur_obj_mask = e.unescape_and_decode(&unitrules_xml_document)
                .map_err(|e| format!("Failed to get unit obj_mask: {}", e))?;
            }
            Event::End(e) if e.name() == b"NAME" && in_unit_element => {
                in_name_element = false;
            }
            Event::End(e) if e.name() == b"OBJ_MASK" && in_unit_element => {
                in_obj_mask_element = false;
            }
            Event::End(e) if e.name() == b"UNIT" => {
                in_unit_element = false;

                let mut obj_masks = HashSet::<&'static str>::new();
                for c in cur_obj_mask.chars() {
                    if let Some(name) = char_to_attrib_str(c) {
                        obj_masks.insert(name);
                    } else {
                        println!("Warning: unknown OBJ_MASK flag found '{}'", c);
                    }
                }

                use indexmap::map::Entry;
                match unit_objmask_map.entry(cur_unit_name.clone()) {
                    Entry::Vacant(v) => {
                        v.insert(obj_masks);
                    }
                    Entry::Occupied(mut o) => {
                        if *o.get() != obj_masks {
                            println!("Warning: different units with identical names have differing OBJ_MASK values");
                        }
                        o.get_mut().extend(obj_masks.iter());
                    }
                }

                cur_unit_name.clear();
                cur_obj_mask.clear();
            }
            Event::Eof => break,
            _ => (),
        }
    }

    // Add some additional "meta" entries.
    unit_objmask_map.insert("SIEGE".to_owned(), Default::default());
    unit_objmask_map.insert("FORTS".to_owned(), Default::default());
    unit_objmask_map.insert("TOWERS".to_owned(), Default::default());
    unit_objmask_map.insert("CITIES".to_owned(), Default::default());
    unit_objmask_map.insert("OBSPOST".to_owned(), Default::default());
    unit_objmask_map.insert("BUILDINGS".to_owned(), Default::default());
    unit_objmask_map.insert("UNITS".to_owned(), Default::default());
    unit_objmask_map.insert("AGE_0".to_owned(), Default::default());
    unit_objmask_map.insert("AGE_1".to_owned(), Default::default());
    unit_objmask_map.insert("AGE_2".to_owned(), Default::default());
    unit_objmask_map.insert("AGE_3".to_owned(), Default::default());
    unit_objmask_map.insert("AGE_4".to_owned(), Default::default());
    unit_objmask_map.insert("AGE_5".to_owned(), Default::default());
    unit_objmask_map.insert("AGE_6".to_owned(), Default::default());
    unit_objmask_map.insert("AGE_7".to_owned(), Default::default());

    Ok(unit_objmask_map)
}

fn parse_balance(ron_dir: &str) -> Result<UnitBalance, String> {
    let balance_xml_file = File::open(PathBuf::from(ron_dir).join("balance.xml"))
        .map_err(|e| format!("Failed to open balance.xml: {}", e))?;
    let balance_xml_reader = BufReader::new(balance_xml_file);

    let mut balance_xml_document = Reader::from_reader(balance_xml_reader);

    println!("Processing balance.xml");

    let mut old_unit_balance = UnitBalance::default();

    let mut buf = Vec::new();
    loop {
        let event = balance_xml_document.read_event(&mut buf)
            .map_err(|e| format!("Failed to read balance.xml: {}", e))?;
        match event {
            Event::Start(e) | Event::Empty(e) if e.name() == b"ENTRY" => {
                let mut name = String::new();
                let mut modifiers = IndexMap::new();
                for attrib in e.attributes() {
                    let attrib = attrib
                        .map_err(|e| format!("Failed to get attribute in a balance ENTRY element: {}", e))?;
                    if attrib.key == b"name" {
                        name = attrib.unescape_and_decode_value(&balance_xml_document)
                            .map_err(|e| format!("Failed to get balance ENTRY element name: {}", e))?;
                    } else {
                        let key = balance_xml_document.decode(attrib.key)
                            .map_err(|e| format!("Failed to get attribute key in a balance ENTRY element: {}", e))?
                            .to_owned();

                        let value = attrib.unescaped_value()
                            .map_err(|e| format!("Failed to get attribute value in a balance ENTRY element: {}", e))?;

                        let value = balance_xml_document.decode(&value)
                            .map_err(|e| format!("Failed to get attribute value in a balance ENTRY element: {}", e))?;

                        let value = value.parse::<f32>()
                            .map_err(|e| format!("Failed to parse attribute value in a balance ENTRY element: {}", e))?;

                        modifiers.insert(key, value);
                    }
                }

                if name.is_empty() {
                    return Err("No \"name\" attribute found in a balance ENTRY element".to_owned());
                }

                old_unit_balance.entries.insert(name, UnitBalanceEntry { modifiers });
            }
            Event::Eof => break,
            _ => (),
        }

        buf.clear();
    }

    Ok(old_unit_balance)
}

fn calculate_new_balance(unit_objmask_map: &IndexMap<String, HashSet<&'static str>>,
                         old_unit_balance: &UnitBalance) -> UnitBalance {
    let mut new_unit_balance = old_unit_balance.clone();

    // Calculate the matrix of all unit balancing modifiers.
    for (unit_a, unit_a_objmask) in unit_objmask_map.iter() {
        let new_entry = new_unit_balance.entries.entry(unit_a.clone()).or_default();
        for (unit_b, unit_b_objmask) in unit_objmask_map.iter() {
            let mut balance = 100.0;

            // Iterate over unit name and object mask names for unit A.
            let unit_a_names_iter = std::iter::once(unit_a.as_str())
                .chain(unit_a_objmask.iter().cloned());
            for entry_name in unit_a_names_iter {
                let entry = old_unit_balance.entries.get(entry_name);
                // Iterate over unit name and object mask names for
                // unit B.
                let unit_b_names_iter = std::iter::once(unit_b.as_str())
                    .chain(unit_b_objmask.iter().cloned());
                for attrib_name in unit_b_names_iter {
                    // Get the modifier if it exists, otherwise assume 100.
                    let modifier = *entry
                        .and_then(|entry| entry.modifiers.get(attrib_name))
                        .unwrap_or(&100.0);

                    balance *= modifier / 100.0;
                }
            }

            new_entry.modifiers.insert(unit_b.clone(), balance);
        }
    }

    // Reset objmask scaling to 100, not strictly necessary since they
    // are bugged, but might as well do it for correctness sake.
    for (_, entry) in &mut new_unit_balance.entries {
        for (_, objmask_name) in OBJMASK_INFO.iter() {
            entry.modifiers.get_mut(*objmask_name).map(|modifier| *modifier = 100.0);
        }
    }

    for (_, objmask_name) in OBJMASK_INFO.iter() {
        let mut entry = new_unit_balance.entries.get_mut(*objmask_name);
        for (_, modifier) in entry.iter_mut().flat_map(|v| v.modifiers.iter_mut()) {
            *modifier = 100.0;
        }
    }

    new_unit_balance
}

fn write_new_balance(ron_dir: &str, new_unit_balance: &UnitBalance) -> Result<(), quick_xml::Error> {
    let balance_xml_file = File::create(ron_dir.to_owned() + "/balance_out.xml")?;
    let balance_xml_writer = BufWriter::new(balance_xml_file);

    let mut balance_xml_out = Writer::new_with_indent(balance_xml_writer, b' ', 2);

    println!("Writing new balance.xml");

    balance_xml_out.write_event(Event::Decl(BytesDecl::new(b"1.0", None, None)))?;

    balance_xml_out.write_event(Event::Start(BytesStart::borrowed(b"ROOT", b"ROOT".len())))?;

    balance_xml_out.write_event(Event::Start(BytesStart::borrowed(b"TABLE", b"TABLE".len())))?;

    for (entry_name, entry) in &new_unit_balance.entries {
        let mut entry_elem = BytesStart::owned(b"ENTRY".to_vec(), b"ENTRY".len());

        entry_elem.push_attribute(("name", entry_name.as_str()));
        for (modifier_name, modifier) in &entry.modifiers {
            let modifier_str = (modifier.round() as i32).to_string();
            entry_elem.push_attribute((modifier_name.as_str(), modifier_str.as_str()));
        }

        balance_xml_out.write_event(Event::Empty(entry_elem))?;
    }

    balance_xml_out.write_event(Event::End(BytesEnd::borrowed(b"TABLE")))?;

    balance_xml_out.write_event(Event::End(BytesEnd::borrowed(b"ROOT")))?;

    Ok(())
}