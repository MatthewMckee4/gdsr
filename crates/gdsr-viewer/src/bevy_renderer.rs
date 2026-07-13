use std::collections::{BTreeSet, HashMap};

use bevy::asset::RenderAssetUsages;
use bevy::camera::{CameraOutputMode, Viewport, visibility::RenderLayers};
use bevy::math::{DAffine2, DVec2};
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::BlendState;
use bevy::sprite::Anchor;
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::{EguiGlobalSettings, PrimaryEguiContext};
use gdsr::{DataType, Layer};

use crate::app::ViewerApp;
use crate::bevy_scene::{PreparedGeometry, PreparedLibrary, SceneOptions, ScenePlan};
use crate::drawable::WorldBBox;

#[derive(Component)]
pub struct GdsWorldCamera;

#[derive(Component)]
pub struct GdsSceneEntity;

#[derive(Resource, Default)]
pub struct BevyRendererState {
    prepared: Option<PreparedLibrary>,
    prepared_cell: Option<String>,
    geometry_generation: Option<u64>,
    render_generation: Option<u64>,
    selected_cell: Option<String>,
    render_depth: u32,
    hidden_layers: Vec<(Layer, DataType)>,
    render_origin: DVec2,
    render_zoom: f64,
    viewport_size: Vec2,
    mesh_handles: Vec<Handle<Mesh>>,
    material_handles: Vec<Handle<ColorMaterial>>,
}

struct LayerAssets {
    fill: Handle<ColorMaterial>,
    stroke: Handle<ColorMaterial>,
    color: Color,
}

pub fn setup(mut commands: Commands, mut egui_settings: ResMut<EguiGlobalSettings>) {
    egui_settings.auto_create_primary_context = false;
    commands.insert_resource(BevyRendererState::default());
    commands.spawn((GdsWorldCamera, Camera2d));
    commands.spawn((
        PrimaryEguiContext,
        Camera2d,
        RenderLayers::none(),
        Camera {
            order: 1,
            output_mode: CameraOutputMode::Write {
                blend_state: Some(BlendState::ALPHA_BLENDING),
                clear_color: ClearColorConfig::None,
            },
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        },
    ));
}

pub fn sync_scene(
    mut commands: Commands,
    mut viewer: NonSendMut<ViewerApp>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cameras: Query<(&mut Camera, &mut Transform, &mut Projection), With<GdsWorldCamera>>,
    scene_entities: Query<Entity, With<GdsSceneEntity>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut state: ResMut<BevyRendererState>,
) {
    let Some(window) = windows.iter().next() else {
        return;
    };
    let rect = viewer.viewport_rect();
    if !rect.is_positive() {
        return;
    }

    let (center_x, center_y, zoom) = viewer.viewport_state();
    let selected_cell = viewer.selected_cell_name().map(str::to_owned);

    let Some(selected_cell_name) = selected_cell.as_deref() else {
        clear_scene(
            &mut commands,
            &scene_entities,
            &mut meshes,
            &mut materials,
            &mut state,
        );
        update_camera(window, rect, center_x, center_y, zoom, &state, &mut cameras);
        state.selected_cell = None;
        return;
    };

    let geometry_generation = viewer.geometry_generation();
    if state.geometry_generation != Some(geometry_generation)
        || state.prepared_cell.as_deref() != Some(selected_cell_name)
    {
        let prepared = viewer
            .library()
            .map(|library| PreparedLibrary::build(library, selected_cell_name));
        match prepared {
            Some(Ok(prepared)) => state.prepared = Some(prepared),
            Some(Err(error)) => {
                log::error!("failed to prepare Bevy scene: {error:?}");
                state.prepared = None;
            }
            None => state.prepared = None,
        }
        state.geometry_generation = Some(geometry_generation);
        state.prepared_cell = Some(selected_cell_name.to_string());
    }

    let render_generation = viewer.render_generation();
    let render_depth = viewer.render_depth();
    let mut hidden_layers: Vec<_> = viewer.hidden_layers().iter().copied().collect();
    hidden_layers.sort_unstable();
    let viewport_size = Vec2::new(rect.width(), rect.height());
    let rebuild = should_rebuild_scene(
        &state,
        render_generation,
        selected_cell_name,
        render_depth,
        &hidden_layers,
        center_x,
        center_y,
        zoom,
        viewport_size,
    );

    if rebuild {
        let visible = expanded_visible_rect(center_x, center_y, zoom, viewport_size);
        let plan = state.prepared.as_ref().map(|prepared| {
            prepared.plan(
                selected_cell_name,
                &SceneOptions {
                    visible: &visible,
                    zoom,
                    depth: render_depth,
                    hidden_layers: viewer.hidden_layers(),
                },
            )
        });

        clear_scene(
            &mut commands,
            &scene_entities,
            &mut meshes,
            &mut materials,
            &mut state,
        );
        if let Some(plan) = plan
            && let Some(prepared) = state.prepared.take()
        {
            spawn_plan(
                &mut commands,
                &mut viewer,
                &mut meshes,
                &mut materials,
                &mut state,
                &prepared,
                &plan,
                DVec2::new(center_x, center_y),
                zoom,
            );
            state.prepared = Some(prepared);
        }

        state.render_generation = Some(render_generation);
        state.selected_cell = selected_cell;
        state.render_depth = render_depth;
        state.hidden_layers = hidden_layers;
        state.render_origin = DVec2::new(center_x, center_y);
        state.render_zoom = zoom;
        state.viewport_size = viewport_size;
    }

    update_camera(window, rect, center_x, center_y, zoom, &state, &mut cameras);
}

