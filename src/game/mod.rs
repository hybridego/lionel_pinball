use crate::game::physics::PhysicsEngine;
use rand::Rng;
use rapier2d::prelude::*;

pub mod maps;
pub mod physics;

pub const GROUP_BALL: Group = Group::GROUP_1;
pub const GROUP_MAP: Group = Group::GROUP_2;
pub const GROUP_SPINNER: Group = Group::GROUP_3;

pub struct Ball {
    pub name: String,
    pub handle: RigidBodyHandle,
    pub color: [u8; 3], // RGB
}

#[derive(Clone, Copy)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub color: [u8; 3],
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EditorTool {
    Pin,
    Wall,
    Eraser,
}

#[derive(Clone, Copy, PartialEq)]
pub enum WinningCondition {
    First,
    Last,
}

pub struct FinishedBall {
    pub name: String,
    pub color: [u8; 3],
    pub finished_at: f64,
}

pub struct GameState {
    pub physics: PhysicsEngine,
    pub balls: Vec<Ball>,
    pub finished_balls: Vec<FinishedBall>,
    pub winning_condition: WinningCondition,

    pub is_running: bool,
    pub edit_mode: bool,
    pub selected_tool: EditorTool,
    pub map_width: f32,
    pub map_height: f32,

    // Editor State
    pub editor_drag_start: Option<(f32, f32)>,
    pub editor_grid_snap: bool,

    // Visual Effects
    pub particles: Vec<Particle>,
}

impl GameState {
    pub fn new() -> Self {
        let mut physics = PhysicsEngine::new();
        // Initialize default map
        // Coordinate system: Center is (0,0). Width 500 means -250 to 250. Height 800 means -400 to 400.
        let width = 500.0;
        let height = 800.0;

        maps::create_map(&mut physics, width, height);

        Self {
            physics,
            balls: Vec::new(),
            finished_balls: Vec::new(),
            winning_condition: WinningCondition::First, // Changed default to First
            is_running: false,
            edit_mode: false,
            selected_tool: EditorTool::Pin,
            map_width: width,
            map_height: height,
            editor_drag_start: None,
            editor_grid_snap: true,
            particles: Vec::new(),
        }
    }

    fn editor_snap(&self, val: f32) -> f32 {
        if self.editor_grid_snap {
            let grid_size = 20.0;
            (val / grid_size).round() * grid_size
        } else {
            val
        }
    }

    pub fn editor_input_start(&mut self, x: f32, y: f32) {
        if !self.edit_mode {
            return;
        }

        let x = self.editor_snap(x);
        let y = self.editor_snap(y);

        match self.selected_tool {
            EditorTool::Pin => {
                // Pin is instant placement on click (or drag if we want to paint)
                // Let's keep it simple: click/release handling.
                // But for pin, maybe better to place on release to avoid duplicates if dragged?
                // Or "painting" support.
                // For now, let's treat Pin as "Place on click" (start).
                let collider = ColliderBuilder::ball(5.0)
                    .translation(vector![x, y])
                    .restitution(0.7)
                    .build();
                self.physics.collider_set.insert(collider);
            }
            EditorTool::Wall => {
                // Start dragging
                self.editor_drag_start = Some((x, y));
            }
            EditorTool::Eraser => {
                // Erase on start
                self.editor_erase(x, y);
            }
        }
    }

    pub fn editor_input_end(&mut self, x: f32, y: f32) {
        if !self.edit_mode {
            return;
        }

        let x = self.editor_snap(x);
        let y = self.editor_snap(y);

        match self.selected_tool {
            EditorTool::Pin => {
                // Already placed on start
            }
            EditorTool::Wall => {
                if let Some((start_x, start_y)) = self.editor_drag_start {
                    // Create wall from start to current
                    let dx = x - start_x;
                    let dy = y - start_y;
                    let length = (dx * dx + dy * dy).sqrt();

                    if length > 5.0 {
                        // Center point
                        let cx = (start_x + x) / 2.0;
                        let cy = (start_y + y) / 2.0;
                        let angle = dy.atan2(dx);

                        let collider = ColliderBuilder::cuboid(length / 2.0, 5.0)
                            .translation(vector![cx, cy])
                            .rotation(angle)
                            .build();
                        self.physics.collider_set.insert(collider);
                    }

                    self.editor_drag_start = None;
                }
            }
            EditorTool::Eraser => {
                // Optional: Erase on end too ("painting" eraser if dragged?)
                // For now just click
            }
        }
    }

