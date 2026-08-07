use std::path::{Path, PathBuf};
use std::sync::mpsc;

use bevy::prelude::NonSendMut;
use bevy_egui::EguiContexts;
use gdsr::{
    DEFAULT_FLOAT_UNITS, DEFAULT_INTEGER_UNITS, DataType, Element, HorizontalPresentation, Layer,
    Movable, Path as GdsPath, PathType, Point, Polygon, Radians, Text, Unit, VerticalPresentation,
};

use crate::drawable::{Drawable, cell_world_bbox};
use crate::panels;
use crate::quick_pick::{QuickPick, QuickPickResult};
use crate::recent::{RecentProjectItem, RecentProjects};
use crate::ruler::RulerState;
use crate::spatial::SpatialGrid;
use crate::state::{
    CellState, CellViewMode, DisplayUnit, FileLoadState, GridSpacing, LayerState, SidePanelTab,
};
use crate::viewport::Viewport;

/// Returns shortcut text with the platform-appropriate modifier (⌘ on macOS, Ctrl on others).
fn shortcut_text(key: &str) -> String {
    let modifier = if cfg!(target_os = "macos") {
        "⌘"
    } else {
        "Ctrl+"
    };
    format!("{modifier}{key}")
}

fn path_type_label(path_type: PathType) -> &'static str {
    match path_type {
        PathType::Square => "Square",
        PathType::Round => "Round",
        PathType::Overlap => "Overlap",
    }
}

fn inverse_delta(delta: Point) -> Point {
    let (x_units, y_units) = delta.units();
    Point::new(
        Unit::float(0.0, x_units) - delta.x(),
        Unit::float(0.0, y_units) - delta.y(),
    )
}

fn move_element_to_cursor(element: Element, wx: f64, wy: f64) -> Element {
    let target = Point::float(wx, wy, 1.0);
    if let Some(bbox) = element.world_bbox() {
        let center = Point::float(
            (bbox.min_x + bbox.max_x) * 0.5,
            (bbox.min_y + bbox.max_y) * 0.5,
            1.0,
        );
        return element.move_by(target - center);
    }

    element.move_to(target)
}

#[derive(Clone, Debug, PartialEq)]
enum EditAction {
    Add {
        cell_name: String,
        index: usize,
        element: Element,
    },
    Delete {
        cell_name: String,
        index: usize,
        element: Element,
    },
    Move {
        cell_name: String,
        index: usize,
        delta: Point,
    },
}

impl EditAction {
    fn cell_name(&self) -> &str {
        match self {
            Self::Add { cell_name, .. }
            | Self::Delete { cell_name, .. }
            | Self::Move { cell_name, .. } => cell_name,
        }
    }

    fn inverse(&self) -> Self {
        match self {
            Self::Add {
                cell_name,
                index,
                element,
            } => Self::Delete {
                cell_name: cell_name.clone(),
                index: *index,
                element: element.clone(),
            },
            Self::Delete {
                cell_name,
                index,
                element,
            } => Self::Add {
                cell_name: cell_name.clone(),
                index: *index,
                element: element.clone(),
            },
            Self::Move {
                cell_name,
                index,
                delta,
            } => Self::Move {
                cell_name: cell_name.clone(),
                index: *index,
                delta: inverse_delta(*delta),
            },
        }
    }

    fn selected_element_after_apply(&self) -> Option<usize> {
        match self {
            Self::Add { index, .. } | Self::Move { index, .. } => Some(*index),
            Self::Delete { .. } => None,
        }
    }

    fn merge_next(&mut self, next: &Self) -> bool {
        let Self::Move {
            cell_name,
            index,
            delta,
        } = self
        else {
            return false;
        };
        let Self::Move {
            cell_name: next_cell_name,
            index: next_index,
            delta: next_delta,
        } = next
        else {
            return false;
        };

        if cell_name != next_cell_name || index != next_index {
            return false;
        }

        *delta = *delta + *next_delta;
        true
    }
}

#[derive(Clone, Debug, PartialEq)]
enum CellNameDialogMode {
    Create,
    Rename { old_name: String },
}

