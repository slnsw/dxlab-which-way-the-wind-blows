use dbscan::{cluster, Classification};
use geo::algorithm::centroid::Centroid;
use geo::algorithm::concave_hull::ConcaveHull;
use geo::algorithm::simplify::Simplify;
use geo::{Coordinate, LineString, Polygon};
use nannou::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct NodeGroupDataItem {
    key: String,
    index: usize,
    day_values: Vec<usize>,
    display_values: Vec<usize>,
}

#[derive(Serialize, Deserialize)]
struct NodeGroupData {
    groups: Vec<NodeGroupDataItem>,
    start_date: String,
    end_date: String,
}

fn main() {
    nannou::app(model).update(update).run();
}

struct Node {
    pub x: f32,
    pub y: f32,
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    radius: f32, // Radius of impact
    ramp: f32,   // Influences the shape of the function
    pub damping: f32,
    pub velocity: Vector2,
    max_velocity: f32,
    id: usize,
}

struct NodeGroup {
    pub id: usize,
    pub size: usize,
    pub label: String,
    nodes: Vec<Node>,
    spring_connections: Vec<Spring>,
    pub convex_hulls: Vec<LineString<f32>>,
    pub day_values: Vec<usize>,
    pub display_values: Vec<usize>,
}

impl NodeGroup {
    fn new(data: &NodeGroupDataItem, settings: &SimSettings) -> Self {
        let index_f: f32 = cast(data.index).unwrap();
        // TODO: get from settings
        let angle = (index_f / 11.0) * (2.0 * PI);

        let d = random_range(150.0, 250.0);
        let y = angle.cos() * d;
        let x = angle.sin() * d;

        let nodes = (0..data.day_values[0])
            .map(|id| {
                let na = random_range(0.0, 2.0 * PI);
                let nd = random_range(5.0, 50.0);
                let nx = x + (na.cos() * nd);
                let ny = y + (na.sin() * nd);
                Node::new(
                    nx,
                    ny,
                    settings.repel_rect.left(),
                    settings.repel_rect.right(),
                    settings.repel_rect.bottom(),
                    settings.repel_rect.top(),
                    settings.node_radius,
                    settings.node_attract_ramp,
                    settings.node_attract_damping,
                    settings.node_max_velocity,
                    id,
                )
            })
            .collect::<Vec<Node>>();

        let spring_connections = create_connections(
            nodes.len(),
            settings.spring_length,
            settings.spring_stiffness,
            settings.spring_damping,
        );

        NodeGroup {
            id: data.index,
            label: data.key.clone(),
            size: nodes.len(),
            nodes,
            spring_connections,
            convex_hulls: Vec::new(),
            day_values: data.day_values.clone(),
            display_values: data.display_values.clone(),
        }
    }

    fn set_size(&mut self, new_size: usize, settings: &SimSettings) {
        let cur_size: i64 = cast(self.size).unwrap();
        let new_size_i: i64 = cast(new_size).unwrap();
        let mut delta = cur_size - new_size_i;
        if delta < 0 {
            delta = -delta;
        }

        if new_size < self.size {
            for _ in 0..delta {
                let index = random_range(0, self.nodes.len());
                self.nodes.remove(index);
            }
        } else {
            // Realign the ids
            for i in 0..self.nodes.len() {
                self.nodes[i].id = i;
            }

            let base_id: i64 = cast(self.nodes.len()).unwrap();

            // Pick a random node and pile them on here
            // if starting from 0 do from a random spot
            let target_index = if self.nodes.len() > 0 {
                random_range(0, self.nodes.len())
            } else {
                0
            };

            let x = if self.size > 0 {
                self.nodes[target_index].x
            } else {
                random_range(-200.0, 200.0)
            };

            let y = if self.size > 0 {
                self.nodes[target_index].y
            } else {
                random_range(-200.0, 200.0)
            };

            // Add our new nodes
            for i in 0..delta {
                let id = base_id + i;
                let id_size: usize = cast(id).unwrap();
                let n = Node::new(
                    x,
                    y,
                    settings.repel_rect.left(),
                    settings.repel_rect.right(),
                    settings.repel_rect.bottom(),
                    settings.repel_rect.top(),
                    settings.node_radius,
                    settings.node_attract_ramp,
                    settings.node_attract_damping,
                    settings.node_max_velocity,
                    id_size,
                );
                self.nodes.push(n);
            }
        }
        self.size = new_size;
        self.spring_connections = create_connections(
            self.nodes.len(),
            settings.spring_length,
            settings.spring_stiffness,
            settings.spring_damping,
        )
    }
}

