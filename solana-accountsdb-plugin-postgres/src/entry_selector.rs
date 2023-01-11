#[derive(Debug)]
pub(crate) struct EntrySelector {
    pub select_all_entry: bool,
}

impl EntrySelector {
    pub fn default() -> Self {
        Self {
            select_all_entry: false,
        }
    }

    pub fn new(enable: bool) -> Self {
        Self {
            select_all_entry: enable,
        }
    }

    /// Check if interest in shred
    pub fn is_enabled(&self) -> bool {
        self.select_all_entry
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn test_default_selector() {
        let entry_selector = EntrySelector::default();
        assert_eq!(false, entry_selector.is_enabled())
    }

    #[test]
    fn test_enable_selector() {
        let entry_selector = EntrySelector::new(true);
        assert_eq!(true, entry_selector.is_enabled())
    }
}