#[derive(Clone, Debug, PartialEq)]
struct CellNameDialog {
    mode: CellNameDialogMode,
    name: String,
    error: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct TextDialog {
    value: String,
    error: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct PolygonDialog {
    layer: u16,
    data_type: u16,
}

#[derive(Clone, Debug, PartialEq)]
struct PolygonTool {
    layer: u16,
    data_type: u16,
    points: Vec<Point>,
    error: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct PathDialog {
    layer: u16,
    data_type: u16,
    width: f64,
    path_type: PathType,
}

#[derive(Clone, Debug, PartialEq)]
struct PathTool {
    layer: u16,
    data_type: u16,
    width: f64,
    path_type: PathType,
    points: Vec<Point>,
    error: Option<String>,
}

pub struct ViewerApp {
    file_load: FileLoadState,
    cell: Option<CellState>,
    layer_state: LayerState,
    viewport: Viewport,
    mouse_world_pos: Option<(f64, f64)>,
    ruler: RulerState,
    show_grid: bool,
    snap_to_grid: bool,
    hovered_element: Option<usize>,
    selected_element: Option<usize>,
    /// Reusable scratch buffer for spatial grid point queries.
    query_buf: Vec<u32>,
    side_panel_tab: SidePanelTab,
    cell_view_mode: CellViewMode,
    scroll_to_selected: bool,
    recent_projects: RecentProjects,
    display_unit: DisplayUnit,
    grid_spacing: GridSpacing,
    cell_picker: QuickPick<String>,
    recent_picker: QuickPick<RecentProjectItem>,
    has_unsaved_changes: bool,
    save_error: Option<String>,
    show_unsaved_close_prompt: bool,
    undo_stack: Vec<EditAction>,
    redo_stack: Vec<EditAction>,
    cell_name_dialog: Option<CellNameDialog>,
    clipboard_element: Option<Element>,
    text_dialog: Option<TextDialog>,
    polygon_dialog: Option<PolygonDialog>,
    polygon_tool: Option<PolygonTool>,
    path_dialog: Option<PathDialog>,
    path_tool: Option<PathTool>,
    viewport_rect: egui::Rect,
    render_generation: u64,
    geometry_generation: u64,
}

impl Default for ViewerApp {
    fn default() -> Self {
        Self {
            file_load: FileLoadState::default(),
            cell: None,
            layer_state: LayerState::default(),
            viewport: Viewport::default(),
            mouse_world_pos: None,
            ruler: RulerState::default(),
            show_grid: true,
            snap_to_grid: false,
            hovered_element: None,
            selected_element: None,
            query_buf: Vec::new(),
            side_panel_tab: SidePanelTab::default(),
            cell_view_mode: CellViewMode::default(),
            scroll_to_selected: false,
            display_unit: DisplayUnit::default(),
            grid_spacing: GridSpacing::default(),
            recent_projects: RecentProjects::load(),
            cell_picker: QuickPick::new("Search cells…", true),
            recent_picker: QuickPick::new("Recent projects…", false),
            has_unsaved_changes: false,
            save_error: None,
            show_unsaved_close_prompt: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            cell_name_dialog: None,
            clipboard_element: None,
            text_dialog: None,
            polygon_dialog: None,
            polygon_tool: None,
            path_dialog: None,
            path_tool: None,
            viewport_rect: egui::Rect::NOTHING,
            render_generation: 0,
            geometry_generation: 0,
        }
    }
}

impl ViewerApp {
    fn invalidate_render(&mut self) {
        self.render_generation = self.render_generation.saturating_add(1);
    }

    fn invalidate_geometry(&mut self) {
        self.sync_layer_state();
        self.geometry_generation = self.geometry_generation.saturating_add(1);
        self.invalidate_render();
    }

    fn sync_layer_state(&mut self) {
        let Some(cell) = self.cell.as_ref() else {
            return;
        };
        self.layer_state.sync_layers(&cell.layers);
        for &(layer, data_type) in &cell.layers {
            self.layer_state.layer_colors.get(layer, data_type);
        }
    }

    pub(crate) const fn geometry_generation(&self) -> u64 {
        self.geometry_generation
    }

    pub(crate) const fn render_generation(&self) -> u64 {
        self.render_generation
    }

    pub(crate) fn library(&self) -> Option<&gdsr::Library> {
        self.cell.as_ref().map(|cell| &cell.library)
    }

    pub(crate) fn selected_cell_name(&self) -> Option<&str> {
        self.cell
            .as_ref()
            .and_then(|cell| cell.selected_cell.as_deref())
    }

    pub(crate) fn render_depth(&self) -> u32 {
        self.cell.as_ref().map_or(1, |cell| cell.render_depth)
    }

    pub(crate) const fn viewport_state(&self) -> (f64, f64, f64) {
        (
            self.viewport.center_x,
            self.viewport.center_y,
            self.viewport.zoom,
        )
    }

    pub(crate) const fn viewport_rect(&self) -> egui::Rect {
        self.viewport_rect
    }

    pub(crate) fn hidden_layers(&self) -> &std::collections::HashSet<(Layer, DataType)> {
        &self.layer_state.hidden_layers
    }

    pub(crate) fn layer_color(&mut self, layer: Layer, data_type: DataType) -> egui::Color32 {
        self.layer_state.layer_colors.get(layer, data_type)
    }

    pub fn with_path(path: &Path) -> Self {
        let (path, rx) = crate::loader::load_request(path);
        let file_load = FileLoadState {
            file_path: Some(path.clone()),
            load_receiver: Some((path, rx)),
            loading: true,
            error_message: None,
        };

        Self {
            file_load,
            ..Default::default()
        }
    }

    /// Opens a native file dialog and starts loading the selected GDS file on a background thread.
    fn open_file_dialog(&mut self) {
        if let Some((path, rx)) = crate::loader::load_file_dialog() {
            self.file_load.load_receiver = Some((path, rx));
            self.file_load.loading = true;
            self.file_load.error_message = None;
        }
    }

    /// Called when the background file loader completes successfully. Populates cell
    /// names and auto-selects the first cell.
    fn on_library_loaded(&mut self, library: gdsr::Library, path: PathBuf) {
        self.recent_projects.add(&path);
        self.recent_projects.save();

        let cell_state = CellState::new(library);
        self.layer_state.reset_for_library();
        self.cell = Some(cell_state);
        self.hovered_element = None;
        self.selected_element = None;
        self.file_load.file_path = Some(path);
        self.file_load.loading = false;
        self.has_unsaved_changes = false;
        self.save_error = None;
        self.show_unsaved_close_prompt = false;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.cell_name_dialog = None;
        self.clipboard_element = None;
        self.text_dialog = None;
        self.polygon_dialog = None;
        self.polygon_tool = None;
        self.path_dialog = None;
        self.path_tool = None;
        self.invalidate_geometry();
    }

    /// Switches to a new cell, keeping hierarchy intact for draw-time expansion.
    fn select_cell(&mut self, name: &str) {
        if let Some(cell) = self.cell.as_mut() {
            cell.selected_cell = Some(name.to_string());
            cell.expand_state.set_expanded(name, true);
            self.scroll_to_selected = true;
            self.hovered_element = None;
            self.selected_element = None;
            cell.load_direct_cell_elements(name);
        }
        self.sync_layer_state();
        self.invalidate_render();
        self.polygon_dialog = None;
        self.polygon_tool = None;
        self.path_dialog = None;
        self.path_tool = None;
    }

    /// Adjusts the viewport to fit all currently loaded elements.
    fn zoom_to_fit(&mut self) {
        if let Some(cell) = self.cell.as_ref() {
            let bounds = cell
                .selected_cell
                .as_ref()
                .and_then(|name| cell_world_bbox(name, &cell.library));
            if let Some(bounds) = bounds {
                let rect =
                    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(800.0, 600.0));
                self.viewport.zoom_to_fit(&bounds, rect);
            }
        }
    }

    /// Loads a file from a path (used by recent projects).
    fn load_path(&mut self, path: &Path) {
        let (path, rx) = crate::loader::load_request(path);
        self.file_load.load_receiver = Some((path, rx));
        self.file_load.loading = true;
        self.file_load.error_message = None;
        self.save_error = None;
    }

    fn save_current_file(&mut self) -> bool {
        if self.cell.is_none() {
            self.save_error = Some("No GDS library loaded".to_string());
            return false;
        }

        if let Some(path) = self.file_load.file_path.clone() {
            return self.save_file_to_path(&path);
        }

        self.save_file_as_dialog()
    }

    fn save_file_as_dialog(&mut self) -> bool {
        if self.cell.is_none() {
            self.save_error = Some("No GDS library loaded".to_string());
            return false;
        }

        let mut dialog = rfd::FileDialog::new().add_filter("GDS files", &["gds", "gds2", "gdsii"]);

        if let Some(path) = &self.file_load.file_path {
            if let Some(parent) = path.parent() {
                dialog = dialog.set_directory(parent);
            }
            if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
                dialog = dialog.set_file_name(file_name);
            }
        } else {
            dialog = dialog.set_file_name("layout.gds");
        }

        let Some(path) = dialog.save_file() else {
            return false;
        };

        self.save_file_to_path(&path)
    }

    fn save_file_to_path(&mut self, path: &Path) -> bool {
        let Some(cell) = self.cell.as_ref() else {
            self.save_error = Some("No GDS library loaded".to_string());
            return false;
        };

        match cell
            .library
            .write_file(path, DEFAULT_FLOAT_UNITS, DEFAULT_INTEGER_UNITS)
        {
            Ok(()) => {
                self.file_load.file_path = Some(path.to_path_buf());
                self.has_unsaved_changes = false;
                self.save_error = None;
                self.show_unsaved_close_prompt = false;
                self.recent_projects.add(path);
                self.recent_projects.save();
                true
            }
            Err(err) => {
                self.save_error = Some(err.to_string());
                false
            }
        }
    }

    fn cancel_close_for_unsaved_changes(&mut self) -> bool {
        if !self.has_unsaved_changes {
            return false;
        }

        self.show_unsaved_close_prompt = true;
        true
    }

    fn unique_cell_name(&self, base: &str) -> String {
        let Some(cell) = self.cell.as_ref() else {
            return base.to_string();
        };
        if !cell.library.cells().contains_key(base) {
            return base.to_string();
        }

        for suffix in 1.. {
            let candidate = format!("{base}_{suffix}");
            if !cell.library.cells().contains_key(&candidate) {
                return candidate;
            }
        }

        base.to_string()
    }

    fn snap_world_position(&self, x: f64, y: f64) -> (f64, f64) {
        if !self.snap_to_grid {
            return (x, y);
        }

        let spacing = crate::grid::effective_spacing(self.grid_spacing, self.viewport.zoom);
        if !spacing.is_finite() || spacing <= 0.0 {
            return (x, y);
        }

        (
            (x / spacing).round() * spacing,
            (y / spacing).round() * spacing,
        )
    }

    fn snapped_move_delta(&self, index: usize, delta: Point) -> Option<Point> {
        if !self.snap_to_grid {
            return Some(delta);
        }

        let bbox = self
            .cell
            .as_ref()
            .and_then(|cell| cell.elements.get(index))
            .and_then(Element::world_bbox)?;
        let center_x = (bbox.min_x + bbox.max_x) * 0.5;
        let center_y = (bbox.min_y + bbox.max_y) * 0.5;
        let target_x = center_x + delta.x().absolute_value();
        let target_y = center_y + delta.y().absolute_value();
        let (snapped_x, snapped_y) = self.snap_world_position(target_x, target_y);
        Some(Point::float(
            snapped_x - center_x,
            snapped_y - center_y,
            1.0,
        ))
    }

    fn delta_is_zero(delta: Point) -> bool {
        delta.x().absolute_value().abs() < 1e-15 && delta.y().absolute_value().abs() < 1e-15
    }

    fn open_create_cell_dialog(&mut self) {
        self.cell_name_dialog = Some(CellNameDialog {
            mode: CellNameDialogMode::Create,
            name: self.unique_cell_name("new_cell"),
            error: None,
        });
    }

    fn open_rename_cell_dialog(&mut self) {
        let Some(old_name) = self
            .cell
            .as_ref()
            .and_then(|cell| cell.selected_cell.clone())
        else {
            return;
        };

        self.cell_name_dialog = Some(CellNameDialog {
            mode: CellNameDialogMode::Rename {
                old_name: old_name.clone(),
            },
            name: old_name,
            error: None,
        });
    }

    fn open_text_dialog(&mut self) {
        if self
            .cell
            .as_ref()
            .and_then(|cell| cell.selected_cell.as_ref())
            .is_none()
        {
            return;
        }

        self.text_dialog = Some(TextDialog {
            value: "Text".to_string(),
            error: None,
        });
    }

    fn open_polygon_dialog(&mut self) {
        if self
            .cell
            .as_ref()
            .and_then(|cell| cell.selected_cell.as_ref())
            .is_none()
        {
            return;
        }

        self.polygon_dialog = Some(PolygonDialog {
            layer: 0,
            data_type: 0,
        });
    }

    fn open_path_dialog(&mut self) {
        if self
            .cell
            .as_ref()
            .and_then(|cell| cell.selected_cell.as_ref())
            .is_none()
        {
            return;
        }

        self.path_dialog = Some(PathDialog {
            layer: 0,
            data_type: 0,
            width: 1.0,
            path_type: PathType::default(),
        });
    }

    fn validate_cell_name(
        &self,
        name: &str,
        existing_name: Option<&str>,
    ) -> Result<String, String> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err("Cell name is required".to_string());
        }

        let Some(cell) = self.cell.as_ref() else {
            return Err("No GDS library loaded".to_string());
        };
        if Some(trimmed) != existing_name && cell.library.cells().contains_key(trimmed) {
            return Err("Cell name already exists".to_string());
        }