fn should_rebuild_scene(
    state: &BevyRendererState,
    render_generation: u64,
    selected_cell: &str,
    render_depth: u32,
    hidden_layers: &[(Layer, DataType)],
    center_x: f64,
    center_y: f64,
    zoom: f64,
    viewport_size: Vec2,
) -> bool {
    if state.prepared.is_none()
        || state.render_generation != Some(render_generation)
        || state.selected_cell.as_deref() != Some(selected_cell)
        || state.render_depth != render_depth
        || state.hidden_layers != hidden_layers
        || viewport_size.x > state.viewport_size.x * 2.0
        || viewport_size.y > state.viewport_size.y * 2.0
        || !state.render_zoom.is_finite()
        || state.render_zoom <= 0.0
    {
        return true;
    }

    let zoom_ratio = zoom / state.render_zoom;
    if !(0.5..=2.0).contains(&zoom_ratio) {
        return true;
    }
    let pan_x = (center_x - state.render_origin.x).abs() * zoom;
    let pan_y = (center_y - state.render_origin.y).abs() * zoom;
    let margin_x = (f64::from(state.viewport_size.x) * 1.5 * zoom_ratio
        - f64::from(viewport_size.x) * 0.5)
        .max(0.0)
        * 0.8;
    let margin_y = (f64::from(state.viewport_size.y) * 1.5 * zoom_ratio
        - f64::from(viewport_size.y) * 0.5)
        .max(0.0)
        * 0.8;
    pan_x > margin_x || pan_y > margin_y
}

fn expanded_visible_rect(
    center_x: f64,
    center_y: f64,
    zoom: f64,
    viewport_size: Vec2,
) -> WorldBBox {
    let half_width = f64::from(viewport_size.x) * 1.5 / zoom;
    let half_height = f64::from(viewport_size.y) * 1.5 / zoom;
    WorldBBox::new(
        center_x - half_width,
        center_y - half_height,
        center_x + half_width,
        center_y + half_height,
    )
}

fn clear_scene(
    commands: &mut Commands,
    scene_entities: &Query<Entity, With<GdsSceneEntity>>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    state: &mut BevyRendererState,
) {
    for entity in scene_entities {
        commands.entity(entity).despawn();
    }
    for handle in state.mesh_handles.drain(..) {
        meshes.remove(handle.id());
    }
    for handle in state.material_handles.drain(..) {
        materials.remove(handle.id());
    }
}

