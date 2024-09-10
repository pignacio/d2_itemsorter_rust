use std::borrow::Cow;

use ratatui::{
    crossterm::event::Event,
    layout::Rect,
    style::{Color, Style},
    widgets::Borders,
    Frame,
};

pub enum CompAction {
    ProcessEvent(Event),
    SelectNext,
    SelectPrevious,
}

pub trait Component {
    fn render(&self, start: usize, frame: &mut Frame, area: Rect) -> u16;
    fn height(&self, width: usize) -> usize;
    fn current_position(&self) -> usize;
    fn handle_action(&mut self, action: CompAction) -> Option<CompAction>;
}

pub struct Text<'a> {
    text: Cow<'a, str>,
}

impl<'a> Text<'a> {
    pub fn new(text: impl Into<Cow<'a, str>>) -> Self {
        Self { text: text.into() }
    }
}

impl<'a> Component for Text<'a> {
    fn render(&self, start: usize, frame: &mut Frame, area: Rect) -> u16 {
        let text = self.text.lines().skip(start).collect::<Vec<_>>();
        frame.render_widget(ratatui::text::Text::raw(text.join("\n")), area);
        text.len() as u16
    }

    fn height(&self, _width: usize) -> usize {
        self.text.lines().count()
    }

    fn current_position(&self) -> usize {
        0
    }

    fn handle_action(&mut self, action: CompAction) -> Option<CompAction> {
        Some(action)
    }
}

pub struct Group {
    pub components: Vec<Box<dyn Component>>,
    pub spacing: u16,
    selected_index: Option<usize>,
}

impl Group {
    pub fn new(components: Vec<Box<dyn Component>>, spacing: u16) -> Self {
        Self {
            components,
            spacing,
            selected_index: None,
        }
    }
}

impl Component for Group {
    fn render(&self, mut start: usize, frame: &mut Frame, mut area: Rect) -> u16 {
        let start_y = area.y;
        for component in self.components.iter() {
            let height = component.height(area.width.into());
            if start > height {
                start = start.saturating_sub(height + 1);
                continue;
            }

            let used_height = component.render(start, frame, area);
            start = 0;
            area.y += used_height + self.spacing;
        }
        area.y - start_y
    }

    fn height(&self, width: usize) -> usize {
        let spacing: usize = self.spacing.into();
        self.components
            .iter()
            .map(|c| c.height(width))
            .sum::<usize>()
            + spacing * (self.components.len() - 1)
    }

    fn current_position(&self) -> usize {
        let index = self.selected_index.unwrap_or(0);
        self.components
            .iter()
            .take(index)
            .map(|c| c.current_position() + self.spacing as usize)
            .sum::<usize>()
    }

    fn handle_action(&mut self, action: CompAction) -> Option<CompAction> {
        let residual_action = if let Some(index) = self.selected_index {
            self.components[index].handle_action(action)?
        } else {
            action
        };

        match residual_action {
            CompAction::ProcessEvent(event) => Some(residual_action),
            CompAction::SelectNext => {
                let Some(index) = self.selected_index else {
                    self.selected_index = Some(0);
                    return None;
                };
                if index + 1 >= self.components.len() {
                    self.selected_index = None;
                    return Some(CompAction::SelectNext);
                }
                self.selected_index = Some(index + 1);
                None
            }
            CompAction::SelectPrevious => {
                let Some(index) = self.selected_index else {
                    self.selected_index = Some(self.components.len() - 1);
                    return None;
                };
                if index == 0 {
                    self.selected_index = None;
                    return Some(CompAction::SelectPrevious);
                }
                self.selected_index = Some(index - 1);
                None
            }
        }
    }
}

pub struct Block {
    width: u16,
    height: u16,
    selected: bool,
}

impl Block {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            selected: false,
        }
    }
}

impl Component for Block {
    fn render(&self, start: usize, frame: &mut Frame, area: Rect) -> u16 {
        if start > 0 {
            return 0;
        }
        let mut block = ratatui::widgets::Block::default()
            .title(format!("Block {}x{}", self.width, self.height))
            .borders(Borders::ALL);

        if self.selected {
            block = block.style(Style::default().fg(Color::Red));
        }

        frame.render_widget(
            block,
            area.intersection(Rect::new(area.x, area.y, self.width, self.height)),
        );

        self.height.into()
    }

    fn height(&self, _width: usize) -> usize {
        self.height.into()
    }

    fn current_position(&self) -> usize {
        0
    }

    fn handle_action(&mut self, action: CompAction) -> Option<CompAction> {
        match action {
            CompAction::SelectNext | CompAction::SelectPrevious => {
                self.selected = !self.selected;
                if self.selected {
                    None
                } else {
                    Some(action)
                }
            }
            _ => Some(action),
        }
    }
}