    fn editor_erase(&mut self, x: f32, y: f32) {
        let point = point![x, y];
        self.physics
            .query_pipeline
            .update(&self.physics.rigid_body_set, &self.physics.collider_set);

        let filter = QueryFilter::default();
        let mut handle_to_remove = None;

        self.physics.query_pipeline.intersections_with_point(
            &self.physics.rigid_body_set,
            &self.physics.collider_set,
            &point,
            filter,
            |handle| {
                handle_to_remove = Some(handle);
                false // Stop at first
            },
        );

        if let Some(handle) = handle_to_remove {
            self.physics.collider_set.remove(
                handle,
                &mut self.physics.island_manager,
                &mut self.physics.rigid_body_set,
                true,
            );
        }
    }

    pub fn update(&mut self, current_time: f64) {
        if self.is_running {
            // Safety Clamp: Limit max velocity to prevent physics explosions (tunneling/crashes)
            let max_speed = 3000.0; // Increased limit for higher gravity
            for (_handle, rb) in self.physics.rigid_body_set.iter_mut() {
                if rb.is_dynamic() {
                    let vel = *rb.linvel();
                    let speed_sq = vel.magnitude_squared();
                    if speed_sq > max_speed * max_speed {
                        let speed = speed_sq.sqrt();
                        let scale = max_speed / speed;
                        rb.set_linvel(vel * scale, true);
                    }
                }
            }

            self.physics.step();
            self.check_finished_balls(current_time);
            self.handle_collisions();
            self.spawn_trails(); // NEW: Trail Effect
            self.update_particles();
        }
    }

    fn spawn_trails(&mut self) {
        let mut rng = rand::thread_rng();
        // For each active ball, spawn a small trail particle
        for ball in &self.balls {
            if let Some(rb) = self.physics.rigid_body_set.get(ball.handle) {
                let pos = rb.translation();
                let vel = rb.linvel();

                // Only spawn if moving reasonable speed
                if vel.magnitude_squared() > 10.0 {
                    let particle = Particle {
                        x: pos.x + rng.gen_range(-2.0..2.0),
                        y: pos.y + rng.gen_range(-2.0..2.0),
                        vx: -vel.x * 0.2, // Drags behind
                        vy: -vel.y * 0.2,
                        life: 0.3,              // Short life
                        color: [200, 200, 255], // Soft white/blue dust
                    };
                    self.particles.push(particle);
                }
            }
        }
    }

    fn handle_collisions(&mut self) {
        let events = self.physics.drain_collision_events();
        for event in events {
            if let CollisionEvent::Started(h1, h2, _flags) = event {
                let c1 = self.physics.collider_set.get(h1);
                let c2 = self.physics.collider_set.get(h2);

                let pos1 = c1
                    .and_then(|c| c.parent())
                    .and_then(|h| self.physics.rigid_body_set.get(h))
                    .map(|rb| *rb.translation());
                // If static, collider might not have parent or RB, use collider translation
                let p1_final = pos1
                    .unwrap_or_else(|| c1.map(|c| *c.translation()).unwrap_or(vector![0.0, 0.0]));

                let pos2 = c2
                    .and_then(|c| c.parent())
                    .and_then(|h| self.physics.rigid_body_set.get(h))
                    .map(|rb| *rb.translation());
                let p2_final = pos2
                    .unwrap_or_else(|| c2.map(|c| *c.translation()).unwrap_or(vector![0.0, 0.0]));

                let rest1 = c1.map(|c| c.restitution()).unwrap_or(0.5);
                let rest2 = c2.map(|c| c.restitution()).unwrap_or(0.5);
                let intensity = rest1.max(rest2);

                // Determine 'Type' based on user_data
                // 1=Red Pin, 2=Green Pin, 3=Orange Bumper, 10=Spinner, 99=Goal
                // Ball has 0 usually.
                let u1 = c1.map(|c| c.user_data).unwrap_or(0);
                let u2 = c2.map(|c| c.user_data).unwrap_or(0);

                // Pick the interesting user_data (non-zero)
                let type_id = if u1 > 0 { u1 } else { u2 };

                let cx = (p1_final.x + p2_final.x) / 2.0;
                let cy = (p1_final.y + p2_final.y) / 2.0;

                self.spawn_particles(cx, cy, intensity, type_id);
            }
        }
    }