fn spawn_plan(
    commands: &mut Commands,
    viewer: &mut ViewerApp,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    state: &mut BevyRendererState,
    prepared: &PreparedLibrary,
    plan: &ScenePlan,
    render_origin: DVec2,
    render_zoom: f64,
) {
    let mut layers = BTreeSet::new();
    for instance in &plan.geometry_instances {
        layers.insert(prepared.geometries[instance.geometry_id].layer);
    }
    layers.extend(plan.texts.iter().map(|text| text.layer));
    layers.extend(plan.node_points.keys().copied());

    let mut layer_materials = HashMap::with_capacity(layers.len());
    for layer in layers {
        let color = viewer.layer_color(layer.0, layer.1);
        let fill = materials.add(ColorMaterial::from_color(Color::srgba_u8(
            color.r(),
            color.g(),
            color.b(),
            80,
        )));
        let stroke = materials.add(ColorMaterial::from_color(Color::srgb_u8(
            color.r(),
            color.g(),
            color.b(),
        )));
        state
            .material_handles
            .extend([fill.clone(), stroke.clone()]);
        layer_materials.insert(
            layer,
            LayerAssets {
                fill,
                stroke,
                color: Color::srgb_u8(color.r(), color.g(), color.b()),
            },
        );
    }

    let used_geometry_ids: BTreeSet<_> = plan
        .geometry_instances
        .iter()
        .map(|instance| instance.geometry_id)
        .collect();
    let mut geometry_handles = HashMap::with_capacity(used_geometry_ids.len());
    for geometry_id in used_geometry_ids {
        let geometry = &prepared.geometries[geometry_id];
        let handles = upload_geometry(meshes, state, geometry, render_zoom);
        geometry_handles.insert(geometry_id, handles);
    }

    for instance in &plan.geometry_instances {
        let geometry = &prepared.geometries[instance.geometry_id];
        let Some((fill_mesh, line_mesh)) = geometry_handles.get(&instance.geometry_id) else {
            continue;
        };
        let Some(layer_assets) = layer_materials.get(&geometry.layer) else {
            continue;
        };
        let base_depth = layer_depth(geometry.layer);
        let transform = instance_transform(
            instance.transform,
            geometry.origin,
            render_origin,
            render_zoom,
            base_depth,
        );
        if let Some(fill_mesh) = fill_mesh {
            commands.spawn((
                GdsSceneEntity,
                Mesh2d(fill_mesh.clone()),
                MeshMaterial2d(layer_assets.fill.clone()),
                transform,
            ));
        }
        if let Some(line_mesh) = line_mesh {
            let mut line_transform = transform;
            line_transform.translation.z += 0.0001;
            commands.spawn((
                GdsSceneEntity,
                Mesh2d(line_mesh.clone()),
                MeshMaterial2d(layer_assets.stroke.clone()),
                line_transform,
            ));
        }
    }

    spawn_load_proxies(
        commands,
        meshes,
        materials,
        state,
        plan,
        render_origin,
        render_zoom,
    );
    spawn_nodes(
        commands,
        meshes,
        state,
        plan,
        &layer_materials,
        render_origin,
        render_zoom,
    );
    spawn_texts(commands, plan, &layer_materials, render_origin, render_zoom);
}

fn upload_geometry(
    meshes: &mut Assets<Mesh>,
    state: &mut BevyRendererState,
    geometry: &PreparedGeometry,
    render_zoom: f64,
) -> (Option<Handle<Mesh>>, Option<Handle<Mesh>>) {
    let positions: Vec<[f32; 3]> = geometry
        .positions
        .iter()
        .map(|position| {
            let position = (*position - geometry.origin) * render_zoom;
            [position.x as f32, position.y as f32, 0.0]
        })
        .collect();
    let fill = make_mesh(
        PrimitiveTopology::TriangleList,
        positions.clone(),
        geometry.triangle_indices.clone(),
    )
    .map(|mesh| meshes.add(mesh));
    let lines = make_mesh(
        PrimitiveTopology::LineList,
        positions,
        geometry.line_indices.clone(),
    )
    .map(|mesh| meshes.add(mesh));
    state.mesh_handles.extend(fill.iter().cloned());
    state.mesh_handles.extend(lines.iter().cloned());
    (fill, lines)
}

fn make_mesh(
    topology: PrimitiveTopology,
    positions: Vec<[f32; 3]>,
    indices: Vec<u32>,
) -> Option<Mesh> {
    if positions.is_empty() || indices.is_empty() {
        return None;
    }
    Some(
        Mesh::new(topology, RenderAssetUsages::RENDER_WORLD)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
            .with_inserted_indices(Indices::U32(indices)),
    )
}

