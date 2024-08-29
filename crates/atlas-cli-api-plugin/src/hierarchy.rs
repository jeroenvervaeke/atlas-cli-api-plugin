use std::collections::{BTreeMap, BTreeSet};

use anyhow::{bail, Result};
use openapiv3::OpenAPI;
use path::Path;
use pluralizer::pluralize;
use serde::Serialize;

mod cli;
mod path;

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct Hierarchy {
    entries: BTreeMap<String, HierarchyEntry>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct HierarchyEntry {
    entity_name: Option<String>,
    entries: BTreeMap<String, HierarchyEntry>,
    verbs: BTreeMap<String, String>,
}

const KNOWN_VERBS: &[&str] = &[
    "add", "create", "delete", "get", "list", "update", "upgrade", "verify",
];

impl Hierarchy {
    pub fn from_openapi_spec(prefix: &str, spec: &OpenAPI) -> Result<Self> {
        let mut operation_ids = BTreeSet::<&str>::new();
        let mut entities = BTreeSet::<String>::new();
        let mut verbs = BTreeSet::<&str>::new();
        let mut root = RawEntry::default();

        for verb in KNOWN_VERBS {
            verbs.insert(verb);
        }

        for (path, reference_or_item) in spec.paths.paths.iter() {
            let path = match path.strip_prefix(prefix).and_then(Path::from_str) {
                Some(path) => path,
                None => continue,
            };

            let mut entry = &mut root;
            for segment in path.consts() {
                entry = entry.children.entry(segment.to_string()).or_default();
            }

            if let Some(item) = reference_or_item.as_item() {
                for (_http_verb, operation) in item.iter() {
                    if let Some(operation_id) = operation.operation_id.as_deref() {
                        entry.operation_ids.insert(operation_id.to_string());
                        operation_ids.insert(operation_id);
                    }
                }
            }
        }

        // discover all entities
        for operation_id in &operation_ids {
            for verb in &verbs {
                if let Some(entity) = operation_id.strip_prefix(verb) {
                    let singular_entity = pluralize(entity, 1, false);
                    entities.insert(singular_entity);
                }
            }
        }

        // discover remaining verbs
        for operation_id in &operation_ids {
            let mut shortest_verb: Option<&str> = None;
            for entity in &entities {
                if let Some(verb) = operation_id.strip_suffix(entity) {
                    let is_shorter = match shortest_verb {
                        Some(shortest) => shortest.len() > verb.len(),
                        None => true,
                    };

                    if is_shorter {
                        shortest_verb = Some(verb);
                    }
                }
            }

            if let Some(verb) = shortest_verb {
                verbs.insert(verb);
            }
        }

        /* println!("{entities:#?}");
        println!("{verbs:#?}");
        println!("{root:#?}"); */

        let mut entries = BTreeMap::new();
        for (name, raw_entry) in root.children {
            if let Some(entry) = raw_entry.into_hierarchy_entry(&verbs)? {
                entries.insert(name, entry);
            }
        }

        Ok(Hierarchy { entries })
    }
}

#[derive(Default, Debug)]
struct RawEntry {
    children: BTreeMap<String, RawEntry>,
    operation_ids: BTreeSet<String>,
}

impl RawEntry {
    fn into_hierarchy_entry(self, verbs: &BTreeSet<&str>) -> Result<Option<HierarchyEntry>> {
        self.into_hierarchy_entry_inner(&Default::default(), verbs)
    }

    fn into_hierarchy_entry_inner(
        self,
        prefix: &Vec<String>,
        verbs: &BTreeSet<&str>,
    ) -> Result<Option<HierarchyEntry>> {
        let mut entities = BTreeSet::<String>::new();
        let mut operation_ids = BTreeMap::<String, String>::new();

        for operation_id in &self.operation_ids {
            for verb in verbs {
                if let Some(entity) = operation_id.strip_prefix(verb) {
                    entities.insert(entity.to_string());
                    operation_ids.insert(verb.to_string(), operation_id.to_owned());
                }
            }
        }

        let entity_name = entities.first().map(|e| {
            let mut entity_name = e.to_owned();
            for p in prefix {
                if let Some(r) = entity_name.strip_prefix(p) {
                    entity_name = r.to_string();
                }
            }

            entity_name
        });
        if self.children.is_empty() && self.operation_ids.is_empty() {
            return Ok(None);
        }

        let mut new_prefix = prefix.clone();
        if let Some(e) = entity_name.as_ref() {
            new_prefix.push(e.to_owned());
        }

        let mut entries = BTreeMap::<String, HierarchyEntry>::new();
        for (name, entry) in self.children {
            if let Some(e) = entry.into_hierarchy_entry_inner(&new_prefix, verbs)? {
                if let Some(existing_entry) = entries.values_mut().find(|x| x.entity_name == e.entity_name) {
                    existing_entry.merge(e)?;
                }
                else {
                    entries.insert(name, e);
                }
            }
        }

        Ok(Some(HierarchyEntry {
            entity_name,
            entries,
            verbs: operation_ids,
        }))
    }
}

impl HierarchyEntry {
    pub fn merge(&mut self, other: Self) -> Result<()> {
        for (verb, operation_id) in other.verbs {
            if self.verbs.insert(verb, operation_id).is_some() {
                bail!("operation already exists");
            }
        }

        for (slug, entity) in other.entries {
            if let Some(existing) = self.entries.get_mut(&slug) {
                existing.merge(entity)?;
            } else {
                self.entries.insert(slug.to_owned(), entity);
            }
        }

        Ok(())
    }
}
