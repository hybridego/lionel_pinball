use rapier2d::prelude::*;
use crate::game::physics::PhysicsEngine;
use rand::Rng;

pub mod physics;
pub mod maps;

pub const GROUP_BALL: Group = Group::GROUP_1;
pub const GROUP_MAP: Group = Group::GROUP_2;
pub const GROUP_SPINNER: Group = Group::GROUP_3;

pub struct Ball {
    pub name: String,
    pub handle: RigidBodyHandle,
    pub color: [u8; 3], // RGB
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
}

pub struct GameState {
    pub physics: PhysicsEngine,
    pub balls: Vec<Ball>,
    pub finished_balls: Vec<FinishedBall>,
    pub winning_condition: WinningCondition,
    pub current_map: maps::MapType,
    pub is_running: bool,
    pub edit_mode: bool,
    pub selected_tool: EditorTool,
    pub map_width: f32,
    pub map_height: f32,
    
    // Editor State
    pub editor_drag_start: Option<(f32, f32)>,
    pub editor_grid_snap: bool,
}

impl GameState {
    pub fn new() -> Self {
        let mut physics = PhysicsEngine::new();
        // Initialize default map
        // Coordinate system: Center is (0,0). Width 500 means -250 to 250. Height 800 means -400 to 400.
        let width = 500.0; 
        let height = 800.0;
        
        maps::create_map(&mut physics, width, height, maps::MapType::Default);

        Self {
            physics,
            balls: Vec::new(),
            finished_balls: Vec::new(),
            winning_condition: WinningCondition::Last,
            current_map: maps::MapType::Default,
            is_running: false,
            edit_mode: false,
            selected_tool: EditorTool::Pin,
            map_width: width,
            map_height: height,
            editor_drag_start: None,
            editor_grid_snap: true,
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
        if !self.edit_mode { return; }
        
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
            },
            EditorTool::Wall => {
                // Start dragging
                self.editor_drag_start = Some((x, y));
            },
            EditorTool::Eraser => {
                // Erase on start
                 self.editor_erase(x, y);
            }
        }
    }
    
    pub fn editor_input_end(&mut self, x: f32, y: f32) {
        if !self.edit_mode { return; }
        
        let x = self.editor_snap(x);
        let y = self.editor_snap(y);
        
        match self.selected_tool {
            EditorTool::Pin => {
                // Already placed on start
            },
            EditorTool::Wall => {
                 if let Some((start_x, start_y)) = self.editor_drag_start {
                     // Create wall from start to current
                     let dx = x - start_x;
                     let dy = y - start_y;
                     let length = (dx*dx + dy*dy).sqrt();
                     
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
            },
            EditorTool::Eraser => {
                // Optional: Erase on end too ("painting" eraser if dragged?)
                // For now just click
            }
        }
    }
    
    fn editor_erase(&mut self, x: f32, y: f32) {
        let point = point![x, y];
        self.physics.query_pipeline.update(&self.physics.rigid_body_set, &self.physics.collider_set);
        
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
            }
        );
        
        if let Some(handle) = handle_to_remove {
            self.physics.collider_set.remove(handle, &mut self.physics.island_manager, &mut self.physics.rigid_body_set, true);
        }
    }

    pub fn update(&mut self) {
        if self.is_running {
            // Safety Clamp: Limit max velocity to prevent physics explosions (tunneling/crashes)
            let max_speed = 1500.0; // Reasonable limit for this map size
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
            self.check_finished_balls();
        }
    }

    fn check_finished_balls(&mut self) {
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
            self.physics.rigid_body_set.remove(ball.handle, &mut self.physics.island_manager, &mut self.physics.collider_set, &mut self.physics.impulse_joint_set, &mut self.physics.multibody_joint_set, true);
            
            self.finished_balls.push(FinishedBall {
                name: ball.name,
                color: ball.color,
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
            .collision_groups(InteractionGroups::new(GROUP_BALL, GROUP_BALL | GROUP_MAP | GROUP_SPINNER))
            .build();
        self.physics.collider_set.insert_with_parent(collider, handle, &mut self.physics.rigid_body_set);

        let color = [rng.gen(), rng.gen(), rng.gen()];

        self.balls.push(Ball {
            name,
            handle,
            color
        });
    }

    pub fn spawn_event_obstacle(&mut self) {
        let mut rng = rand::thread_rng();
        let x_offset = rng.gen_range(-self.map_width/2.0 + 20.0 .. self.map_width/2.0 - 20.0);
        let y_start = self.map_height / 2.0 - 50.0; 

        // Spawn a dynamic box (chocolate/crate)
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![x_offset, y_start])
            .rotation(rng.gen_range(0.0..3.14))
            .build();
        let handle = self.physics.rigid_body_set.insert(rigid_body);
        
        // Random size
        let w = rng.gen_range(10.0..30.0);
        let h = rng.gen_range(10.0..30.0);
        
        let collider = ColliderBuilder::cuboid(w, h)
            .restitution(0.3)
            .density(2.0) // Heavier
            .collision_groups(InteractionGroups::new(GROUP_MAP, GROUP_BALL))
            .build();
            
        self.physics.collider_set.insert_with_parent(collider, handle, &mut self.physics.rigid_body_set);
        
        // We don't track obstacles in 'balls' list, they are just physics objects. 
        // Although we might want to clean them up if they fall out.
        // For now, let them stay or fall forever (Rapier handles them).
        // Actually, if they fall out, they just fall forever in void, which is fine for now but waste resources.
        // Ideally we track them in a separate list or tag them.
    }

    pub fn reset_map(&mut self) {
        self.balls.clear();
        self.finished_balls.clear();
        self.physics = PhysicsEngine::new();
        // Re-create map
        let width = self.map_width;
        let height = self.map_height;
        maps::create_map(&mut self.physics, width, height, self.current_map);
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
            self.physics.rigid_body_set.remove(handle, &mut self.physics.island_manager, &mut self.physics.collider_set, &mut self.physics.impulse_joint_set, &mut self.physics.multibody_joint_set, true);
        }
        
        self.is_running = false;
    }


    pub fn nudge(&mut self) {
        let mut rng = rand::thread_rng();
        for ball in &self.balls {
            if let Some(rb) = self.physics.rigid_body_set.get_mut(ball.handle) {
                // Wake up just in case
                rb.wake_up(true);
                // Apply random impulse: mostly up + random side
                let x_impulse = rng.gen_range(-20.0..20.0);
                let y_impulse = rng.gen_range(10.0..30.0); // Upward kick
                rb.apply_impulse(vector![x_impulse, y_impulse], true);
            }
        }
    }


}