fn instance_transform(
    transform: DAffine2,
    geometry_origin: DVec2,
    render_origin: DVec2,
    render_zoom: f64,
    depth: f32,
) -> Transform {
    let world_origin = transform.transform_point2(geometry_origin);
    let translation = (world_origin - render_origin) * render_zoom;
    let (scale, angle, _) = transform.to_scale_angle_translation();
    Transform {
        translation: Vec3::new(translation.x as f32, translation.y as f32, depth),
        rotation: Quat::from_rotation_z(angle as f32),
        scale: Vec3::new(scale.x as f32, scale.y as f32, 1.0),
    }
}

fn layer_depth(layer: (Layer, DataType)) -> f32 {
    f32::from(layer.0.value()) * 0.001 + f32::from(layer.1.value()) * 0.000_001
}

fn scene_position(position: DVec2, render_origin: DVec2, render_zoom: f64) -> Vec2 {
    let position = (position - render_origin) * render_zoom;
    Vec2::new(position.x as f32, position.y as f32)
}

fn proxy_screen_bounds(bbox: WorldBBox, render_origin: DVec2, render_zoom: f64) -> (Vec2, Vec2) {
    let mut min = scene_position(
        DVec2::new(bbox.min_x, bbox.min_y),
        render_origin,
        render_zoom,
    );
    let mut max = scene_position(
        DVec2::new(bbox.max_x, bbox.max_y),
        render_origin,
        render_zoom,
    );
    if max.x - min.x < 4.0 {
        let center = f32::midpoint(min.x, max.x);
        min.x = center - 2.0;
        max.x = center + 2.0;
    }
    if max.y - min.y < 4.0 {
        let center = f32::midpoint(min.y, max.y);
        min.y = center - 2.0;
        max.y = center + 2.0;
    }
    (min, max)
}

fn spawn_load_proxies(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    state: &mut BevyRendererState,
    plan: &ScenePlan,
    render_origin: DVec2,
    render_zoom: f64,
) {
    if plan.load_proxies.is_empty() {
        return;
    }
    let mut positions = Vec::with_capacity(plan.load_proxies.len() * 4);
    let mut triangles = Vec::with_capacity(plan.load_proxies.len() * 6);
    let mut lines = Vec::with_capacity(plan.load_proxies.len() * 8);
    for proxy in &plan.load_proxies {
        let Ok(base) = u32::try_from(positions.len()) else {
            break;
        };
        let (min, max) = proxy_screen_bounds(proxy.bbox, render_origin, render_zoom);
        positions.extend([
            [min.x, min.y, 0.0],
            [max.x, min.y, 0.0],
            [max.x, max.y, 0.0],
            [min.x, max.y, 0.0],
        ]);
        triangles.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        lines.extend([
            base,
            base + 1,
            base + 1,
            base + 2,
            base + 2,
            base + 3,
            base + 3,
            base,
        ]);
    }

    let fill_material = materials.add(ColorMaterial::from_color(Color::srgba_u8(
        180, 180, 180, 25,
    )));
    let stroke_material = materials.add(ColorMaterial::from_color(Color::srgb_u8(180, 180, 180)));
    state
        .material_handles
        .extend([fill_material.clone(), stroke_material.clone()]);
    if let Some(mesh) = make_mesh(
        PrimitiveTopology::TriangleList,
        positions.clone(),
        triangles,
    ) {
        let handle = meshes.add(mesh);
        state.mesh_handles.push(handle.clone());
        commands.spawn((
            GdsSceneEntity,
            Mesh2d(handle),
            MeshMaterial2d(fill_material),
            Transform::from_xyz(0.0, 0.0, 100.0),
        ));
    }
    if let Some(mesh) = make_mesh(PrimitiveTopology::LineList, positions, lines) {
        let handle = meshes.add(mesh);
        state.mesh_handles.push(handle.clone());
        commands.spawn((
            GdsSceneEntity,
            Mesh2d(handle),
            MeshMaterial2d(stroke_material),
            Transform::from_xyz(0.0, 0.0, 100.001),
        ));
    }

    for proxy in plan.load_proxies.iter().take(2_048) {
        let width = (proxy.bbox.max_x - proxy.bbox.min_x).abs() * render_zoom;
        let height = (proxy.bbox.max_y - proxy.bbox.min_y).abs() * render_zoom;
        let Some(label) = proxy.label.as_deref() else {
            continue;
        };
        if width < 40.0 || height < 20.0 {
            continue;
        }
        let center = scene_position(
            DVec2::new(
                f64::midpoint(proxy.bbox.min_x, proxy.bbox.max_x),
                f64::midpoint(proxy.bbox.min_y, proxy.bbox.max_y),
            ),
            render_origin,
            render_zoom,
        );
        commands.spawn((
            GdsSceneEntity,
            Text2d::new(label),
            TextFont::from_font_size(12.0),
            TextColor(Color::srgb_u8(180, 180, 180)),
            Anchor::CENTER,
            Transform::from_xyz(center.x, center.y, 100.002),
        ));
    }
}