        Ok(trimmed.to_string())
    }

    fn mark_cell_structure_changed(&mut self) {
        self.selected_element = None;
        self.hovered_element = None;
        self.invalidate_geometry();
        self.has_unsaved_changes = true;
        self.save_error = None;
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    fn create_cell_named(&mut self, name: &str) -> Result<(), String> {
        let name = self.validate_cell_name(name, None)?;
        let Some(cell) = self.cell.as_mut() else {
            return Err("No GDS library loaded".to_string());
        };
        if !cell.create_cell(&name) {
            return Err("Could not create cell".to_string());
        }

        self.scroll_to_selected = true;
        self.mark_cell_structure_changed();
        Ok(())
    }

    fn rename_cell_named(&mut self, old_name: &str, new_name: &str) -> Result<(), String> {
        let new_name = self.validate_cell_name(new_name, Some(old_name))?;
        if new_name == old_name {
            return Ok(());
        }

        let Some(cell) = self.cell.as_mut() else {
            return Err("No GDS library loaded".to_string());
        };
        if !cell.rename_cell(old_name, &new_name) {
            return Err("Could not rename cell".to_string());
        }

        self.scroll_to_selected = true;
        self.mark_cell_structure_changed();
        Ok(())
    }

    fn submit_cell_name_dialog(&mut self) {
        let Some(dialog) = self.cell_name_dialog.clone() else {
            return;
        };

        let result = match dialog.mode {
            CellNameDialogMode::Create => self.create_cell_named(&dialog.name),
            CellNameDialogMode::Rename { old_name } => {
                self.rename_cell_named(&old_name, &dialog.name)
            }
        };

        match result {
            Ok(()) => {
                self.cell_name_dialog = None;
            }
            Err(err) => {
                if let Some(dialog) = self.cell_name_dialog.as_mut() {
                    dialog.error = Some(err);
                }
            }
        }
    }

    fn text_insert_position(&self) -> (f64, f64) {
        let (x, y) = self
            .mouse_world_pos
            .unwrap_or((self.viewport.center_x, self.viewport.center_y));
        self.snap_world_position(x, y)
    }

    fn add_text_element(&mut self, value: &str) -> Result<(), String> {
        let value = value.trim();
        if value.is_empty() {
            return Err("Text is required".to_string());
        }
        let Some(cell_name) = self
            .cell
            .as_ref()
            .and_then(|cell| cell.selected_cell.clone())
        else {
            return Err("No cell selected".to_string());
        };

        let (x, y) = self.text_insert_position();
        let element = Element::Text(Text::new(
            value,
            Point::float(x, y, 1.0),
            Layer::new(0),
            DataType::new(0),
            1.0,
            Radians::new(0.0),
            false,
            VerticalPresentation::default(),
            HorizontalPresentation::default(),
        ));

        let Some(cell) = self.cell.as_mut() else {
            return Err("No GDS library loaded".to_string());
        };
        let index = cell.elements.len();
        if !cell.insert_element(index, element.clone()) {
            return Err("Could not add text".to_string());
        }

        self.selected_element = Some(index);
        self.hovered_element = Some(index);
        self.invalidate_geometry();
        self.has_unsaved_changes = true;
        self.save_error = None;
        self.record_edit(EditAction::Add {
            cell_name,
            index,
            element,
        });
        Ok(())
    }

    fn add_polygon_element(
        &mut self,
        points: Vec<Point>,
        layer: Layer,
        data_type: DataType,
    ) -> Result<(), String> {
        if points.len() < 3 {
            return Err("Polygon requires at least three vertices".to_string());
        }
        let Some(cell_name) = self
            .cell
            .as_ref()
            .and_then(|cell| cell.selected_cell.clone())
        else {
            return Err("No cell selected".to_string());
        };

        let element = Element::Polygon(Polygon::new(points, layer, data_type));
        let Some(cell) = self.cell.as_mut() else {
            return Err("No GDS library loaded".to_string());
        };
        let index = cell.elements.len();
        if !cell.insert_element(index, element.clone()) {
            return Err("Could not add polygon".to_string());
        }

        self.selected_element = Some(index);
        self.hovered_element = Some(index);
        self.invalidate_geometry();
        self.has_unsaved_changes = true;
        self.save_error = None;
        self.record_edit(EditAction::Add {
            cell_name,
            index,
            element,
        });
        Ok(())
    }

    fn add_path_element(
        &mut self,
        points: Vec<Point>,
        layer: Layer,
        data_type: DataType,
        width: f64,
        path_type: PathType,
    ) -> Result<(), String> {
        if points.len() < 2 {
            return Err("Path requires at least two points".to_string());
        }
        if !width.is_finite() || width <= 0.0 {
            return Err("Path width must be greater than zero".to_string());
        }
        let Some(cell_name) = self
            .cell
            .as_ref()
            .and_then(|cell| cell.selected_cell.clone())
        else {
            return Err("No cell selected".to_string());
        };

        let element = Element::Path(GdsPath::new(
            points,
            layer,
            data_type,
            Some(path_type),
            Some(Unit::float(width, 1.0)),
            None,
            None,
        ));
        let Some(cell) = self.cell.as_mut() else {
            return Err("No GDS library loaded".to_string());
        };
        let index = cell.elements.len();
        if !cell.insert_element(index, element.clone()) {
            return Err("Could not add path".to_string());
        }

        self.selected_element = Some(index);
        self.hovered_element = Some(index);
        self.invalidate_geometry();
        self.has_unsaved_changes = true;
        self.save_error = None;
        self.record_edit(EditAction::Add {
            cell_name,
            index,
            element,
        });
        Ok(())
    }

    fn begin_polygon_tool(&mut self, layer: u16, data_type: u16) {
        self.ruler.cancel();
        self.path_tool = None;
        self.selected_element = None;
        self.hovered_element = None;
        self.polygon_tool = Some(PolygonTool {
            layer,
            data_type,
            points: Vec::new(),
            error: None,
        });
    }

    fn add_polygon_vertex(&mut self, wx: f64, wy: f64) -> bool {
        let (wx, wy) = self.snap_world_position(wx, wy);
        let point = Point::float(wx, wy, 1.0);
        let Some(tool) = self.polygon_tool.as_mut() else {
            return false;
        };

        if tool.points.last().is_some_and(|last| *last == point) {
            return false;
        }

        tool.points.push(point);
        tool.error = None;
        true
    }

    fn finish_polygon_tool(&mut self) -> bool {
        let Some(tool) = self.polygon_tool.clone() else {
            return false;
        };

        let result = self.add_polygon_element(
            tool.points,
            Layer::new(tool.layer),
            DataType::new(tool.data_type),
        );
        match result {
            Ok(()) => {
                self.polygon_tool = None;
                true
            }
            Err(err) => {
                if let Some(tool) = self.polygon_tool.as_mut() {
                    tool.error = Some(err);
                }
                false
            }
        }
    }

    fn cancel_polygon_tool(&mut self) {
        self.polygon_tool = None;
    }

    fn begin_path_tool(&mut self, layer: u16, data_type: u16, width: f64, path_type: PathType) {
        self.ruler.cancel();
        self.polygon_tool = None;
        self.selected_element = None;
        self.hovered_element = None;
        self.path_tool = Some(PathTool {
            layer,
            data_type,
            width,
            path_type,
            points: Vec::new(),
            error: None,
        });
    }

    fn add_path_vertex(&mut self, wx: f64, wy: f64) -> bool {
        let (wx, wy) = self.snap_world_position(wx, wy);
        let point = Point::float(wx, wy, 1.0);
        let Some(tool) = self.path_tool.as_mut() else {
            return false;
        };

        if tool.points.last().is_some_and(|last| *last == point) {
            return false;
        }

        tool.points.push(point);
        tool.error = None;
        true
    }

    fn finish_path_tool(&mut self) -> bool {
        let Some(tool) = self.path_tool.clone() else {
            return false;
        };

        let result = self.add_path_element(
            tool.points,
            Layer::new(tool.layer),
            DataType::new(tool.data_type),
            tool.width,
            tool.path_type,
        );
        match result {
            Ok(()) => {
                self.path_tool = None;
                true
            }
            Err(err) => {
                if let Some(tool) = self.path_tool.as_mut() {
                    tool.error = Some(err);
                }
                false
            }
        }
    }

    fn cancel_path_tool(&mut self) {
        self.path_tool = None;
    }

    fn submit_text_dialog(&mut self) {
        let Some(dialog) = self.text_dialog.clone() else {
            return;
        };

        match self.add_text_element(&dialog.value) {
            Ok(()) => {
                self.text_dialog = None;
            }
            Err(err) => {
                if let Some(dialog) = self.text_dialog.as_mut() {
                    dialog.error = Some(err);
                }
            }
        }
    }

    fn draw_cell_name_dialog(&mut self, ctx: &egui::Context) {
        let Some(dialog) = self.cell_name_dialog.as_mut() else {
            return;
        };

        let mut submit = false;
        let mut cancel = false;
        let title = match dialog.mode {
            CellNameDialogMode::Create => "New Cell",
            CellNameDialogMode::Rename { .. } => "Rename Cell",
        };
        let primary_label = match dialog.mode {
            CellNameDialogMode::Create => "Create",
            CellNameDialogMode::Rename { .. } => "Rename",
        };

        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                let response =
                    ui.add(egui::TextEdit::singleline(&mut dialog.name).desired_width(240.0));
                if response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
                    submit = true;
                }
                if let Some(err) = &dialog.error {
                    ui.colored_label(egui::Color32::RED, err);
                }
                ui.horizontal(|ui| {
                    if ui.button(primary_label).clicked() {
                        submit = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                });
            });

        if cancel {
            self.cell_name_dialog = None;
        } else if submit {
            self.submit_cell_name_dialog();
        }
    }

    fn draw_text_dialog(&mut self, ctx: &egui::Context) {
        let Some(dialog) = self.text_dialog.as_mut() else {
            return;
        };

        let mut submit = false;
        let mut cancel = false;
        egui::Window::new("Add Text")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                let response =
                    ui.add(egui::TextEdit::singleline(&mut dialog.value).desired_width(240.0));
                if response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
                    submit = true;
                }
                if let Some(err) = &dialog.error {
                    ui.colored_label(egui::Color32::RED, err);
                }
                ui.horizontal(|ui| {
                    if ui.button("Add").clicked() {
                        submit = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                });
            });

        if cancel {
            self.text_dialog = None;
        } else if submit {
            self.submit_text_dialog();
        }
    }

    fn draw_polygon_dialog(&mut self, ctx: &egui::Context) {
        let Some(dialog) = self.polygon_dialog.as_mut() else {
            return;
        };

        let mut begin = false;
        let mut cancel = false;
        egui::Window::new("Add Polygon")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Layer");
                    ui.add(egui::DragValue::new(&mut dialog.layer).range(0..=255));
                });
                ui.horizontal(|ui| {
                    ui.label("Datatype");
                    ui.add(egui::DragValue::new(&mut dialog.data_type).range(0..=255));
                });
                ui.horizontal(|ui| {
                    if ui.button("Draw").clicked() {
                        begin = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                });
            });

        if cancel {
            self.polygon_dialog = None;
        } else if begin {
            let layer = dialog.layer;
            let data_type = dialog.data_type;
            self.polygon_dialog = None;
            self.begin_polygon_tool(layer, data_type);
        }
    }

    fn draw_path_dialog(&mut self, ctx: &egui::Context) {
        let Some(dialog) = self.path_dialog.as_mut() else {
            return;
        };

        let mut begin = false;
        let mut cancel = false;
        egui::Window::new("Add Path")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Layer");
                    ui.add(egui::DragValue::new(&mut dialog.layer).range(0..=255));
                });
                ui.horizontal(|ui| {
                    ui.label("Datatype");
                    ui.add(egui::DragValue::new(&mut dialog.data_type).range(0..=255));
                });
                ui.horizontal(|ui| {
                    ui.label("Width");
                    ui.add(
                        egui::DragValue::new(&mut dialog.width)
                            .range(0.0..=f64::INFINITY)
                            .speed(0.1),
                    );
                });
                egui::ComboBox::from_id_salt("path_dialog_type")
                    .selected_text(path_type_label(dialog.path_type))
                    .show_ui(ui, |ui| {
                        for path_type in PathType::values() {
                            ui.selectable_value(
                                &mut dialog.path_type,
                                path_type,
                                path_type_label(path_type),
                            );
                        }
                    });
                ui.horizontal(|ui| {
                    if ui.button("Draw").clicked() {
                        begin = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                });
            });

        if cancel {
            self.path_dialog = None;
        } else if begin {
            let layer = dialog.layer;
            let data_type = dialog.data_type;
            let width = dialog.width;
            let path_type = dialog.path_type;
            self.path_dialog = None;
            self.begin_path_tool(layer, data_type, width, path_type);
        }
    }

    fn record_edit(&mut self, action: EditAction) {
        if let Some(last) = self.undo_stack.last_mut()
            && last.merge_next(&action)
        {
            self.redo_stack.clear();
            return;
        }

        self.undo_stack.push(action);
        self.redo_stack.clear();
    }

    fn apply_edit_action(&mut self, action: &EditAction) -> bool {
        let cell_name = action.cell_name().to_string();
        if self
            .cell
            .as_ref()
            .and_then(|cell| cell.selected_cell.as_deref())
            != Some(cell_name.as_str())
        {
            self.select_cell(&cell_name);
        }

        let Some(cell) = self.cell.as_mut() else {
            self.selected_element = None;
            return false;
        };

        let applied = match action {
            EditAction::Add { index, element, .. } => cell.insert_element(*index, element.clone()),
            EditAction::Delete { index, .. } => cell.delete_element(*index),
            EditAction::Move { index, delta, .. } => cell.move_element(*index, *delta),
        };

        if !applied {
            self.selected_element = None;
            return false;
        }

        self.selected_element = action.selected_element_after_apply();
        self.hovered_element = self.selected_element;
        self.invalidate_geometry();
        self.has_unsaved_changes = true;
        self.save_error = None;
        true
    }

    fn undo_edit(&mut self) -> bool {
        let Some(action) = self.undo_stack.pop() else {
            return false;
        };

        let inverse = action.inverse();
        if !self.apply_edit_action(&inverse) {
            self.undo_stack.push(action);
            return false;
        }

        self.redo_stack.push(action);
        true
    }

    fn redo_edit(&mut self) -> bool {
        let Some(action) = self.redo_stack.pop() else {
            return false;
        };

        if !self.apply_edit_action(&action) {
            self.redo_stack.push(action);
            return false;
        }

        self.undo_stack.push(action);
        true
    }

    fn copy_selected_element(&mut self) -> bool {
        let Some(index) = self.selected_element else {
            return false;
        };
        let Some(element) = self
            .cell
            .as_ref()
            .and_then(|cell| cell.elements.get(index))
            .cloned()
        else {
            return false;
        };

        self.clipboard_element = Some(element);
        true
    }

    fn paste_clipboard_element(&mut self) -> bool {
        let Some(element) = self.clipboard_element.clone() else {
            return false;
        };
        let Some((wx, wy)) = self.mouse_world_pos else {
            return false;
        };
        let Some(cell_name) = self
            .cell
            .as_ref()
            .and_then(|cell| cell.selected_cell.clone())
        else {
            return false;
        };

        let (wx, wy) = self.snap_world_position(wx, wy);
        let pasted = move_element_to_cursor(element, wx, wy);
        let Some(cell) = self.cell.as_mut() else {
            return false;
        };
        let index = cell.elements.len();
        if !cell.insert_element(index, pasted.clone()) {
            return false;
        }

        self.selected_element = Some(index);
        self.hovered_element = Some(index);
        self.invalidate_geometry();
        self.has_unsaved_changes = true;
        self.save_error = None;
        self.record_edit(EditAction::Add {
            cell_name,
            index,
            element: pasted,
        });
        true
    }

    fn delete_selected_element(&mut self) -> bool {
        let Some(index) = self.selected_element else {
            return false;
        };
        let Some((cell_name, element)) = self.cell.as_ref().and_then(|cell| {
            let cell_name = cell.selected_cell.clone()?;
            let element = cell.elements.get(index)?.clone();
            Some((cell_name, element))
        }) else {
            self.selected_element = None;
            return false;
        };
        let Some(cell) = self.cell.as_mut() else {
            self.selected_element = None;
            return false;
        };
        if !cell.delete_element(index) {
            self.selected_element = None;
            return false;
        }

        self.selected_element = None;
        self.hovered_element = None;
        self.invalidate_geometry();
        self.has_unsaved_changes = true;
        self.save_error = None;
        self.record_edit(EditAction::Delete {
            cell_name,
            index,
            element,
        });
        true
    }

    fn move_selected_element(&mut self, delta: Point) -> bool {
        let Some(index) = self.selected_element else {
            return false;
        };
        let Some(delta) = self.snapped_move_delta(index, delta) else {
            return false;
        };
        if Self::delta_is_zero(delta) {
            return false;
        }
        let Some(cell_name) = self
            .cell
            .as_ref()
            .and_then(|cell| cell.selected_cell.clone())
        else {
            self.selected_element = None;
            return false;
        };
        let Some(cell) = self.cell.as_mut() else {
            self.selected_element = None;
            return false;
        };
        if !cell.move_element(index, delta) {
            self.selected_element = None;
            return false;
        }

        self.hovered_element = Some(index);
        self.invalidate_geometry();
        self.has_unsaved_changes = true;
        self.save_error = None;
        self.record_edit(EditAction::Move {
            cell_name,
            index,
            delta,
        });
        true
    }

    fn draw_unsaved_close_prompt(&mut self, ctx: &egui::Context) {
        if !self.show_unsaved_close_prompt {
            return;
        }

        egui::Window::new("Unsaved changes")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label("Save changes before closing?");
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() && self.save_current_file() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button("Discard").clicked() {
                        self.has_unsaved_changes = false;
                        self.show_unsaved_close_prompt = false;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button("Cancel").clicked() {
                        self.show_unsaved_close_prompt = false;
                    }
                });
            });
    }
}

