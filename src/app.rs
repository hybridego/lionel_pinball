use eframe::egui;
use crate::game::GameState;


pub struct PinballApp {
    state: GameState,
    input_text: String,
    // Configuration
}

impl PinballApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        Self {
            state: GameState::new(),
            input_text: "Alice*5\nBob*3".to_owned(),
        }
    }

    fn parse_and_spawn(&mut self) {
        self.state.reset_game(); // Only clear balls
        
        let lines = self.input_text.lines();
        for line in lines {
            let parts: Vec<&str> = line.split('*').collect();
            if parts.len() == 2 {
                let name = parts[0].trim();
                let count = parts[1].trim().parse::<usize>().unwrap_or(1);
                for i in 1..=count {
                    self.state.spawn_ball(format!("{}#{}", name, i));
                }
            } else if !line.is_empty() {
                // Just one
                self.state.spawn_ball(line.trim().to_string());
            }
        }
        self.state.is_running = true;
    }
}

impl eframe::App for PinballApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Game Loop
        self.state.update();
        if self.state.is_running {
             ctx.request_repaint(); // Animation
        }

        // Sidebar
        egui::SidePanel::left("sidebar_panel").show(ctx, |ui| {
            ui.heading("Settings");
            ui.label("Enter Names (Name*Count):");
            ui.text_edit_multiline(&mut self.input_text);
            
            if ui.button("Start Game").clicked() {
                self.parse_and_spawn();
            }
            
            if ui.button("Stop/Reset").clicked() {
                self.state.is_running = false;
                self.state.reset_game();
            }
            
            if ui.button("Nudge / Shake (Space)").clicked() {
                self.state.nudge();
            }
            
            if ui.button("Trigger Event (Drop Object)").clicked() {
                self.state.spawn_event_obstacle();
            }

            if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
                self.state.nudge();
            }
            
            ui.separator();
            ui.label("Map Selection:");
            egui::ComboBox::from_label("Choose Map")
                .selected_text(match self.state.current_map {
                    crate::game::maps::MapType::Default => "Default (Pins)",
                    crate::game::maps::MapType::ZigZag => "ZigZag Plates",
                    crate::game::maps::MapType::Pachinko => "Pachinko (Dense)",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.state.current_map, crate::game::maps::MapType::Default, "Default (Pins)");
                    ui.selectable_value(&mut self.state.current_map, crate::game::maps::MapType::ZigZag, "ZigZag Plates");
                    ui.selectable_value(&mut self.state.current_map, crate::game::maps::MapType::Pachinko, "Pachinko (Dense)");
                });
            
            if ui.button("New Map (Randomize)").clicked() {
                self.state.reset_map();
            }

            ui.separator();
            ui.checkbox(&mut self.state.edit_mode, "Edit Mode");
            if self.state.edit_mode {
                ui.label("Tools:");
                ui.radio_value(&mut self.state.selected_tool, crate::game::EditorTool::Pin, "Pin");
                ui.radio_value(&mut self.state.selected_tool, crate::game::EditorTool::Wall, "Wall (Drag)");
                ui.radio_value(&mut self.state.selected_tool, crate::game::EditorTool::Eraser, "Eraser");
                ui.checkbox(&mut self.state.editor_grid_snap, "Grid Snap");
                
                ui.label(egui::RichText::new("Drag to create walls.").small());
            }
            
            ui.separator();
            ui.label("Winning Condition:");
            ui.radio_value(&mut self.state.winning_condition, crate::game::WinningCondition::First, "First to Arrive");
            ui.radio_value(&mut self.state.winning_condition, crate::game::WinningCondition::Last, "Last to Arrive");
            
            ui.separator();
            ui.label(format!("Balls Active: {}", self.state.balls.len()));
            ui.label(format!("Finished: {}", self.state.finished_balls.len()));
            
            if !self.state.finished_balls.is_empty() {
                ui.separator();
                ui.label("Results:");
                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    for (i, ball) in self.state.finished_balls.iter().enumerate() {
                         ui.horizontal(|ui| {
                             ui.label(format!("{}.", i + 1));
                             ui.label(egui::RichText::new(&ball.name).color(egui::Color32::from_rgb(ball.color[0], ball.color[1], ball.color[2])));
                         });
                    }
                });
                
                // Show Winner
                ui.separator();
                let winner_idx = match self.state.winning_condition {
                    crate::game::WinningCondition::First => if !self.state.finished_balls.is_empty() { Some(0) } else { None },
                    crate::game::WinningCondition::Last => if self.state.balls.is_empty() && !self.state.finished_balls.is_empty() { Some(self.state.finished_balls.len() - 1) } else { None },
                };
                
                if let Some(idx) = winner_idx {
                    let winner = &self.state.finished_balls[idx];
                    ui.label(egui::RichText::new(format!("WINNER: {}", winner.name)).size(20.0).strong().color(egui::Color32::GREEN));
                }
            }
        });

        // Main Canvas
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
            
            // Coordinate mapping
            // Physics world: 0,0 is center. Y is up.
            // Screen: 0,0 is top-left. Y is down.
            let rect = response.rect;
            let center = rect.center();
            
            // Helper to transform world point to screen point
            let to_screen = |x: f32, y: f32| -> egui::Pos2 {
                // simple scaling
                let scale = 1.0; 
                // world y is up, screen y is down.
                egui::pos2(center.x + x * scale, center.y - y * scale) 
            };

            // Helper to transform screen point to world point
            let to_world = |pos: egui::Pos2| -> (f32, f32) {
                let scale = 1.0;
                let x = (pos.x - center.x) / scale;
                let y = (center.y - pos.y) / scale;
                (x, y)
            };
            
            // Input Handling
            if self.state.edit_mode {
                if let Some(pos) = response.interact_pointer_pos() {
                    let (wx, wy) = to_world(pos);
                    
                    if response.drag_started() {
                        self.state.editor_input_start(wx, wy);
                    } else if response.drag_stopped() {
                         // Drag released might happen outside? Egui handles it if we grabbed.
                         // But response.drag_released() is true on release frame.
                         self.state.editor_input_end(wx, wy); 
                    } else if response.clicked() {
                        // For simple clicks (Pin/Eraser) that didn't register as drag
                        // If pin, we want to place.
                        // Our editor_input_start handles simple placement too? 
                        // drag_started usually fires on click too? No, usually separate.
                        // Let's call start/end immediately for click?
                        // Actually, if we just want "Click" for pins, we can use clicked().
                        // But drag_started/released is consistent.
                        // Let's rely on drag_started/released for everything if possible, 
                        // OR if click matches simple tool.
                        // For simplicity, let's trigger start/end on click if not dragging.
                        if self.state.selected_tool == crate::game::EditorTool::Pin || self.state.selected_tool == crate::game::EditorTool::Eraser {
                             self.state.editor_input_start(wx, wy);
                             self.state.editor_input_end(wx, wy);
                        }
                    }
                    
                    // Preview for Wall Dragging
                    if let Some((sx, sy)) = self.state.editor_drag_start {
                         if self.state.selected_tool == crate::game::EditorTool::Wall {
                             // Snap current mouse pos for preview
                             // We don't have access to snap function here easily unless exposed, 
                             // but we can trust GameState handles actual creation.
                             // Visual feedback:
                             let start_screen = to_screen(sx, sy);
                             let end_screen = pos; // current mouse pos
                             painter.line_segment([start_screen, end_screen], egui::Stroke::new(2.0, egui::Color32::YELLOW));
                         }
                    }
                }
            }
            
            // Draw Walls/Pins (Static Colliders)
            // Ideally we iterate colliders and draw them.
            // For now, let's just cheat and draw the known map boundaries or iterate if we can exposed iter
            
            // Let's iterate the collider set in physics
            for (_handle, collider) in self.state.physics.collider_set.iter() {
                let translation = collider.translation();
                let shape = collider.shape();
                
                // Check shape type
                if let Some(ball) = shape.as_ball() {
                    let radius = ball.radius;
                    let color = if collider.user_data == 1 {
                        egui::Color32::from_rgb(255, 100, 100) // Reddish/Salmon for Super Pin
                    } else if collider.user_data == 2 {
                        egui::Color32::GREEN // Green for Medium Pin
                    } else if collider.user_data == 3 {
                        egui::Color32::from_rgb(255, 165, 0) // Orange for Funnel Bumper
                    } else if collider.user_data == 99 {
                        egui::Color32::from_rgb(0, 255, 255) // Cyan for Goal
                    } else {
                        egui::Color32::GRAY
                    };
                    
                    painter.circle_filled(
                        to_screen(translation.x, translation.y), 
                        radius, 
                        color
                    );
                } else if let Some(cuboid) = shape.as_cuboid() {
                     let half_extents = cuboid.half_extents;
                     let rotation = collider.rotation();
                     let angle = rotation.angle();
                     
                     // Color logic based on user_data
                     let color = if collider.user_data == 1 {
                        egui::Color32::from_rgb(255, 100, 100) // Red
                     } else if collider.user_data == 3 {
                        egui::Color32::from_rgb(255, 165, 0) // Orange
                     } else if collider.user_data == 99 {
                        egui::Color32::from_rgb(0, 255, 255) // Cyan for Goal
                     } else {
                        egui::Color32::DARK_GRAY
                     };
                     
                     // If no rotation, draw aligned rect
                     if angle.abs() < 0.001 {
                         let rect_min = to_screen(translation.x - half_extents.x, translation.y + half_extents.y);
                         let rect_max = to_screen(translation.x + half_extents.x, translation.y - half_extents.y);
                         painter.rect_filled(
                             egui::Rect::from_min_max(rect_min, rect_max), 
                             0.0, 
                             color
                         );
                     } else {
                         // Rotated rect
                         // Local corners: (+x,+y), (-x,+y), (-x,-y), (+x,-y)
                         // But Rapier 2D cuboid is half_extents
                         let hx = half_extents.x;
                         let hy = half_extents.y;
                         
                         // Rotate points manually
                         let cos_a = angle.cos();
                         let sin_a = angle.sin();
                         
                         let rotate = |lx: f32, ly: f32| -> (f32, f32) {
                             (lx * cos_a - ly * sin_a, lx * sin_a + ly * cos_a)
                         };
                         
                         // Corners in local frame relative to center, then translated
                         let corners = [
                             (hx, hy),
                             (-hx, hy),
                             (-hx, -hy),
                             (hx, -hy),
                         ];
                         
                         let points: Vec<egui::Pos2> = corners.iter().map(|(lx, ly)| {
                             let (rx, ry) = rotate(*lx, *ly);
                             to_screen(translation.x + rx, translation.y + ry)
                         }).collect();
                         
                         painter.add(egui::Shape::convex_polygon(
                             points, 
                             color, 
                             egui::Stroke::NONE
                         ));
                     }
                }
            }

            // Draw Balls
            for ball in &self.state.balls {
                let ball_handle = ball.handle;
                if let Some(rb) = self.state.physics.rigid_body_set.get(ball_handle) {
                    let pos = rb.translation();
                    let screen_pos = to_screen(pos.x, pos.y);
                    let color = egui::Color32::from_rgb(ball.color[0], ball.color[1], ball.color[2]);
                    
                    painter.circle(
                        screen_pos, 
                        8.0, 
                        color, 
                        egui::Stroke::new(1.5, egui::Color32::BLACK) // Outline
                    );
                    
                    // Adaptive Text Color
                    let text_color = if ui.visuals().dark_mode {
                        egui::Color32::WHITE
                    } else {
                        egui::Color32::BLACK
                    };
                    
                    // Label
                    let text_pos = screen_pos + egui::vec2(0.0, 12.0);
                    painter.text(
                        text_pos, 
                        egui::Align2::CENTER_TOP, 
                        &ball.name, 
                        egui::FontId::proportional(12.0), 
                        text_color
                    );
                }
            }
        });
    }
}
