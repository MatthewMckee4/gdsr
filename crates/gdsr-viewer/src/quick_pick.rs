/// An item that can be displayed in a [`QuickPick`].
pub trait QuickPickItem {
    /// Text used for case-insensitive substring filtering.
    fn filter_text(&self) -> &str;

    /// Render the item content. The picker handles selection highlighting.
    fn ui(&self, ui: &mut egui::Ui);
}

impl QuickPickItem for String {
    fn filter_text(&self) -> &str {
        self
    }

    fn ui(&self, ui: &mut egui::Ui) {
        ui.label(self.as_str());
    }
}

/// A filterable floating picker dialog, similar to VS Code's Quick Pick or Zed's command palette.
pub struct QuickPick<T> {
    open: bool,
    query: String,
    hint: String,
    /// Index into the filtered list (not the original items).
    cursor: usize,
    scroll_to_cursor: bool,
    items: Vec<T>,
    filterable: bool,
}

/// The result of showing a `QuickPick` for a single frame.
pub enum QuickPickResult {
    /// No selection was made this frame.
    None,
    /// The user selected an item (by click or Enter). The value is the index into the
    /// original items slice.
    Selected(usize),
    /// The user dismissed the picker (Escape or close button).
    Dismissed,
}

impl<T: QuickPickItem> QuickPick<T> {
    pub fn new(hint: impl Into<String>, filterable: bool) -> Self {
        Self {
            open: false,
            query: String::new(),
            hint: hint.into(),
            cursor: 0,
            scroll_to_cursor: false,
            items: Vec::new(),
            filterable,
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn items(&self) -> &[T] {
        &self.items
    }

    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
    }

    pub fn open(&mut self) {
        self.open = true;
        self.query.clear();
        self.cursor = 0;
        self.scroll_to_cursor = false;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.query.clear();
        self.cursor = 0;
        self.scroll_to_cursor = false;
    }

    pub fn toggle(&mut self) {
        if self.open {
            self.close();
        } else {
            self.open();
        }
    }

    /// Shows the picker and returns the outcome for this frame.
    ///
    /// When `filterable` is true, items are filtered by case-insensitive substring match
    /// on their `filter_text`. When false, all items are always shown.
    pub fn show(&mut self, ctx: &egui::Context) -> QuickPickResult {
        if !self.open {
            return QuickPickResult::None;
        }

        let mut result = QuickPickResult::None;
        let query_lower = self.query.to_lowercase();

        let filtered_indices: Vec<usize> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                !self.filterable
                    || query_lower.is_empty()
                    || item.filter_text().to_lowercase().contains(&query_lower)
            })
            .map(|(i, _)| i)
            .collect();

        // Clamp cursor to valid range when the filtered list changes.
        if !filtered_indices.is_empty() {
            self.cursor = self.cursor.min(filtered_indices.len() - 1);
        } else {
            self.cursor = 0;
        }

        egui::Window::new(&self.hint)
            .title_bar(false)
            .resizable(false)
            .fixed_size([400.0, 300.0])
            .anchor(egui::Align2::CENTER_TOP, [0.0, 80.0])
            .show(ctx, |ui| {
                let prev_query = self.query.clone();
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.query)
                        .hint_text(&self.hint)
                        .desired_width(f32::INFINITY),
                );
                response.request_focus();

                // Reset cursor to top when the query changes.
                if self.query != prev_query {
                    self.cursor = 0;
                    self.scroll_to_cursor = true;
                }

                // Arrow key navigation (consumed so the text cursor doesn't move).
                if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)) {
                    if !filtered_indices.is_empty() {
                        self.cursor = (self.cursor + 1).min(filtered_indices.len() - 1);
                        self.scroll_to_cursor = true;
                    }
                }
                if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)) {
                    self.cursor = self.cursor.saturating_sub(1);
                    self.scroll_to_cursor = true;
                }

                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Some(&idx) = filtered_indices.get(self.cursor) {
                        result = QuickPickResult::Selected(idx);
                    }
                }

                ui.add_space(4.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (pos, &idx) in filtered_indices.iter().enumerate() {
                        let highlighted = pos == self.cursor;
                        let item = &self.items[idx];

                        let bg_idx = ui.painter().add(egui::Shape::Noop);

                        let inner = ui.horizontal(|ui| {
                            ui.set_min_width(ui.available_width());
                            item.ui(ui);
                        });
                        let response = inner.response.interact(egui::Sense::click());

                        let visuals = ui.style().interact_selectable(&response, highlighted);
                        if highlighted
                            || response.hovered()
                            || response.highlighted()
                            || response.has_focus()
                        {
                            ui.painter().set(
                                bg_idx,
                                egui::Shape::from(egui::epaint::RectShape::filled(
                                    response.rect.expand(visuals.expansion),
                                    visuals.corner_radius,
                                    visuals.bg_fill,
                                )),
                            );
                        }

                        if highlighted && self.scroll_to_cursor {
                            response.scroll_to_me(Some(egui::Align::Center));
                            self.scroll_to_cursor = false;
                        }
                        if response.clicked() {
                            result = QuickPickResult::Selected(idx);
                        }
                    }
                });
            });

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            result = QuickPickResult::Dismissed;
        }

        if matches!(
            result,
            QuickPickResult::Selected(_) | QuickPickResult::Dismissed
        ) {
            self.close();
        }

        result
    }
}