fn hit_test_element(
    elements: &[gdsr::Element],
    grid: &SpatialGrid,
    query_buf: &mut Vec<u32>,
    wx: f64,
    wy: f64,
    zoom: f64,
) -> Option<usize> {
    let candidates = grid.query_point(wx, wy, query_buf);
    for &idx in candidates.iter().rev() {
        if let Some(el) = elements.get(idx as usize) {
            if el.hit_test(wx, wy, zoom) {
                return Some(idx as usize);
            }
        }
    }
    None
}

fn accepts_canvas_shortcut(wants_keyboard_input: bool, modifiers: egui::Modifiers) -> bool {
    !wants_keyboard_input && modifiers.is_none()
}

impl ViewerApp {
    fn update(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.viewport().close_requested()) && self.cancel_close_for_unsaved_changes()
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }

        // Poll background loader
        if let Some((path, rx)) = self.file_load.load_receiver.take() {
            match rx.try_recv() {
                Ok(Ok(library)) => {
                    self.on_library_loaded(library, path);
                }
                Ok(Err(err)) => {
                    self.file_load.error_message = Some(err);
                    self.file_load.loading = false;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    self.file_load.load_receiver = Some((path, rx));
                    ctx.request_repaint();
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.file_load.error_message =
                        Some("File loading thread disconnected".to_string());
                    self.file_load.loading = false;
                }
            }
        }

        if let Some(cell) = self.cell.as_mut() {
            if self
                .selected_element
                .is_some_and(|idx| idx >= cell.elements.len())
            {
                self.selected_element = None;
            }
        }
        let polygon_tool_active = self.polygon_tool.is_some();
        let path_tool_active = self.path_tool.is_some();
        let drawing_tool_active = polygon_tool_active || path_tool_active;

        // Global keyboard shortcuts
        let wants_keyboard_input = ctx.egui_wants_keyboard_input();
        if ctx.input(|i| {
            accepts_canvas_shortcut(wants_keyboard_input, i.modifiers)
                && i.key_pressed(egui::Key::F)
        }) {
            self.zoom_to_fit();
        }
        if ctx.input(|i| {
            accepts_canvas_shortcut(wants_keyboard_input, i.modifiers)
                && i.key_pressed(egui::Key::G)
        }) {
            self.show_grid = !self.show_grid;
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::O) && !i.modifiers.alt) {
            self.open_file_dialog();
        }
        if ctx.input(|i| i.modifiers.command && i.modifiers.alt && i.key_pressed(egui::Key::O)) {
            self.recent_picker.toggle();
        }
        if ctx.input(|i| i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::Z)) {
            if self.redo_edit() {
                ctx.request_repaint();
            }
        } else if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z))
            && self.undo_edit()
        {
            ctx.request_repaint();
        }
        let text_input_open = self.cell_name_dialog.is_some()
            || self.text_dialog.is_some()
            || self.polygon_dialog.is_some()
            || self.path_dialog.is_some()
            || self.cell_picker.is_open()
            || self.recent_picker.is_open();
        if !text_input_open
            && ctx.input(|i| i.key_pressed(egui::Key::Enter))
            && self.polygon_tool.is_some()
            && self.finish_polygon_tool()
        {
            ctx.request_repaint();
        }
        if !text_input_open
            && ctx.input(|i| i.key_pressed(egui::Key::Enter))
            && self.path_tool.is_some()
            && self.finish_path_tool()
        {
            ctx.request_repaint();
        }
        if !text_input_open && ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::C)) {
            self.copy_selected_element();
        }
        if !text_input_open
            && ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::V))
            && self.paste_clipboard_element()
        {
            ctx.request_repaint();
        }
        if ctx.input(|i| i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::S)) {
            self.save_file_as_dialog();
        } else if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
            self.save_current_file();
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::P)) {
            self.cell_picker.toggle();
        }
        if !drawing_tool_active && ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::R))
        {
            self.ruler.clear_all();
        } else if !drawing_tool_active && ctx.input(|i| i.key_pressed(egui::Key::R)) {
            self.ruler.toggle();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape))
            && !self.cell_picker.is_open()
            && !self.recent_picker.is_open()
        {
            if self.polygon_tool.is_some() {
                self.cancel_polygon_tool();
                ctx.request_repaint();
            } else if self.path_tool.is_some() {
                self.cancel_path_tool();
                ctx.request_repaint();
            } else {
                self.ruler.cancel();
            }
        }
        let delete_pressed =
            ctx.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace));
        if delete_pressed
            && !self.cell_picker.is_open()
            && !self.recent_picker.is_open()
            && !drawing_tool_active
            && self.delete_selected_element()
        {
            ctx.request_repaint();
        }
        if self.ruler.start.is_some() {
            ctx.request_repaint();
        }

        let mut viewport_ui = egui::Ui::new(
            ctx.clone(),
            "viewport".into(),
            egui::UiBuilder::new()
                .layer_id(egui::LayerId::background())
                .max_rect(ctx.viewport_rect()),
        );

        egui::Panel::top("menu_bar").show(&mut viewport_ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui
                        .add(egui::Button::new("Open...").shortcut_text(shortcut_text("O")))
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        self.open_file_dialog();
                    }
                    if ui
                        .add(
                            egui::Button::new("Recent Projects...")
                                .shortcut_text(shortcut_text("⌥O")),
                        )
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        self.recent_picker.open();
                    }
                    ui.separator();
                    if ui
                        .add_enabled(
                            self.cell.is_some(),
                            egui::Button::new("Save").shortcut_text(shortcut_text("S")),
                        )
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        self.save_current_file();
                    }
                    if ui
                        .add_enabled(
                            self.cell.is_some(),
                            egui::Button::new("Save As...").shortcut_text(shortcut_text("Shift+S")),
                        )
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        self.save_file_as_dialog();
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui
                        .add_enabled(
                            !self.undo_stack.is_empty(),
                            egui::Button::new("Undo").shortcut_text(shortcut_text("Z")),
                        )
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        if self.undo_edit() {
                            ctx.request_repaint();
                        }
                    }
                    if ui
                        .add_enabled(
                            !self.redo_stack.is_empty(),
                            egui::Button::new("Redo").shortcut_text(shortcut_text("Shift+Z")),
                        )
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        if self.redo_edit() {
                            ctx.request_repaint();
                        }
                    }
                    ui.separator();
                    if ui
                        .add_enabled(
                            self.selected_element.is_some(),
                            egui::Button::new("Copy").shortcut_text(shortcut_text("C")),
                        )
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        self.copy_selected_element();
                    }
                    if ui
                        .add_enabled(
                            self.clipboard_element.is_some()
                                && self
                                    .cell
                                    .as_ref()
                                    .and_then(|cell| cell.selected_cell.as_ref())
                                    .is_some(),
                            egui::Button::new("Paste").shortcut_text(shortcut_text("V")),
                        )
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        if self.paste_clipboard_element() {
                            ctx.request_repaint();
                        }
                    }
                    ui.separator();
                    if ui
                        .add_enabled(
                            self.cell
                                .as_ref()
                                .and_then(|cell| cell.selected_cell.as_ref())
                                .is_some(),
                            egui::Button::new("Add Text..."),
                        )
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        self.open_text_dialog();
                    }
                    if ui
                        .add_enabled(
                            self.cell
                                .as_ref()
                                .and_then(|cell| cell.selected_cell.as_ref())
                                .is_some(),
                            egui::Button::new("Add Polygon..."),
                        )
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        self.open_polygon_dialog();
                    }
                    if ui
                        .add_enabled(
                            self.cell
                                .as_ref()
                                .and_then(|cell| cell.selected_cell.as_ref())
                                .is_some(),
                            egui::Button::new("Add Path..."),
                        )
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        self.open_path_dialog();
                    }
                });
                ui.menu_button("View", |ui| {
                    if ui
                        .add(egui::Button::new("Zoom to Fit").shortcut_text("F"))
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        self.zoom_to_fit();
                    }
                    if ui
                        .add(egui::Button::new("Zoom In").shortcut_text("+"))
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        self.viewport.zoom_at_center(1.2);
                    }
                    if ui
                        .add(egui::Button::new("Zoom Out").shortcut_text("-"))
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        self.viewport.zoom_at_center(1.0 / 1.2);
                    }
                    ui.separator();
                    if ui
                        .add(egui::Button::new("Ruler").shortcut_text("R"))
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        self.ruler.toggle();
                    }
                    if ui
                        .add(egui::Button::new("Clear Rulers").shortcut_text(shortcut_text("R")))
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        self.ruler.clear_all();
                    }
                    if ui
                        .add(
                            egui::Button::new(if self.show_grid {
                                "Hide Grid"
                            } else {
                                "Show Grid"
                            })
                            .shortcut_text("G"),
                        )
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        self.show_grid = !self.show_grid;
                    }
                    ui.checkbox(&mut self.snap_to_grid, "Snap to Grid");
                    ui.separator();
                    ui.label("Pan: Arrow Keys");
                });
            });
        });

        // Bottom activity bar
        let mut depth_changed = false;
        egui::Panel::bottom("status_bar").show(&mut viewport_ui, |ui| {
            ui.horizontal(|ui| {
                let is_tree = self.side_panel_tab == SidePanelTab::Cells
                    && self.cell_view_mode == CellViewMode::Tree;
                if ui
                    .selectable_label(is_tree, "Tree")
                    .on_hover_text("Cell hierarchy")
                    .clicked()
                {
                    self.side_panel_tab = SidePanelTab::Cells;
                    self.cell_view_mode = CellViewMode::Tree;
                }

                let is_flat = self.side_panel_tab == SidePanelTab::Cells
                    && self.cell_view_mode == CellViewMode::Flat;
                if ui
                    .selectable_label(is_flat, "Cells")
                    .on_hover_text("All cells")
                    .clicked()
                {
                    self.side_panel_tab = SidePanelTab::Cells;
                    self.cell_view_mode = CellViewMode::Flat;
                    if self
                        .cell
                        .as_ref()
                        .and_then(|c| c.selected_cell.as_ref())
                        .is_some()
                    {
                        self.scroll_to_selected = true;
                    }
                }

                if ui
                    .selectable_label(self.side_panel_tab == SidePanelTab::Layers, "Layers")
                    .clicked()
                {
                    self.side_panel_tab = SidePanelTab::Layers;
                }

                ui.separator();

                if let Some((wx, wy)) = self.mouse_world_pos {
                    ui.label(self.display_unit.format_pair(wx, wy));
                }

                ui.separator();

                if self.file_load.loading {
                    ui.label("Loading...");
                } else if let Some(err) = &self.file_load.error_message {
                    ui.colored_label(egui::Color32::RED, format!("Error: {err}"));
                } else if let Some(err) = &self.save_error {
                    ui.colored_label(egui::Color32::RED, format!("Save error: {err}"));
                } else if let Some(path) = &self.file_load.file_path {
                    let file_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
                    ui.label(if self.has_unsaved_changes {
                        format!("{file_name}*")
                    } else {
                        file_name.to_string()
                    });
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::ComboBox::from_id_salt("display_unit")
                        .selected_text(self.display_unit.label())
                        .width(60.0)
                        .show_ui(ui, |ui| {
                            for unit in DisplayUnit::ALL {
                                ui.selectable_value(&mut self.display_unit, unit, unit.label());
                            }
                        });
                    ui.label("Grid:");
                    egui::ComboBox::from_id_salt("grid_spacing")
                        .selected_text(self.grid_spacing.label())
                        .width(60.0)
                        .show_ui(ui, |ui| {
                            for &(label, multiplier) in GridSpacing::PRESETS {
                                let preset = GridSpacing { multiplier };
                                ui.selectable_value(&mut self.grid_spacing, preset, label);
                            }
                        });
                    ui.checkbox(&mut self.snap_to_grid, "Snap");
                    if let Some(cell) = &mut self.cell {
                        let prev_depth = cell.render_depth;
                        if ui.small_button("+").clicked() && cell.render_depth < 99 {
                            cell.render_depth += 1;
                        }
                        ui.label(format!("Depth: {}", cell.render_depth));
                        if ui.small_button("−").clicked() && cell.render_depth > 0 {
                            cell.render_depth -= 1;
                        }
                        if cell.render_depth != prev_depth {
                            depth_changed = true;
                        }
                    }
                    if self.ruler.active {
                        ui.label("Ruler: click to place point (Esc to cancel)");
                    }
                    if let Some(tool) = &self.polygon_tool {
                        ui.label(format!("Polygon: {} vertices", tool.points.len()));
                        if let Some(err) = &tool.error {
                            ui.colored_label(egui::Color32::RED, err);
                        }
                    }
                    if let Some(tool) = &mut self.path_tool {
                        ui.label(format!("Path: {} points", tool.points.len()));
                        ui.add(egui::DragValue::new(&mut tool.layer).range(0..=255));
                        ui.add(egui::DragValue::new(&mut tool.data_type).range(0..=255));
                        ui.add(
                            egui::DragValue::new(&mut tool.width)
                                .range(0.0..=f64::INFINITY)
                                .speed(0.1),
                        );
                        egui::ComboBox::from_id_salt("path_tool_type")
                            .selected_text(path_type_label(tool.path_type))
                            .width(84.0)
                            .show_ui(ui, |ui| {
                                for path_type in PathType::values() {
                                    ui.selectable_value(
                                        &mut tool.path_type,
                                        path_type,
                                        path_type_label(path_type),
                                    );
                                }
                            });
                        if let Some(err) = &tool.error {
                            ui.colored_label(egui::Color32::RED, err);
                        }
                    }
                    if let Some(stats) = self.cell.as_ref().and_then(|c| c.cell_stats.as_ref()) {
                        panels::draw_stats_bar(ui, stats);
                    }
                });
            });
        });

        if depth_changed {
            if let Some(cell) = self.cell.as_mut() {
                cell.refresh_layers();
            }
            self.sync_layer_state();
            self.invalidate_render();
        }

        // Cell picker (⌘P)
        self.cell_picker.set_items(
            self.cell
                .as_ref()
                .map(|c| c.cell_names.clone())
                .unwrap_or_default(),
        );
        if let QuickPickResult::Selected(idx) = self.cell_picker.show(ctx) {
            let name = self.cell_picker.items()[idx].clone();
            self.select_cell(&name);
        }

        // Recent projects picker (⌘⌥O)
        self.recent_picker.set_items(
            self.recent_projects
                .paths()
                .iter()
                .map(|p| RecentProjectItem::from_path(p))
                .collect(),
        );
        if let QuickPickResult::Selected(idx) = self.recent_picker.show(ctx) {
            let path = self.recent_picker.items()[idx].path.clone();
            self.load_path(&path);
        }

        let mut cell_changed = false;
        let mut color_changed = false;
        let cell = &mut self.cell;
        let layer_state = &mut self.layer_state;
        let active_tab = self.side_panel_tab;
        let view_mode = self.cell_view_mode;
        let scroll_to_selected = &mut self.scroll_to_selected;
        let selected_element_idx = self.selected_element;
        let mut side_panel_actions = panels::SidePanelActions::default();
        egui::Panel::left("side_panel")
            .default_size(200.0)
            .size_range(40.0..=800.0)
            .resizable(true)
            .show(&mut viewport_ui, |ui| {
                ui.allocate_at_least(egui::vec2(ui.available_width(), 0.0), egui::Sense::hover());
                if let Some(cell) = cell.as_mut() {
                    let selected_element =
                        selected_element_idx.and_then(|idx| cell.elements.get(idx));
                    side_panel_actions = panels::draw_side_panel(
                        ui,
                        active_tab,
                        &cell.cell_tree,
                        &cell.flat_tree,
                        view_mode,
                        &mut cell.selected_cell,
                        &mut cell_changed,
                        &mut color_changed,
                        &mut cell.expand_state,
                        scroll_to_selected,
                        &cell.layers,
                        layer_state,
                        selected_element,
                    );
                }
            });

        if side_panel_actions.delete_selected && self.delete_selected_element() {
            ctx.request_repaint();
        }
        if side_panel_actions.create_cell {
            self.open_create_cell_dialog();
        }
        if side_panel_actions.rename_cell {
            self.open_rename_cell_dialog();
        }

        if cell_changed {
            if let Some(name) = self.cell.as_ref().and_then(|c| c.selected_cell.clone()) {
                self.select_cell(&name);
            }
        }

        // Layer changes require new Bevy materials and a new visible-scene plan.
        if color_changed {
            self.invalidate_render();
        }

        let cell = &mut self.cell;
        let viewport = &mut self.viewport;
        let layer_state = &mut self.layer_state;
        let mouse_world_pos = &mut self.mouse_world_pos;
        let ruler = &mut self.ruler;
        let show_grid = self.show_grid;
        let grid_spacing = self.grid_spacing;
        let hovered_element = &mut self.hovered_element;
        let selected_element = &mut self.selected_element;
        let query_buf = &mut self.query_buf;
        let viewport_rect = &mut self.viewport_rect;
        let drawing_preview_points = self
            .polygon_tool
            .as_ref()
            .map(|tool| tool.points.clone())
            .or_else(|| self.path_tool.as_ref().map(|tool| tool.points.clone()));
        let mut selected_element_drag_delta = None;
        let mut viewport_click_world = None;
        let mut viewport_double_clicked = false;
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(&mut viewport_ui, |ui| {
                let mut empty_cache = std::collections::HashMap::new();
                let (elements, spatial_grid, library, tessellation_cache) =
                    if let Some(cell) = cell.as_mut() {
                        (
                            cell.elements.as_slice(),
                            cell.spatial_grid.as_ref(),
                            Some(&cell.library),
                            &mut cell.tessellation_cache,
                        )
                    } else {
                        (&[] as &[gdsr::Element], None, None, &mut empty_cache)
                    };

                let interaction = viewport.draw(
                    ui,
                    elements,
                    layer_state,
                    library,
                    tessellation_cache,
                    ruler,
                    show_grid,
                    grid_spacing,
                    *hovered_element,
                    *selected_element,
                    drawing_preview_points.as_deref(),
                );
                *viewport_rect = interaction.rect;
                selected_element_drag_delta = interaction.selected_element_drag_delta;
                *mouse_world_pos = interaction.mouse_world;
                if interaction.clicked || interaction.double_clicked {
                    viewport_click_world = interaction.mouse_world;
                    viewport_double_clicked = interaction.double_clicked;
                }

                let prev_hovered = *hovered_element;
                let prev_selected = *selected_element;
                *hovered_element = None;
                if !drawing_tool_active {
                    if let Some((wx, wy)) = *mouse_world_pos {
                        if let Some(grid) = spatial_grid {
                            *hovered_element =
                                hit_test_element(elements, grid, query_buf, wx, wy, viewport.zoom);
                        }
                    }
                    if interaction.clicked {
                        *selected_element = *hovered_element;
                    }
                }
                if *hovered_element != prev_hovered || *selected_element != prev_selected {
                    ctx.request_repaint();
                }
            });
        if polygon_tool_active {
            if let Some((wx, wy)) = viewport_click_world {
                if self.add_polygon_vertex(wx, wy) {
                    ctx.request_repaint();
                }
                if viewport_double_clicked && self.finish_polygon_tool() {
                    ctx.request_repaint();
                }
            }
        } else if path_tool_active {
            if let Some((wx, wy)) = viewport_click_world {
                if self.add_path_vertex(wx, wy) {
                    ctx.request_repaint();
                }
                if viewport_double_clicked && self.finish_path_tool() {
                    ctx.request_repaint();
                }
            }
        } else if let Some((dx, dy)) = selected_element_drag_delta {
            if self.move_selected_element(Point::float(dx, dy, 1.0)) {
                ctx.request_repaint();
            }
        }

        self.draw_cell_name_dialog(ctx);
        self.draw_text_dialog(ctx);
        self.draw_polygon_dialog(ctx);
        self.draw_path_dialog(ctx);
        self.draw_unsaved_close_prompt(ctx);
    }
}

