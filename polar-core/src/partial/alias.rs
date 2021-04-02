use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::Iterator;

use crate::terms::Symbol;

#[derive(Default)]
pub struct AliasSet {
    symbol_groups: HashMap<Symbol, usize>,
    groups: HashMap<usize, HashSet<Symbol>>,
    next_group_id: usize,
}

impl AliasSet {
    /// Add an alias for `symbol` to `alias` to the set.
    pub fn add(&mut self, symbol: Symbol, alias: Symbol) {
        match (
            self.symbol_groups.get(&symbol).cloned(),
            self.symbol_groups.get(&alias).cloned(),
        ) {
            (Some(symbol_group_id), Some(alias_group_id)) => {
                if symbol_group_id == alias_group_id {
                    // Groups are the same, nothing to do.
                    return;
                } else {
                    // Groups are not the same.
                    debug_assert!(!self.get_group(symbol_group_id).contains(&alias));
                    debug_assert!(!self.get_group(alias_group_id).contains(&symbol));

                    // Merge them by destroying alias group
                    // (TODO could make this a ptr swap?)
                    let alias_group = self.groups.remove(&alias_group_id).unwrap();

                    for item in alias_group.into_iter() {
                        self.symbol_groups.insert(item.clone(), symbol_group_id);
                        self.get_group_mut(symbol_group_id).insert(item);
                    }
                }
            }
            (Some(symbol_group_id), None) => {
                self.get_group_mut(symbol_group_id).insert(alias.clone());
                self.symbol_groups.insert(alias, symbol_group_id);
            }
            (None, Some(alias_group_id)) => {
                self.get_group_mut(alias_group_id).insert(alias);
                self.symbol_groups.insert(symbol, alias_group_id);
            }
            (None, None) => {
                let mut set = HashSet::new();
                set.insert(symbol.clone());
                set.insert(alias.clone());
                let group_id = self.new_group(set);
                self.symbol_groups.insert(symbol, group_id);
                self.symbol_groups.insert(alias, group_id);
            }
        }
    }

    /// Get alternative names for `symbol`.
    pub fn iter_aliases<'a>(
        &'a self,
        symbol: Symbol,
    ) -> Option<Box<dyn Iterator<Item = &'a Symbol> + 'a>> {
        self.symbol_groups.get(&symbol).cloned().map(|group_id| {
            Box::new(
                self.get_group(group_id)
                    .iter()
                    .filter(move |alias| alias != &&symbol),
            ) as Box<dyn Iterator<Item = &'a Symbol>>
        })
    }

    fn get_group_mut(&mut self, group_id: usize) -> &mut HashSet<Symbol> {
        self.groups.get_mut(&group_id).unwrap()
    }

    fn get_group(&self, group_id: usize) -> &HashSet<Symbol> {
        self.groups.get(&group_id).unwrap()
    }

    fn new_group(&mut self, set: HashSet<Symbol>) -> usize {
        let group_id = self.next_group_id;
        self.next_group_id += 1;
        self.groups.insert(group_id, set);
        group_id
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn aliases_set<'a>(set: &'a AliasSet, symbol: &Symbol) -> HashSet<&'a Symbol> {
        set.iter_aliases(symbol.clone()).unwrap().collect()
    }

    #[test]
    fn test_basic() {
        let mut set = AliasSet::default();

        let a = sym!("a");
        let a1 = sym!("a1");
        let a2 = sym!("a2");

        let b = sym!("b");
        let b1 = sym!("b1");
        let b2 = sym!("b2");

        set.add(a.clone(), a1.clone());

        assert_eq!(aliases_set(&set, &a), hashset!{&a1});
        assert_eq!(aliases_set(&set, &a1), hashset!{&a});

        set.add(a1.clone(), a2.clone());

        assert_eq!(aliases_set(&set, &a), hashset!{&a1, &a2});
        assert_eq!(aliases_set(&set, &a1), hashset!{&a, &a2});
        assert_eq!(aliases_set(&set, &a2), hashset!{&a, &a1});

        set.add(b.clone(), b1.clone());
        set.add(b.clone(), b1.clone());

        assert_eq!(aliases_set(&set, &a), hashset!{&a1, &a2});
        assert_eq!(aliases_set(&set, &a1), hashset!{&a, &a2});
        assert_eq!(aliases_set(&set, &a2), hashset!{&a, &a1});

        assert_eq!(aliases_set(&set, &b), hashset!{&b1});
        assert_eq!(aliases_set(&set, &b1), hashset!{&b});

        set.add(b.clone(), b2.clone());
        assert_eq!(aliases_set(&set, &b), hashset!{&b1, &b2});
        assert_eq!(aliases_set(&set, &b1), hashset!{&b, &b2});
        assert_eq!(aliases_set(&set, &b2), hashset!{&b, &b1});

        set.add(a1.clone(), b1.clone());
        assert_eq!(aliases_set(&set, &a), hashset!{&a1, &a2, &b, &b1, &b2});
        assert_eq!(aliases_set(&set, &a1), hashset!{&a, &a2, &b, &b1, &b2});
        assert_eq!(aliases_set(&set, &a2), hashset!{&a, &a1, &b, &b1, &b2});
        assert_eq!(aliases_set(&set, &b), hashset!{&b1, &b2, &a, &a1, &a2});
        assert_eq!(aliases_set(&set, &b1), hashset!{&b, &b2, &a, &a1, &a2});
        assert_eq!(aliases_set(&set, &b2), hashset!{&b, &b1, &a, &a1, &a2});
    }
}
