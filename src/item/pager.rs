use crate::page::Page;

use super::Item;

struct ItemPager {
    pages: Vec<Page>,
    current_page: Page,
}

impl ItemPager {
    fn new() -> Self {
        ItemPager {
            pages: Vec::new(),
            current_page: Page::new(),
        }
    }
    fn add_item(&mut self, item: Item) {}

    fn split_line(&mut self) {}

    fn split_page(&mut self) {}

    fn get_pages(&self) -> Vec<Page> {
        let mut pages = self.pages.to_vec();
        pages.push(self.current_page.clone());
        return pages;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_initialize_to_single_empty_page() {
        let pager = ItemPager::new();

        let pages = pager.get_pages();

        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].items.len(), 0);
    }
}
