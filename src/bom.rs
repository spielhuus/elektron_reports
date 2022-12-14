use std::collections::HashMap;
use elektron_sexp::{SchemaElement, Schema};
use crate::error::Error;

#[derive(Debug, Clone)]
pub struct BomItem {
    pub amount: usize,
    pub references: Vec<String>,
    pub value: String,
    pub footprint: String,
    pub datasheet: String,
    pub description: String,
}

fn reference(value: &str) -> String {
    let mut reference_characters = String::new();
    let mut reference_numbers = String::new();
    for c in value.chars() {
        if c.is_numeric() {
            reference_numbers.push(c);
        } else {
            reference_characters.push(c);
        }
    }
    format!("{}{:0>4}", reference_characters, reference_numbers)
}

pub fn bom(document: &Schema, group: bool) -> Result<Vec<BomItem>, Error> {
    let mut bom_items: Vec<BomItem> = Vec::new();
    for item in document.iter_all() {
        if let SchemaElement::Symbol(symbol) = item {
            if symbol.unit == 1
                && !symbol.lib_id.starts_with("power:")
                && !symbol.lib_id.starts_with("Mechanical:")
            {
                bom_items.push(BomItem {
                    amount: 1,
                    references: vec![symbol.get_property("Reference").unwrap()],
                    value: symbol.get_property("Value").unwrap(),
                    footprint: symbol.get_property("Footprint").unwrap(),
                    datasheet: symbol.get_property("Datasheet").unwrap(),
                    description: if let Some(description) = symbol.get_property("Description") {
                        description
                    } else {
                        String::new()
                    },
                });
            }
        }
    }

    if group {
        let mut map: HashMap<String, Vec<&BomItem>> = HashMap::new();
        for item in &bom_items {
            let key = format!("{}:{}", item.value, item.footprint);
            map.entry(key).or_insert(Vec::new()).push(item);
        }
        bom_items = map
            .iter()
            .map(|(_, value)| {
                let mut refs: Vec<String> = Vec::new();
                for v in value {
                    refs.push(v.references.get(0).unwrap().to_string());
                }
                BomItem {
                    amount: value.len(),
                    references: refs,
                    value: value[0].value.to_string(),
                    footprint: value[0].footprint.to_string(),
                    datasheet: value[0].datasheet.to_string(),
                    description: value[0].description.to_string(),
                }
            })
            .collect();
    }

    bom_items.sort_by(|a, b| {
        let ref_a = reference(&a.references[0]);
        let ref_b = reference(&b.references[0]);
        ref_a.partial_cmp(&ref_b).unwrap()
    });

    Ok(bom_items)
}

#[cfg(test)]
mod tests {
    use super::bom;
    use elektron_sexp::Schema;

    #[test]
    fn test_bom() {
        let schema = Schema::load("files/summe/summe.kicad_sch").unwrap();
        let result = bom(&schema, true).unwrap();
        assert_eq!(4, result.len());
    }
}