    fn spawn_particles(&mut self, x: f32, y: f32, intensity: f32, type_id: u128) {
        let mut rng = rand::thread_rng();

        // Boost counts for "Flashy" feel
        // Base 10.. max 80 for super hits
        let base_count = 10.0;
        let count = ((base_count + intensity * 15.0).clamp(10.0, 100.0)) as usize;

        for _ in 0..count {
            let angle: f32 = rng.gen_range(0.0..6.28);

            // Speed boost
            let speed_mult = intensity.clamp(0.8, 4.0);
            let speed = rng.gen_range(60.0..200.0) * speed_mult;

            let vx = angle.cos() * speed;
            let vy = angle.sin() * speed;
            let life = rng.gen_range(0.4..1.0); // Longer life

            // Color Logic based on Type ID
            let color = match type_id {
                11 => {
                    // Level 1 - Blue
                    [0, 100, 255] // Distinct Blue (not Cyan)
                }
                12 => {
                    // Level 2 - Green
                    match rng.gen_range(0..2) {
                        0 => [50, 255, 50],
                        _ => [100, 255, 100],
                    }
                }
                13 => {
                    // Level 3 - Yellow
                    match rng.gen_range(0..2) {
                        0 => [255, 255, 0],
                        _ => [255, 255, 100],
                    }
                }
                14 => {
                    // Level 4 - Orange
                    match rng.gen_range(0..3) {
                        0 => [255, 100, 0],
                        1 => [255, 200, 0],
                        _ => [255, 255, 255],
                    }
                }
                15 => {
                    // Level 5 - Red
                    match rng.gen_range(0..3) {
                        0 => [255, 50, 50],
                        1 => [255, 0, 0],
                        _ => [255, 200, 200],
                    }
                }
                21 => {
                    // Slow Windmill - Cyan
                    [0, 255, 255]
                }
                22 => {
                    // Normal Windmill - Magenta
                    [255, 0, 255]
                }
                23 => {
                    // Fast Windmill - Purple
                    [128, 0, 128]
                }
                10 => {
                    // Legacy / Fallback
                    match rng.gen_range(0..3) {
                        0 => [0, 200, 255],   // Cyan
                        1 => [100, 100, 255], // Blue
                        _ => [200, 255, 255], // White Cyan
                    }
                }
                99 => {
                    // Goal - Rainbow/Victory
                    [rng.gen(), rng.gen(), rng.gen()]
                }
                _ => {
                    // Default / Wall / Ball-on-Ball
                    // Use intensity to decide
                    if intensity > 2.0 {
                        [200, 200, 255] // Bright Blue-White
                    } else {
                        // Sparky gray/blue
                        let v = rng.gen_range(150..255);
                        [v, v, 255]
                    }
                }
            };

            self.particles.push(Particle {
                x,
                y,
                vx,
                vy,
                life,
                color,
            });
        }
    }

    fn update_particles(&mut self) {
        let dt = 1.0 / 60.0; // estim
        for p in &mut self.particles {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.vy -= 200.0 * dt; // Gravity
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);
    }

    fn check_finished_balls(&mut self, current_time: f64) {
        let finish_y = -self.map_height / 2.0 + 50.0; // Threshold
        let mut completed_indices = Vec::new();

        for (i, ball) in self.balls.iter().enumerate() {
            if let Some(rb) = self.physics.rigid_body_set.get(ball.handle) {
                if rb.translation().y < finish_y {
                    completed_indices.push(i);
                }
            }
        }

        // Process in reverse to maintain indices when removing
        for i in completed_indices.into_iter().rev() {
            let ball = self.balls.remove(i);
            // Remove from physics
            self.physics.rigid_body_set.remove(
                ball.handle,
                &mut self.physics.island_manager,
                &mut self.physics.collider_set,
                &mut self.physics.impulse_joint_set,
                &mut self.physics.multibody_joint_set,
                true,
            );

            self.finished_balls.push(FinishedBall {
                name: ball.name,
                color: ball.color,
                finished_at: current_time,
            });
        }
    }

    pub fn spawn_ball(&mut self, name: String) {
        let mut rng = rand::thread_rng();
        let x_offset = rng.gen_range(-100.0..100.0);
        let y_start = self.map_height / 2.0 - 20.0; // Near top

        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![x_offset, y_start])
            .ccd_enabled(true) // Prevent tunneling
            .linear_damping(0.1) // Air resistance stability
            .build();
        let handle = self.physics.rigid_body_set.insert(rigid_body);