fn spawn_nodes(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    state: &mut BevyRendererState,
    plan: &ScenePlan,
    layer_materials: &HashMap<(Layer, DataType), LayerAssets>,
    render_origin: DVec2,
    render_zoom: f64,
) {
    for (layer, points) in &plan.node_points {
        let mut positions = Vec::with_capacity(points.len() * 4);
        let mut triangles = Vec::with_capacity(points.len() * 6);
        for point in points {
            let Ok(base) = u32::try_from(positions.len()) else {
                break;
            };
            let point = scene_position(*point, render_origin, render_zoom);
            let half_size = 3.0;
            positions.extend([
                [point.x - half_size, point.y - half_size, 0.0],
                [point.x + half_size, point.y - half_size, 0.0],
                [point.x + half_size, point.y + half_size, 0.0],
                [point.x - half_size, point.y + half_size, 0.0],
            ]);
            triangles.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        }
        let Some(mesh) = make_mesh(PrimitiveTopology::TriangleList, positions, triangles) else {
            continue;
        };
        let Some(layer_assets) = layer_materials.get(layer) else {
            continue;
        };
        let handle = meshes.add(mesh);
        state.mesh_handles.push(handle.clone());
        commands.spawn((
            GdsSceneEntity,
            Mesh2d(handle),
            MeshMaterial2d(layer_assets.stroke.clone()),
            Transform::from_xyz(0.0, 0.0, layer_depth(*layer) + 0.0002),
        ));
    }
}

fn spawn_texts(
    commands: &mut Commands,
    plan: &ScenePlan,
    layer_materials: &HashMap<(Layer, DataType), LayerAssets>,
    render_origin: DVec2,
    render_zoom: f64,
) {
    for text in plan.texts.iter().take(10_000) {
        let Some(layer_assets) = layer_materials.get(&text.layer) else {
            continue;
        };
        let position = scene_position(text.position, render_origin, render_zoom);
        commands.spawn((
            GdsSceneEntity,
            Text2d::new(text.value.as_ref()),
            TextFont::from_font_size(12.0),
            TextColor(layer_assets.color),
            Anchor::BOTTOM_LEFT,
            Transform::from_xyz(position.x, position.y, layer_depth(text.layer) + 0.0002),
        ));
    }
}

fn update_camera(
    window: &Window,
    rect: egui::Rect,
    center_x: f64,
    center_y: f64,
    zoom: f64,
    state: &BevyRendererState,
    cameras: &mut Query<(&mut Camera, &mut Transform, &mut Projection), With<GdsWorldCamera>>,
) {
    let scale_factor = window.scale_factor();
    let position = UVec2::new(
        (rect.min.x * scale_factor).max(0.0) as u32,
        (rect.min.y * scale_factor).max(0.0) as u32,
    );
    let available = UVec2::new(
        window.physical_width().saturating_sub(position.x).max(1),
        window.physical_height().saturating_sub(position.y).max(1),
    );
    let size = UVec2::new(
        (rect.width() * scale_factor).max(1.0) as u32,
        (rect.height() * scale_factor).max(1.0) as u32,
    )
    .min(available);
    let (camera_position, projection_scale) = camera_frame(state, center_x, center_y, zoom);

    for (mut camera, mut transform, mut projection) in cameras.iter_mut() {
        camera.viewport = Some(Viewport {
            physical_position: position,
            physical_size: size,
            ..default()
        });
        transform.translation.x = camera_position.x as f32;
        transform.translation.y = camera_position.y as f32;
        if let Projection::Orthographic(projection) = projection.as_mut() {
            projection.scale = projection_scale as f32;
        }
    }
}