impl Node {
    fn new(
        x: f32,
        y: f32,
        min_x: f32,
        max_x: f32,
        min_y: f32,
        max_y: f32,
        radius: f32,
        ramp: f32,
        damping: f32,
        max_velocity: f32,
        id: usize,
    ) -> Self {
        Node {
            x,
            y,
            min_x,
            max_x,
            min_y,
            max_y,
            radius,
            ramp,
            damping,
            max_velocity,
            velocity: vec2(0.0, 0.0),
            id,
        }
    }

    fn update(&mut self) {
        self.velocity = self.velocity.limit_magnitude(self.max_velocity);

        self.x += self.velocity.x;
        self.y += self.velocity.y;

        let lenience = 32.0;

        if self.x < self.min_x {
            let amt_over = self.x - self.min_x;
            let norm = f32::min(1.0, -(amt_over / lenience));
            let vel_delta = f32::max(self.velocity.x * norm, -0.1);
            self.velocity.x -= vel_delta;
        }
        if self.x > self.max_x {
            let amt_over = self.x - self.max_x;
            let norm = f32::min(1.0, amt_over / lenience);
            let vel_delta = f32::max(self.velocity.x * norm, 0.1);
            self.velocity.x -= vel_delta;
        }

        if self.y < self.min_y {
            let amt_over = self.y - self.min_y;
            let norm = f32::min(1.0, -(amt_over / lenience));
            let vel_delta = f32::max(self.velocity.x * norm, 0.1);
            self.velocity.y -= vel_delta;
        }
        if self.y > self.max_y {
            let amt_over = self.y - self.max_y;
            let norm = f32::min(1.0, amt_over / lenience);
            let vel_delta = f32::max(self.velocity.x * norm, 0.1);
            self.velocity.y -= vel_delta;
        }

        self.velocity *= 1.0 - self.damping;
    }
}

struct Model {
    node_groups: Vec<NodeGroup>,
    // Options
    day: usize,
    settings: SimSettings,
    frame: u32,
    map_texture: wgpu::Texture,
    display_font: nannou::text::Font,
}

impl Model {
    fn set_day(&mut self, day: usize) {
        self.day = day;

        for g in 0..self.node_groups.len() {
            let val = self.node_groups[g].day_values[day];
            self.node_groups[g].set_size(val, &self.settings)
        }
    }
}

struct SimSettings {
    node_radius: f32,
    node_max_velocity: f32,
    node_attract_ramp: f32,
    node_attract_strength: f32,
    node_attract_strength_friendly: f32,
    node_attract_damping: f32,
    spring_length: f32,
    spring_stiffness: f32,
    spring_damping: f32,
    repel_rect: Rect,
    frame_rate: u32,
    day_seconds: u32,
    stabilize_time: u32,
    start_date: String,
    end_date: String,
}

struct Spring {
    from: usize,
    to: usize,
    length: f32,
    stiffness: f32,
    damping: f32,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(800, 800)
        .view(view)
        .mouse_released(mouse_released)
        .key_pressed(key_pressed)
        .build()
        .unwrap();

    let assets = app.assets_path().unwrap();

    let img_path = assets.join("map.png");
    let map_texture = wgpu::Texture::from_path(app, img_path).unwrap();

    let font_path = assets.join("VCR_OSD_MONO_1.001.ttf");
    let display_font: nannou::text::Font = nannou::text::font::from_file(font_path).unwrap();

    let node_group_data = read_node_data("../data.json").unwrap();