        let collider = ColliderBuilder::ball(8.0)
            .restitution(0.7)
            .friction(0.0)
            .density(1.0)
            .collision_groups(InteractionGroups::new(
                GROUP_BALL,
                GROUP_BALL | GROUP_MAP | GROUP_SPINNER,
            ))
            .build();
        self.physics.collider_set.insert_with_parent(
            collider,
            handle,
            &mut self.physics.rigid_body_set,
        );

        let color = [rng.gen(), rng.gen(), rng.gen()];

        self.balls.push(Ball {
            name,
            handle,
            color,
        });
    }

    pub fn spawn_event_obstacle(&mut self) {
        let mut rng = rand::thread_rng();
        let x_offset = rng.gen_range(-self.map_width / 2.0 + 40.0..self.map_width / 2.0 - 40.0);
        let y_start = self.map_height / 2.0 - 50.0;

        // 1. Random Neon Color (High Saturation/Brightness)
        // HSV to RGB conversion simplified or just pick vibrant mix
        let hue = rng.gen_range(0.0f32..360.0f32);
        let s = 1.0f32;
        let v = 1.0f32;

        let c = v * s;
        let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r_f, g_f, b_f) = if hue < 60.0 {
            (c, x, 0.0)
        } else if hue < 120.0 {
            (x, c, 0.0)
        } else if hue < 180.0 {
            (0.0, c, x)
        } else if hue < 240.0 {
            (0.0, x, c)
        } else if hue < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        let r = ((r_f + m) * 255.0) as u128;
        let g = ((g_f + m) * 255.0) as u128;
        let b = ((b_f + m) * 255.0) as u128;

        // 2. Random Shape: 0=Circle, 1=Square, 2=Triangle, 3=Star
        let shape_id: u128 = rng.gen_range(0..4);

        // UserData Packing:
        // Bit 64: Flag (1)
        // Bits 48-55: R
        // Bits 40-47: G
        // Bits 32-39: B
        // Bits 0-31: Shape ID
        let flag: u128 = 1 << 64;
        let user_data = flag | (r << 48) | (g << 40) | (b << 32) | shape_id;

        // Physics Body
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![x_offset, y_start])
            .rotation(rng.gen_range(0.0..3.14))
            .build();
        let handle = self.physics.rigid_body_set.insert(rigid_body);

        let size = rng.gen_range(15.0..25.0);

        let collider = match shape_id {
            0 => {
                // Circle
                ColliderBuilder::ball(size)
            }
            1 => {
                // Square
                ColliderBuilder::cuboid(size, size)
            }
            2 => {
                // Triangle
                // Equilateral triangle
                let h = size * 3.0f32.sqrt() / 2.0;
                let p1 = point![0.0, -h * 2.0 / 3.0];
                let p2 = point![-size, h / 3.0];
                let p3 = point![size, h / 3.0];
                ColliderBuilder::triangle(p1, p2, p3)
            }
            _ => {
                // Star (3)
                // Physics Proxy: Circle for smooth rolling, or maybe a Hexagon?
                // Let's use a Ball for simplicity and good bouncing behavior.
                // Visually it will be a star.
                ColliderBuilder::ball(size)
            }
        }
        .restitution(0.6)
        .density(1.5)
        .collision_groups(InteractionGroups::new(GROUP_MAP, GROUP_BALL))
        .user_data(user_data)
        .build();

        self.physics.collider_set.insert_with_parent(
            collider,
            handle,
            &mut self.physics.rigid_body_set,
        );
    }

    pub fn reset_map(&mut self) {
        self.balls.clear();
        self.finished_balls.clear();
        self.physics = PhysicsEngine::new();
        // Re-create map
        let width = self.map_width;
        let height = self.map_height;
        maps::create_map(&mut self.physics, width, height);
        self.is_running = false;
    }

    pub fn reset_game(&mut self) {
        // Keep map, just remove balls
        // Indices to remove
        let mut handles_to_remove = Vec::new();
        for ball in &self.balls {
            handles_to_remove.push(ball.handle);
        }

        self.balls.clear();
        self.finished_balls.clear();

        // Remove bodies from physics
        for handle in handles_to_remove {
            self.physics.rigid_body_set.remove(
                handle,
                &mut self.physics.island_manager,
                &mut self.physics.collider_set,
                &mut self.physics.impulse_joint_set,
                &mut self.physics.multibody_joint_set,
                true,
            );
        }

        self.is_running = false;
    }
}