fn camera_frame(
    state: &BevyRendererState,
    center_x: f64,
    center_y: f64,
    zoom: f64,
) -> (DVec2, f64) {
    let render_zoom = if state.render_zoom.is_finite() && state.render_zoom > 0.0 {
        state.render_zoom
    } else if zoom.is_finite() && zoom > 0.0 {
        zoom
    } else {
        1.0
    };
    let camera_position = (DVec2::new(center_x, center_y) - state.render_origin) * render_zoom;
    let projection_scale = if zoom.is_finite() && zoom > 0.0 {
        render_zoom / zoom
    } else {
        1.0
    };
    (camera_position, projection_scale)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gdsr::{Cell, Library};

    fn renderer_state() -> BevyRendererState {
        let mut library = Library::new("test");
        library.add_cell(Cell::new("top"));
        BevyRendererState {
            prepared: Some(PreparedLibrary::build(&library, "top").expect("scene should prepare")),
            prepared_cell: Some("top".to_string()),
            render_generation: Some(1),
            selected_cell: Some("top".to_string()),
            render_depth: 5,
            render_origin: DVec2::ZERO,
            render_zoom: 10.0,
            viewport_size: Vec2::splat(100.0),
            ..default()
        }
    }

    fn should_rebuild(state: &BevyRendererState, center_x: f64, zoom: f64, size: Vec2) -> bool {
        should_rebuild_scene(state, 1, "top", 5, &[], center_x, 0.0, zoom, size)
    }

    #[test]
    fn camera_changes_inside_scene_margin_do_not_rebuild() {
        let state = renderer_state();

        assert!(!should_rebuild(&state, 3.9, 10.0, Vec2::splat(199.0)));
        assert!(!should_rebuild(&state, 3.9, 5.0, Vec2::splat(100.0)));
        assert!(!should_rebuild(&state, 0.0, 19.9, Vec2::splat(100.0)));
    }

    #[test]
    fn camera_changes_outside_scene_margin_rebuild() {
        let state = renderer_state();

        assert!(should_rebuild(&state, 8.1, 10.0, Vec2::splat(100.0)));
        assert!(should_rebuild(&state, 5.0, 5.0, Vec2::splat(100.0)));
        assert!(should_rebuild(&state, 0.0, 20.1, Vec2::splat(100.0)));
        assert!(should_rebuild(&state, 0.0, 10.0, Vec2::splat(201.0)));
    }

    #[test]
    fn instance_transform_preserves_reflection_and_rotation() {
        let affine = DAffine2::from_scale_angle_translation(
            DVec2::new(2.0, -3.0),
            std::f64::consts::FRAC_PI_3,
            DVec2::new(5.0, 7.0),
        );
        let geometry_origin = DVec2::new(11.0, 13.0);
        let render_origin = DVec2::new(2.0, 3.0);
        let render_zoom = 4.0;
        let local_point = DVec2::new(1.5, -2.5);
        let transform =
            instance_transform(affine, geometry_origin, render_origin, render_zoom, 0.0);
        let actual = transform.transform_point(Vec3::new(
            (local_point.x * render_zoom) as f32,
            (local_point.y * render_zoom) as f32,
            0.0,
        ));
        let expected =
            (affine.transform_point2(geometry_origin + local_point) - render_origin) * render_zoom;

        assert!((f64::from(actual.x) - expected.x).abs() < 1.0e-4);
        assert!((f64::from(actual.y) - expected.y).abs() < 1.0e-4);
    }

    #[test]
    fn point_proxy_has_visible_screen_area() {
        let (min, max) = proxy_screen_bounds(WorldBBox::new(1.0, 2.0, 1.0, 2.0), DVec2::ZERO, 10.0);

        assert_eq!(max - min, Vec2::splat(4.0));
    }

    #[test]
    fn empty_scene_uses_live_zoom_for_valid_camera_projection() {
        let (position, scale) = camera_frame(&BevyRendererState::default(), 2.0, 3.0, 4.0);

        assert_eq!(position, DVec2::new(8.0, 12.0));
        assert_eq!(scale, 1.0);
    }
}