    let settings = SimSettings {
        node_radius: 120.0,
        node_attract_ramp: 1.4,
        node_attract_strength: -1.5,
        node_attract_strength_friendly: -1.0,
        node_attract_damping: 0.9,
        node_max_velocity: 2.0,
        spring_length: 120.0 * 1.2,
        spring_stiffness: 4.3,
        spring_damping: 0.9,
        repel_rect: app.window_rect().pad(32.0),
        frame_rate: 20,
        day_seconds: 5,
        stabilize_time: 10,
        start_date: node_group_data.start_date,
        end_date: node_group_data.end_date,
    };

    fs::create_dir_all(format!("./out/{}", settings.start_date)).unwrap();

    let node_groups = node_group_data
        .groups
        .iter()
        .map(|data| NodeGroup::new(&data, &settings))
        .collect();

    Model {
        // Config
        settings,
        // Scene
        node_groups,
        day: 0,
        frame: 0,
        map_texture,
        display_font,
    }
}

fn read_node_data<P: AsRef<Path>>(path: P) -> Result<NodeGroupData> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let u = serde_json::from_reader(reader)?;
    Ok(u)
}

fn create_connections(
    group_size: usize,
    spring_length: f32,
    spring_stiffness: f32,
    spring_damping: f32,
) -> Vec<Spring> {
    let group_start = 0;
    (1..group_size)
        .map(|j| {
            // let buddy = random_range(group_start, group_start + group_size);
            Spring {
                from: group_start,
                to: group_start + j,
                length: spring_length,
                stiffness: spring_stiffness,
                damping: spring_damping,
            }
        })
        .collect::<Vec<Spring>>()
}

fn gravity(groups: &mut Vec<NodeGroup>) {
    let target = vec2(0.0, 0.0);
    let strength = -3.0;
    for g in 0..groups.len() {
        for n in 0..groups[g].nodes.len() {
            let current_node = &groups[g].nodes[n];
            let current_node_vector = vec2(current_node.x, current_node.y);
            let d = current_node_vector.distance(target);
            let s = (d / current_node.radius).powf(1.0 / current_node.ramp);
            let f = s * 9.0 * strength * (1.0 / (s + 1.0) + ((s - 3.0) / 4.0)) / d;
            let df = (current_node_vector - target) * f;

            groups[g].nodes[n].velocity += df;
        }
    }
}

fn attract_nodes(
    groups: &mut Vec<NodeGroup>,
    g: usize,
    og: usize,
    friendly_strength: f32,
    strength: f32,
) {
    let strength = if groups[g].id == groups[og].id {
        friendly_strength
    } else {
        strength
    };

    for t in 0..groups[g].nodes.len() {
        for o in 0..groups[og].nodes.len() {
            // If we're the same group, AND the same node id, we can skip.
            if groups[g].id == groups[og].id && groups[g].nodes[t].id == groups[og].nodes[o].id {
                continue;
            }

            let df = attract(&groups[g].nodes[t], &groups[og].nodes[o], strength);

            groups[og].nodes[o].velocity += df;
        }
    }
}

fn attract(current_node: &Node, other_node: &Node, strength: f32) -> Vector2 {
    let current_node_vector = vec2(current_node.x, current_node.y);
    let other_node_vector = vec2(other_node.x, other_node.y);
    let d = current_node_vector.distance(other_node_vector);

    if d > 0.0 && d < current_node.radius {
        let s = (d / current_node.radius).powf(1.0 / current_node.ramp);
        let f = s * 9.0 * strength * (1.0 / (s + 1.0) + ((s - 3.0) / 4.0)) / d;
        let mut df = current_node_vector - other_node_vector;
        df *= f;
        df
    } else {
        vec2(0.0, 0.0)
    }
}