pub fn update_system(
    mut contexts: EguiContexts,
    mut viewer: NonSendMut<ViewerApp>,
) -> bevy::prelude::Result {
    viewer.update(contexts.ctx_mut()?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gdsr::{
        Cell, DataType, Element, Layer, Library, PathType, Point, Polygon, Reference, Unit,
    };

    fn p(x: f64, y: f64) -> Point {
        Point::float(x, y, 1.0)
    }

    #[test]
    fn canvas_shortcuts_require_unmodified_non_text_input() {
        assert!(accepts_canvas_shortcut(false, egui::Modifiers::NONE));
        assert!(!accepts_canvas_shortcut(true, egui::Modifiers::NONE));
        assert!(!accepts_canvas_shortcut(false, egui::Modifiers::CTRL));
    }

    fn test_elements() -> Vec<Element> {
        vec![
            Polygon::new(
                [p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0), p(0.0, 10.0)],
                Layer::new(1),
                DataType::new(0),
            )
            .into(),
        ]
    }

    fn cell_state_with_test_elements() -> CellState {
        let mut source_cell = Cell::new("top");
        for element in test_elements() {
            source_cell.add(element);
        }

        let mut library = Library::new("test");
        library.add_cell(source_cell);

        let mut cell = CellState::new(library);
        cell.selected_cell = Some("top".to_string());
        assert!(cell.load_direct_cell_elements("top"));
        cell
    }

    fn loaded_element_count(app: &ViewerApp) -> usize {
        app.cell
            .as_ref()
            .expect("cell should remain loaded")
            .elements
            .len()
    }

    fn first_element_bbox(app: &ViewerApp) -> crate::drawable::WorldBBox {
        first_element_bbox_at(app, 0)
    }

    fn first_element_bbox_at(app: &ViewerApp, index: usize) -> crate::drawable::WorldBBox {
        app.cell
            .as_ref()
            .expect("cell should remain loaded")
            .elements[index]
            .world_bbox()
            .expect("element has bbox")
    }

    fn library_with_referenced_leaf() -> Library {
        let mut leaf = Cell::new("leaf");
        leaf.add(test_elements().remove(0));

        let mut top = Cell::new("top");
        top.add(Reference::new("leaf"));

        let mut library = Library::new("test");
        library.add_cell(leaf);
        library.add_cell(top);
        library
    }

    #[test]
    fn hit_test_element_returns_matching_index() {
        let elements = test_elements();
        let Some(bounds) = crate::viewport::compute_bounds(&elements) else {
            panic!("test elements should have bounds");
        };
        let grid = SpatialGrid::build(&elements, &bounds);
        let mut query_buf = Vec::new();

        assert_eq!(
            hit_test_element(&elements, &grid, &mut query_buf, 5.0, 5.0, 1.0),
            Some(0)
        );
    }

    #[test]
    fn hit_test_element_returns_none_for_miss() {
        let elements = test_elements();
        let Some(bounds) = crate::viewport::compute_bounds(&elements) else {
            panic!("test elements should have bounds");
        };
        let grid = SpatialGrid::build(&elements, &bounds);
        let mut query_buf = Vec::new();

        assert_eq!(
            hit_test_element(&elements, &grid, &mut query_buf, 20.0, 20.0, 1.0),
            None
        );
    }

    #[test]
    fn delete_selected_element_removes_element_marks_dirty_and_clears_selection() {
        let cell = cell_state_with_test_elements();
        let mut app = ViewerApp {
            cell: Some(cell),
            selected_element: Some(0),
            hovered_element: Some(0),
            ..Default::default()
        };

        assert!(app.delete_selected_element());

        let cell = app.cell.expect("cell should remain loaded");
        assert!(cell.elements.is_empty());
        assert!(
            cell.library
                .get_cell("top")
                .expect("top cell should exist")
                .elements()
                .is_empty()
        );
        assert!(cell.spatial_grid.is_none());
        assert!(app.selected_element.is_none());
        assert!(app.hovered_element.is_none());
        assert!(app.has_unsaved_changes);
        assert_eq!(app.undo_stack.len(), 1);
        assert!(app.redo_stack.is_empty());
    }

    #[test]
    fn move_selected_element_moves_element_marks_dirty_and_keeps_selection() {
        let cell = cell_state_with_test_elements();
        let mut app = ViewerApp {
            cell: Some(cell),
            selected_element: Some(0),
            ..Default::default()
        };

        assert!(app.move_selected_element(Point::float(2.0, 3.0, 1.0)));

        let cell = app.cell.expect("cell should remain loaded");
        let bbox = cell.elements[0]
            .world_bbox()
            .expect("moved element has bbox");
        assert!((bbox.min_x - 2.0).abs() < 1e-12);
        assert!((bbox.min_y - 3.0).abs() < 1e-12);
        assert_eq!(app.selected_element, Some(0));
        assert_eq!(app.hovered_element, Some(0));
        assert!(cell.spatial_grid.is_some());
        assert!(app.has_unsaved_changes);
        assert_eq!(app.undo_stack.len(), 1);
        assert!(app.redo_stack.is_empty());

        let library_bbox = cell
            .library
            .get_cell("top")
            .expect("top cell should exist")
            .elements()[0]
            .world_bbox()
            .expect("moved element has bbox");
        assert!((library_bbox.min_x - 2.0).abs() < 1e-12);
    }

    #[test]
    fn undo_redo_delete_restores_and_removes_element() {
        let cell = cell_state_with_test_elements();
        let mut app = ViewerApp {
            cell: Some(cell),
            selected_element: Some(0),
            ..Default::default()
        };

        assert!(app.delete_selected_element());
        assert_eq!(loaded_element_count(&app), 0);

        assert!(app.undo_edit());

        assert_eq!(loaded_element_count(&app), 1);
        assert_eq!(app.selected_element, Some(0));
        assert!(app.undo_stack.is_empty());
        assert_eq!(app.redo_stack.len(), 1);

        assert!(app.redo_edit());

        assert_eq!(loaded_element_count(&app), 0);
        assert!(app.selected_element.is_none());
        assert_eq!(app.undo_stack.len(), 1);
        assert!(app.redo_stack.is_empty());
    }

    #[test]
    fn undo_redo_move_restores_and_reapplies_total_drag() {
        let cell = cell_state_with_test_elements();
        let mut app = ViewerApp {
            cell: Some(cell),
            selected_element: Some(0),
            ..Default::default()
        };

        assert!(app.move_selected_element(Point::float(2.0, 3.0, 1.0)));
        assert!(app.move_selected_element(Point::float(1.0, 1.0, 1.0)));
        assert_eq!(app.undo_stack.len(), 1);

        let moved_bbox = first_element_bbox(&app);
        assert!((moved_bbox.min_x - 3.0).abs() < 1e-12);
        assert!((moved_bbox.min_y - 4.0).abs() < 1e-12);

        assert!(app.undo_edit());

        let restored_bbox = first_element_bbox(&app);
        assert!((restored_bbox.min_x - 0.0).abs() < 1e-12);
        assert!((restored_bbox.min_y - 0.0).abs() < 1e-12);
        assert_eq!(app.selected_element, Some(0));
        assert!(app.undo_stack.is_empty());
        assert_eq!(app.redo_stack.len(), 1);

        assert!(app.redo_edit());

        let redone_bbox = first_element_bbox(&app);
        assert!((redone_bbox.min_x - 3.0).abs() < 1e-12);
        assert!((redone_bbox.min_y - 4.0).abs() < 1e-12);
        assert_eq!(app.selected_element, Some(0));
        assert_eq!(app.undo_stack.len(), 1);
        assert!(app.redo_stack.is_empty());
    }

    #[test]
    fn add_text_element_adds_selects_and_marks_dirty() {
        let cell = cell_state_with_test_elements();
        let mut app = ViewerApp {
            cell: Some(cell),
            mouse_world_pos: Some((12.0, 34.0)),
            ..Default::default()
        };

        assert!(app.add_text_element(" label ").is_ok());

        assert_eq!(loaded_element_count(&app), 2);
        assert_eq!(app.selected_element, Some(1));
        assert_eq!(app.hovered_element, Some(1));
        assert!(app.has_unsaved_changes);
        assert_eq!(app.undo_stack.len(), 1);
        assert!(app.redo_stack.is_empty());

        let Some(Element::Text(text)) = app
            .cell
            .as_ref()
            .expect("cell should remain loaded")
            .elements
            .get(1)
        else {
            panic!("added element should be text");
        };
        assert_eq!(text.text(), "label");
        assert!((text.origin().x().absolute_value() - 12.0).abs() < 1e-12);
        assert!((text.origin().y().absolute_value() - 34.0).abs() < 1e-12);
    }

    #[test]
    fn add_text_element_snaps_position_when_enabled() {
        let cell = cell_state_with_test_elements();
        let mut app = ViewerApp {
            cell: Some(cell),
            mouse_world_pos: Some((23.0, 37.0)),
            snap_to_grid: true,
            ..Default::default()
        };
        app.viewport.zoom = 10.0;

        assert!(app.add_text_element("pin").is_ok());

        let Some(Element::Text(text)) = app
            .cell
            .as_ref()
            .expect("cell should remain loaded")
            .elements
            .get(1)
        else {
            panic!("added element should be text");
        };
        assert!((text.origin().x().absolute_value() - 20.0).abs() < 1e-12);
        assert!((text.origin().y().absolute_value() - 40.0).abs() < 1e-12);
    }

    #[test]
    fn undo_add_text_element_removes_it() {
        let cell = cell_state_with_test_elements();
        let mut app = ViewerApp {
            cell: Some(cell),
            mouse_world_pos: Some((12.0, 34.0)),
            ..Default::default()
        };

        assert!(app.add_text_element("label").is_ok());
        assert_eq!(loaded_element_count(&app), 2);

        assert!(app.undo_edit());

        assert_eq!(loaded_element_count(&app), 1);
        assert!(app.selected_element.is_none());
        assert!(app.undo_stack.is_empty());
        assert_eq!(app.redo_stack.len(), 1);
    }

    #[test]
    fn add_polygon_element_adds_selects_and_marks_dirty() {
        let cell = cell_state_with_test_elements();
        let mut app = ViewerApp {
            cell: Some(cell),
            ..Default::default()
        };

        assert!(
            app.add_polygon_element(
                vec![p(1.0, 2.0), p(3.0, 2.0), p(3.0, 4.0)],
                Layer::new(7),
                DataType::new(2),
            )
            .is_ok()
        );

        assert_eq!(loaded_element_count(&app), 2);
        assert_eq!(app.selected_element, Some(1));
        assert_eq!(app.hovered_element, Some(1));
        assert!(app.has_unsaved_changes);
        assert_eq!(app.undo_stack.len(), 1);
        assert!(app.redo_stack.is_empty());

        let Some(Element::Polygon(polygon)) = app
            .cell
            .as_ref()
            .expect("cell should remain loaded")
            .elements
            .get(1)
        else {
            panic!("added element should be polygon");
        };
        assert_eq!(polygon.layer(), Layer::new(7));
        assert_eq!(polygon.data_type(), DataType::new(2));
        assert_eq!(polygon.points().len(), 4);
        assert_eq!(polygon.points().first(), polygon.points().last());
    }

    #[test]
    fn polygon_tool_snaps_vertices_when_enabled() {
        let cell = cell_state_with_test_elements();
        let mut app = ViewerApp {
            cell: Some(cell),
            snap_to_grid: true,
            ..Default::default()
        };
        app.viewport.zoom = 10.0;
        app.begin_polygon_tool(7, 2);

        assert!(app.add_polygon_vertex(23.0, 37.0));

        let tool = app.polygon_tool.expect("polygon tool should remain active");
        assert_eq!(tool.points, vec![p(20.0, 40.0)]);
    }

    #[test]
    fn finish_polygon_tool_requires_three_vertices() {
        let cell = cell_state_with_test_elements();
        let mut app = ViewerApp {
            cell: Some(cell),
            ..Default::default()
        };
        app.begin_polygon_tool(7, 2);
        assert!(app.add_polygon_vertex(1.0, 1.0));
        assert!(app.add_polygon_vertex(2.0, 2.0));

        assert!(!app.finish_polygon_tool());

        assert_eq!(loaded_element_count(&app), 1);
        assert_eq!(
            app.polygon_tool
                .as_ref()
                .and_then(|tool| tool.error.as_deref()),
            Some("Polygon requires at least three vertices")
        );
    }

    #[test]
    fn finish_polygon_tool_adds_polygon_with_target_layer() {
        let cell = cell_state_with_test_elements();
        let mut app = ViewerApp {
            cell: Some(cell),
            ..Default::default()
        };
        app.begin_polygon_tool(9, 4);
        assert!(app.add_polygon_vertex(1.0, 1.0));
        assert!(app.add_polygon_vertex(3.0, 1.0));
        assert!(app.add_polygon_vertex(3.0, 3.0));

        assert!(app.finish_polygon_tool());

        assert!(app.polygon_tool.is_none());
        assert_eq!(loaded_element_count(&app), 2);
        let Some(Element::Polygon(polygon)) = app
            .cell
            .as_ref()
            .expect("cell should remain loaded")
            .elements
            .get(1)
        else {
            panic!("added element should be polygon");
        };
        assert_eq!(polygon.layer(), Layer::new(9));
        assert_eq!(polygon.data_type(), DataType::new(4));
    }

    #[test]
    fn undo_add_polygon_element_removes_it() {
        let cell = cell_state_with_test_elements();
        let mut app = ViewerApp {
            cell: Some(cell),
            ..Default::default()
        };

        assert!(
            app.add_polygon_element(
                vec![p(1.0, 2.0), p(3.0, 2.0), p(3.0, 4.0)],
                Layer::new(7),
                DataType::new(2),
            )
            .is_ok()
        );
        assert_eq!(loaded_element_count(&app), 2);

        assert!(app.undo_edit());

        assert_eq!(loaded_element_count(&app), 1);
        assert!(app.selected_element.is_none());
        assert!(app.undo_stack.is_empty());
        assert_eq!(app.redo_stack.len(), 1);
    }

    #[test]
    fn add_path_element_adds_selects_and_marks_dirty() {
        let cell = cell_state_with_test_elements();
        let mut app = ViewerApp {
            cell: Some(cell),
            ..Default::default()
        };

        assert!(
            app.add_path_element(
                vec![p(1.0, 2.0), p(3.0, 4.0)],
                Layer::new(6),
                DataType::new(3),
                2.5,
                PathType::Round,
            )
            .is_ok()
        );

        assert_eq!(loaded_element_count(&app), 2);
        assert_eq!(app.selected_element, Some(1));
        assert_eq!(app.hovered_element, Some(1));
        assert!(app.has_unsaved_changes);
        assert_eq!(app.undo_stack.len(), 1);
        assert!(app.redo_stack.is_empty());

        let Some(Element::Path(path)) = app
            .cell
            .as_ref()
            .expect("cell should remain loaded")
            .elements
            .get(1)
        else {
            panic!("added element should be path");
        };
        assert_eq!(path.layer(), Layer::new(6));
        assert_eq!(path.data_type(), DataType::new(3));
        assert_eq!(path.path_type(), &Some(PathType::Round));
        assert_eq!(path.width(), Some(Unit::float(2.5, 1.0)));
        assert_eq!(path.points(), &[p(1.0, 2.0), p(3.0, 4.0)]);
    }

    #[test]
    fn add_path_element_rejects_less_than_two_points() {
        let cell = cell_state_with_test_elements();
        let mut app = ViewerApp {
            cell: Some(cell),
            ..Default::default()
        };

        assert_eq!(
            app.add_path_element(
                vec![p(1.0, 2.0)],
                Layer::new(6),
                DataType::new(3),
                2.5,
                PathType::Round,
            ),
            Err("Path requires at least two points".to_string())
        );
        assert_eq!(loaded_element_count(&app), 1);
    }

    #[test]
    fn path_tool_snaps_vertices_when_enabled() {
        let cell = cell_state_with_test_elements();
        let mut app = ViewerApp {
            cell: Some(cell),
            snap_to_grid: true,
            ..Default::default()
        };
        app.viewport.zoom = 10.0;
        app.begin_path_tool(6, 3, 2.5, PathType::Round);

        assert!(app.add_path_vertex(23.0, 37.0));

        let tool = app.path_tool.expect("path tool should remain active");
        assert_eq!(tool.points, vec![p(20.0, 40.0)]);
    }

    #[test]
    fn finish_path_tool_requires_two_points() {
        let cell = cell_state_with_test_elements();
        let mut app = ViewerApp {
            cell: Some(cell),
            ..Default::default()
        };
        app.begin_path_tool(6, 3, 2.5, PathType::Round);
        assert!(app.add_path_vertex(1.0, 1.0));

        assert!(!app.finish_path_tool());

        assert_eq!(loaded_element_count(&app), 1);
        assert_eq!(
            app.path_tool
                .as_ref()
                .and_then(|tool| tool.error.as_deref()),
            Some("Path requires at least two points")
        );
    }

    #[test]
    fn finish_path_tool_adds_path_with_target_settings() {
        let cell = cell_state_with_test_elements();
        let mut app = ViewerApp {
            cell: Some(cell),
            ..Default::default()
        };
        app.begin_path_tool(8, 5, 3.5, PathType::Overlap);
        assert!(app.add_path_vertex(1.0, 1.0));
        assert!(app.add_path_vertex(3.0, 3.0));

        assert!(app.finish_path_tool());

        assert!(app.path_tool.is_none());
        assert_eq!(loaded_element_count(&app), 2);
        let Some(Element::Path(path)) = app
            .cell
            .as_ref()
            .expect("cell should remain loaded")
            .elements
            .get(1)
        else {
            panic!("added element should be path");
        };
        assert_eq!(path.layer(), Layer::new(8));
        assert_eq!(path.data_type(), DataType::new(5));
        assert_eq!(path.path_type(), &Some(PathType::Overlap));
        assert_eq!(path.width(), Some(Unit::float(3.5, 1.0)));
    }

    #[test]
    fn undo_add_path_element_removes_it() {
        let cell = cell_state_with_test_elements();
        let mut app = ViewerApp {
            cell: Some(cell),
            ..Default::default()
        };

        assert!(
            app.add_path_element(
                vec![p(1.0, 2.0), p(3.0, 4.0)],
                Layer::new(6),
                DataType::new(3),
                2.5,
                PathType::Round,
            )
            .is_ok()
        );
        assert_eq!(loaded_element_count(&app), 2);

        assert!(app.undo_edit());

        assert_eq!(loaded_element_count(&app), 1);
        assert!(app.selected_element.is_none());
        assert!(app.undo_stack.is_empty());
        assert_eq!(app.redo_stack.len(), 1);
    }

    #[test]
    fn copy_selected_element_stores_element() {
        let cell = cell_state_with_test_elements();
        let mut app = ViewerApp {
            cell: Some(cell),
            selected_element: Some(0),
            ..Default::default()
        };

        assert!(app.copy_selected_element());

        assert!(app.clipboard_element.is_some());
    }

    #[test]
    fn paste_clipboard_element_inserts_at_cursor_and_records_undo() {
        let cell = cell_state_with_test_elements();
        let mut app = ViewerApp {
            cell: Some(cell),
            selected_element: Some(0),
            mouse_world_pos: Some((20.0, 30.0)),
            ..Default::default()
        };
        assert!(app.copy_selected_element());

        assert!(app.paste_clipboard_element());

        assert_eq!(loaded_element_count(&app), 2);
        assert_eq!(app.selected_element, Some(1));
        assert!(app.has_unsaved_changes);
        assert_eq!(app.undo_stack.len(), 1);
        let pasted_bbox = first_element_bbox_at(&app, 1);
        assert!((pasted_bbox.min_x - 15.0).abs() < 1e-12);
        assert!((pasted_bbox.min_y - 25.0).abs() < 1e-12);

        assert!(app.undo_edit());

        assert_eq!(loaded_element_count(&app), 1);
        assert!(app.selected_element.is_none());
        assert_eq!(app.redo_stack.len(), 1);
    }

    #[test]
    fn paste_clipboard_element_snaps_cursor_when_enabled() {
        let cell = cell_state_with_test_elements();
        let mut app = ViewerApp {
            cell: Some(cell),
            selected_element: Some(0),
            mouse_world_pos: Some((23.0, 37.0)),
            snap_to_grid: true,
            ..Default::default()
        };
        app.viewport.zoom = 10.0;
        assert!(app.copy_selected_element());

        assert!(app.paste_clipboard_element());

        let pasted_bbox = first_element_bbox_at(&app, 1);
        assert!((pasted_bbox.min_x - 15.0).abs() < 1e-12);
        assert!((pasted_bbox.min_y - 35.0).abs() < 1e-12);
    }

    #[test]
    fn paste_clipboard_element_can_paste_across_cells() {
        let cell = cell_state_with_test_elements();
        let mut app = ViewerApp {
            cell: Some(cell),
            selected_element: Some(0),
            mouse_world_pos: Some((20.0, 30.0)),
            ..Default::default()
        };
        assert!(app.copy_selected_element());
        assert!(app.create_cell_named("other").is_ok());

        assert!(app.paste_clipboard_element());

        let cell = app.cell.as_ref().expect("cell should remain loaded");
        assert_eq!(cell.selected_cell.as_deref(), Some("other"));
        assert_eq!(
            cell.library
                .get_cell("other")
                .expect("other cell should exist")
                .elements()
                .len(),
            1
        );
    }

    #[test]
    fn move_selected_element_snaps_center_when_enabled() {
        let cell = cell_state_with_test_elements();
        let mut app = ViewerApp {
            cell: Some(cell),
            selected_element: Some(0),
            snap_to_grid: true,
            ..Default::default()
        };
        app.viewport.zoom = 10.0;

        assert!(app.move_selected_element(Point::float(6.0, 6.0, 1.0)));

        let bbox = first_element_bbox(&app);
        assert!((bbox.min_x - 5.0).abs() < 1e-12);
        assert!((bbox.min_y - 5.0).abs() < 1e-12);
    }

    #[test]
    fn create_cell_named_adds_selects_and_marks_dirty() {
        let cell = cell_state_with_test_elements();
        let mut app = ViewerApp {
            cell: Some(cell),
            selected_element: Some(0),
            ..Default::default()
        };
        assert!(app.move_selected_element(Point::float(1.0, 0.0, 1.0)));

        assert!(app.create_cell_named("created").is_ok());

        let cell = app.cell.as_ref().expect("cell should remain loaded");
        assert!(cell.library.get_cell("created").is_some());
        assert_eq!(cell.selected_cell.as_deref(), Some("created"));
        assert!(cell.cell_names.contains(&"created".to_string()));
        assert!(cell.elements.is_empty());
        assert!(app.has_unsaved_changes);
        assert!(app.undo_stack.is_empty());
        assert!(app.redo_stack.is_empty());
    }

    #[test]
    fn rename_cell_named_updates_selection_references_and_marks_dirty() {
        let mut cell = CellState::new(library_with_referenced_leaf());
        cell.selected_cell = Some("leaf".to_string());
        assert!(cell.load_direct_cell_elements("leaf"));
        let mut app = ViewerApp {
            cell: Some(cell),
            ..Default::default()
        };

        assert!(app.rename_cell_named("leaf", "renamed").is_ok());

        let cell = app.cell.as_ref().expect("cell should remain loaded");
        assert!(cell.library.get_cell("leaf").is_none());
        assert!(cell.library.get_cell("renamed").is_some());
        assert_eq!(cell.selected_cell.as_deref(), Some("renamed"));
        assert_eq!(
            cell.library
                .get_cell("top")
                .expect("top cell should exist")
                .referenced_cell_names(),
            vec!["renamed"]
        );
        assert!(app.has_unsaved_changes);
    }

    #[test]
    fn rename_cell_named_rejects_duplicate_name() {
        let mut app = ViewerApp {
            cell: Some(CellState::new(library_with_referenced_leaf())),
            ..Default::default()
        };

        assert_eq!(
            app.rename_cell_named("leaf", "top"),
            Err("Cell name already exists".to_string())
        );
    }

    #[test]
    fn save_file_to_path_writes_library_and_clears_dirty_state() {
        let dir = tempfile::tempdir().expect("temporary directory should be created");
        let path = dir.path().join("saved.gds");
        let mut source_cell = Cell::new("top");
        source_cell.add(Polygon::new(
            [
                Point::integer(0, 0, DEFAULT_INTEGER_UNITS),
                Point::integer(10, 0, DEFAULT_INTEGER_UNITS),
                Point::integer(10, 10, DEFAULT_INTEGER_UNITS),
                Point::integer(0, 10, DEFAULT_INTEGER_UNITS),
            ],
            Layer::new(1),
            DataType::new(0),
        ));
        let mut library = Library::new("test");
        library.add_cell(source_cell);
        let mut cell = CellState::new(library);
        cell.selected_cell = Some("top".to_string());
        assert!(cell.load_direct_cell_elements("top"));
        let mut app = ViewerApp {
            cell: Some(cell),
            has_unsaved_changes: true,
            ..Default::default()
        };

        assert!(app.save_file_to_path(&path));

        assert!(!app.has_unsaved_changes);
        assert!(app.save_error.is_none());
        assert_eq!(app.file_load.file_path.as_deref(), Some(path.as_path()));

        let saved = Library::read_file(&path, Some(DEFAULT_INTEGER_UNITS))
            .expect("saved GDS should be readable");
        assert_eq!(
            saved
                .get_cell("top")
                .expect("top cell should exist")
                .elements()
                .len(),
            1
        );
    }

    #[test]
    fn save_current_file_without_loaded_library_sets_error() {
        let mut app = ViewerApp::default();

        assert!(!app.save_current_file());
        assert_eq!(app.save_error.as_deref(), Some("No GDS library loaded"));
    }

    #[test]
    fn dirty_close_request_opens_prompt_and_cancels_close() {
        let mut app = ViewerApp {
            has_unsaved_changes: true,
            ..Default::default()
        };

        assert!(app.cancel_close_for_unsaved_changes());
        assert!(app.show_unsaved_close_prompt);
    }

    #[test]
    fn clean_close_request_does_not_open_prompt() {
        let mut app = ViewerApp::default();

        assert!(!app.cancel_close_for_unsaved_changes());
        assert!(!app.show_unsaved_close_prompt);
    }

    #[test]
    fn select_cell_loads_direct_elements_without_flattening_references() {
        let mut leaf = Cell::new("leaf");
        leaf.add(test_elements().remove(0));

        let mut top = Cell::new("top");
        top.add(Reference::new("leaf"));

        let mut library = Library::new("test");
        library.add_cell(leaf);
        library.add_cell(top);

        let mut cell_state = CellState::new(library);
        cell_state.render_depth = 8;
        let mut app = ViewerApp {
            cell: Some(cell_state),
            ..Default::default()
        };

        app.select_cell("top");

        let cell = app.cell.expect("cell should be loaded");
        assert_eq!(cell.elements.len(), 1);
        assert!(matches!(cell.elements[0], Element::Reference(_)));
        assert_eq!(
            cell.layers,
            std::collections::BTreeSet::from([(Layer::new(1), DataType::new(0))])
        );
    }
}