// ------ apply forces on spring and attached nodes ------
fn spring(nodes: &mut Vec<Node>, spring_connection: &Spring) {
    let length = spring_connection.length;
    let stiffness = spring_connection.stiffness;
    let damping = spring_connection.damping;

    let mut diff = vec2(nodes[spring_connection.to].x, nodes[spring_connection.to].y)
        - vec2(
            nodes[spring_connection.from].x,
            nodes[spring_connection.from].y,
        );
    diff = diff.normalize();

    // Deviation from true spring
    // If longer than length, don't apply forces. 1 way spring.
    if abs(diff.magnitude()) > length {
        return;
    }

    diff *= length;
    let target = vec2(
        nodes[spring_connection.from].x,
        nodes[spring_connection.from].y,
    ) + diff;

    let mut force = target - vec2(nodes[spring_connection.to].x, nodes[spring_connection.to].y);
    force *= 0.5;
    force *= stiffness;
    force *= 1.0 - damping;

    nodes[spring_connection.to].velocity += force;
    force *= -1.0;
    nodes[spring_connection.from].velocity += force;
}

fn update_hulls(group: &NodeGroup) -> Vec<LineString<f32>> {
    let points = group
        .nodes
        .iter()
        .map(|n| vec![n.x, n.y])
        .collect::<Vec<Vec<f32>>>();

    let clusters = cluster(40.0, 20, &points);

    let mut clustered_points = Vec::<Vec<Coordinate<f32>>>::new();
    for (point_index, cluster_def) in clusters.iter().enumerate() {
        match cluster_def {
            Classification::Core(cluster_id) => {
                if clustered_points.len() <= *cluster_id {
                    clustered_points.push(Vec::<Coordinate<f32>>::new());
                }

                let copy_point = &points[point_index];
                clustered_points[*cluster_id].push(Coordinate {
                    x: copy_point[0],
                    y: copy_point[1],
                });
            }
            _ => {}
        }
    }

    clustered_points
        .iter()
        .map(|coords| {
            let poly = Polygon::new(LineString(coords.to_vec()), vec![]);
            let hull = poly.concave_hull(2.0);

            hull.exterior().simplify(&1.0)
        })
        .collect::<Vec<LineString<f32>>>()
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    for g in 0..model.node_groups.len() {
        for og in 0..model.node_groups.len() {
            let ng = &mut model.node_groups;
            attract_nodes(
                ng,
                g,
                og,
                model.settings.node_attract_strength_friendly,
                model.settings.node_attract_strength,
            );
        }
    }

    for g in 0..model.node_groups.len() {
        let node_group = &mut model.node_groups[g];
        for connection in node_group.spring_connections.iter() {
            // apply spring forces
            spring(&mut node_group.nodes, connection);
        }
    }

    gravity(&mut model.node_groups);

    for g in 0..model.node_groups.len() {
        for i in 0..model.node_groups[g].nodes.len() {
            // Apply velocity vector and update position
            model.node_groups[g].nodes[i].update();
        }
    }

    for g in 0..model.node_groups.len() {
        let new_hulls = update_hulls(&model.node_groups[g]);
        model.node_groups[g].convex_hulls = new_hulls;
    }

    // Advance day if required.
    let frames_per_day = model.settings.day_seconds * model.settings.frame_rate;
    let stab_frames = model.settings.stabilize_time * model.settings.frame_rate;

    if model.frame >= stab_frames {
        let offset_frames = model.frame - stab_frames;
        if offset_frames % frames_per_day == 0 {
            let day: usize = cast(offset_frames / frames_per_day).unwrap();

            if model.day != day {
                model.set_day(day);
            }
        }
    }

    model.frame += 1;
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background()
        .color(rgba(107.0 / 255.0, 119.0 / 255.0, 237.0 / 255.0, 1.0));

    draw.texture(&model.map_texture);

    for group in &model.node_groups {
        let mut text_pos: Vector2 = Vector2 { x: 0.0, y: 0.0 };
        let mut biggest_area = 0.0;
        let mut biggest_width = 0.0;

        group.convex_hulls.iter().for_each(|hull| {
            let mut x_min = 10000.0;
            let mut x_max = -10000.0;
            let mut y_min = 10000.0;
            let mut y_max = -10000.0;

            for p in hull.clone().into_iter() {
                x_min = f32::min(x_min, p.x);
                x_max = f32::max(x_max, p.x);
                y_min = f32::min(y_min, p.y);
                y_max = f32::max(y_max, p.y);
            }

            let center = hull.centroid().unwrap();
            let center_vec = vec2(center.x(), center.y());
            let hull_width = x_max - x_min;
            let hull_height = y_max - y_min;
            let area = hull_width * hull_height;

            if area > biggest_area {
                biggest_area = area;
                biggest_width = hull_width;
                text_pos = center_vec;
            }

            let num = if area > 160000.0 {
                3
            } else if area > 30000.0 {
                2
            } else {
                1
            };
            for i in 0..num + 1 {
                let mult = (i as f32) / (num as f32);
                let line = hull
                    .clone()
                    .into_iter()
                    .map(|p| {
                        let point_vec = vec2(p.x, p.y);
                        let direction_vector = point_vec - center_vec;
                        center_vec + (direction_vector * mult)
                    })
                    .collect::<Vec<Vector2>>();
                draw.polyline()
                    .color(get_group_colour(group.id))
                    .stroke_weight(4.0)
                    .join_round()
                    .points(line.clone());
            }
        });

        // Check that the text is within the windows bounds
        let win_rect = app.main_window().rect().pad(96.0);
        let in_bounds = win_rect.contains(text_pos);

        if group.convex_hulls.len() > 0 && biggest_area > 500.0 && in_bounds && biggest_width > 64.0
        {
            let cur_value = group.display_values[model.day];
            let fmt = format!(
                r#"
            {}
            {}
            "#,
                group.label, cur_value
            );
            let label = fmt.as_str();

            draw.text(label)
                .font(model.display_font.clone())
                .font_size(18)
                .line_spacing(2.0)
                .x_y(text_pos.x - 2.0, text_pos.y - 2.0)
                .center_justify()
                .color(WHITE);
            draw.text(label)
                .font(model.display_font.clone())
                .font_size(18)
                .line_spacing(2.0)
                .x_y(text_pos.x, text_pos.y)
                .center_justify()
                .color(BLACK);
        }
    }

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();

    let stab_frames = model.settings.stabilize_time * model.settings.frame_rate;
    let end_frame = stab_frames + (model.settings.frame_rate * model.settings.day_seconds * 7);

    if model.frame > stab_frames && model.frame < end_frame {
        let adjusted_frame = model.frame - stab_frames;
        app.main_window().capture_frame(
            format!(
                "./out/{}/{:0>6}.png",
                &model.settings.start_date, adjusted_frame
            )
            .as_str(),
        );
    }

    if model.frame > end_frame {
        std::process::exit(0);
    }
}

fn get_group_colour(id: usize) -> Rgba {
    match id {
        0 => rgba(0.97254902, 0.149019608, 0.0, 1.0),
        1 => rgba(47.0 / 255.0, 49.0 / 255.0, 235.0 / 255.0, 1.0),
        2 => rgba(0.97254902, 0.149019608, 0.0, 1.0),
        3 => rgba(47.0 / 255.0, 49.0 / 255.0, 235.0 / 255.0, 1.0),
        4 => rgba(0.97254902, 0.149019608, 0.0, 1.0),
        5 => rgba(47.0 / 255.0, 49.0 / 255.0, 235.0 / 255.0, 1.0),
        6 => rgba(0.97254902, 0.149019608, 0.0, 1.0),
        7 => rgba(47.0 / 255.0, 49.0 / 255.0, 235.0 / 255.0, 1.0),
        8 => rgba(0.97254902, 0.149019608, 0.0, 1.0),
        9 => rgba(47.0 / 255.0, 49.0 / 255.0, 235.0 / 255.0, 1.0),
        10 => rgba(0.97254902, 0.149019608, 0.0, 1.0),
        11 => rgba(47.0 / 255.0, 49.0 / 255.0, 235.0 / 255.0, 1.0),
        _ => rgba(0.97254902, 0.149019608, 0.0, 1.0),
    }
}

fn key_pressed(_app: &App, _model: &mut Model, _key: Key) {}

fn mouse_released(_app: &App, _model: &mut Model, _button: MouseButton) {}
