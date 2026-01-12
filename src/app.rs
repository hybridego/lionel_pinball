use crate::game::GameState;
use eframe::egui;
use rapier2d::prelude::point; // Import point macro

pub struct PinballApp {
    state: GameState,
    input_text: String,
    // Configuration
}

impl PinballApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Font Setup for Korean Support
        let mut fonts = egui::FontDefinitions::default();

        // Load the font using include_bytes! (Embeds it in the WASM)
        // Path is relative to this file (src/app.rs) -> ../assets/
        fonts.font_data.insert(
            "korean_font".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/ChungBuk_70_Regular.ttf")),
        );

        // Put my font first (highest priority) for Proportional text:
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "korean_font".to_owned());

        // Put my font as last fallback for Monospace:
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("korean_font".to_owned());

        cc.egui_ctx.set_fonts(fonts);
        cc.egui_ctx.set_visuals(egui::Visuals::dark()); // Neon Dark Mode
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
        let time = ctx.input(|i| i.time);
        self.state.update(time);
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

            if ui.button("Trigger Event (Drop Object)").clicked() {
                self.state.spawn_event_obstacle();
            }

            if ui.button("New Map (Randomize)").clicked() {
                self.state.reset_map();
            }

            ui.separator();
            ui.checkbox(&mut self.state.edit_mode, "Edit Mode");
            if self.state.edit_mode {
                ui.label("Tools:");
                ui.radio_value(
                    &mut self.state.selected_tool,
                    crate::game::EditorTool::Pin,
                    "Pin",
                );
                ui.radio_value(
                    &mut self.state.selected_tool,
                    crate::game::EditorTool::Wall,
                    "Wall (Drag)",
                );
                ui.radio_value(
                    &mut self.state.selected_tool,
                    crate::game::EditorTool::Eraser,
                    "Eraser",
                );
                ui.checkbox(&mut self.state.editor_grid_snap, "Grid Snap");

                ui.label(egui::RichText::new("Drag to create walls.").small());
            }

            ui.separator();
            ui.label("Winning Condition:");
            ui.radio_value(
                &mut self.state.winning_condition,
                crate::game::WinningCondition::First,
                "First to Arrive",
            );
            ui.radio_value(
                &mut self.state.winning_condition,
                crate::game::WinningCondition::Last,
                "Last to Arrive",
            );

            ui.separator();
            ui.label(format!("Balls Active: {}", self.state.balls.len()));
            ui.label(format!("Finished: {}", self.state.finished_balls.len()));

            if !self.state.finished_balls.is_empty() {
                ui.separator();
                ui.label("Results:");
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for (i, ball) in self.state.finished_balls.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}.", i + 1));

                                // Animation Logic
                                let time = ctx.input(|i| i.time);
                                let age = time - ball.finished_at;
                                let mut color = egui::Color32::from_rgb(
                                    ball.color[0],
                                    ball.color[1],
                                    ball.color[2],
                                );
                                let mut size = 14.0; // Default size approximation

                                if age < 0.5 {
                                    // Flash Effect (White -> Color) and Pop (Big -> Normal)
                                    let t = (age / 0.5) as f32; // 0.0 to 1.0 over 0.5s

                                    // Lerp White -> Target
                                    let w = 255.0 * (1.0 - t);
                                    let r = (ball.color[0] as f32 * t + w).clamp(0.0, 255.0) as u8;
                                    let g = (ball.color[1] as f32 * t + w).clamp(0.0, 255.0) as u8;
                                    let b = (ball.color[2] as f32 * t + w).clamp(0.0, 255.0) as u8;
                                    color = egui::Color32::from_rgb(r, g, b);

                                    // Pop Size
                                    size = 14.0 + 10.0 * (1.0 - t).max(0.0);

                                    ui.ctx().request_repaint(); // Continue animation
                                }

                                ui.label(egui::RichText::new(&ball.name).size(size).color(color));
                            });
                        }
                    });

                // Show Winner
                ui.separator();
                let winner_idx = match self.state.winning_condition {
                    crate::game::WinningCondition::First => {
                        if !self.state.finished_balls.is_empty() {
                            Some(0)
                        } else {
                            None
                        }
                    }
                    crate::game::WinningCondition::Last => {
                        if self.state.balls.is_empty() && !self.state.finished_balls.is_empty() {
                            Some(self.state.finished_balls.len() - 1)
                        } else {
                            None
                        }
                    }
                };

                if let Some(idx) = winner_idx {
                    let winner = &self.state.finished_balls[idx];
                    ui.label(
                        egui::RichText::new(format!("WINNER: {}", winner.name))
                            .size(20.0)
                            .strong()
                            .color(egui::Color32::GREEN),
                    );
                }
            }
        });

        // Main Canvas
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) =
                ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());

            // Dark Background for Neon Contrast
            painter.rect_filled(response.rect, 0.0, egui::Color32::from_rgb(10, 10, 15));

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
                        if self.state.selected_tool == crate::game::EditorTool::Pin
                            || self.state.selected_tool == crate::game::EditorTool::Eraser
                        {
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
                            painter.line_segment(
                                [start_screen, end_screen],
                                egui::Stroke::new(2.0, egui::Color32::YELLOW),
                            );
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
                    // Color logic
                    let mut is_event = false;
                    let mut shape_id = 0;

                    let color = if (collider.user_data >> 64) & 1 == 1 {
                        is_event = true;
                        let r = (collider.user_data >> 48) as u8;
                        let g = (collider.user_data >> 40) as u8;
                        let b = (collider.user_data >> 32) as u8;
                        shape_id = collider.user_data as u8;
                        egui::Color32::from_rgb(r, g, b)
                    } else if collider.user_data == 11 {
                        egui::Color32::from_rgb(0, 100, 255) // Level 1: Blue
                    } else if collider.user_data == 12 {
                        egui::Color32::from_rgb(50, 255, 50) // Level 2: Green
                    } else if collider.user_data == 13 {
                        egui::Color32::from_rgb(255, 255, 0) // Level 3: Yellow
                    } else if collider.user_data == 14 {
                        egui::Color32::from_rgb(255, 165, 0) // Level 4: Orange
                    } else if collider.user_data == 15 {
                        egui::Color32::from_rgb(255, 50, 50) // Level 5: Red
                    } else if collider.user_data == 99 {
                        egui::Color32::from_rgb(0, 255, 255)
                    } else {
                        egui::Color32::GRAY
                    };

                    // Glow Effect
                    let glow_color =
                        egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 100);

                    if is_event && shape_id == 3 {
                        // DRAW STAR
                        // Center: translation.x, translation.y
                        // Outer Radius: ball.radius
                        // Inner Radius: ball.radius * 0.5
                        // 5 points
                        let cx = translation.x;
                        let cy = translation.y;
                        let outer_r = radius;
                        let inner_r = radius * 0.4;
                        let rotation = collider.rotation().angle();

                        let mut points = Vec::new();
                        for i in 0..10 {
                            let angle = rotation + (i as f32 * std::f32::consts::PI / 5.0)
                                - std::f32::consts::PI / 2.0;
                            let r = if i % 2 == 0 { outer_r } else { inner_r };
                            let px = cx + r * angle.cos();
                            let py = cy + r * angle.sin();
                            points.push(to_screen(px, py));
                        }

                        painter.add(egui::Shape::convex_polygon(
                            points,
                            color,
                            egui::Stroke::new(2.0, glow_color),
                        ));
                    } else {
                        // DRAW BALL / CIRCLE
                        painter.circle_filled(
                            to_screen(translation.x, translation.y),
                            radius * 2.0, // Larger glow
                            glow_color,
                        );
                        painter.circle_filled(
                            to_screen(translation.x, translation.y),
                            radius,
                            color,
                        );
                    }
                } else if let Some(cuboid) = shape.as_cuboid() {
                    let half_extents = cuboid.half_extents;
                    let rotation = collider.rotation();
                    let angle = rotation.angle();

                    // Color logic based on user_data
                    // Color logic based on user_data
                    // Color logic based on user_data
                    // Color logic based on user_data
                    let color = if (collider.user_data >> 64) & 1 == 1 {
                        let r = (collider.user_data >> 48) as u8;
                        let g = (collider.user_data >> 40) as u8;
                        let b = (collider.user_data >> 32) as u8;
                        egui::Color32::from_rgb(r, g, b)
                    } else if collider.user_data == 11 {
                        egui::Color32::from_rgb(0, 100, 255) // Blue
                    } else if collider.user_data == 12 {
                        egui::Color32::from_rgb(50, 255, 50) // Green
                    } else if collider.user_data == 13 {
                        egui::Color32::from_rgb(255, 255, 0) // Yellow
                    } else if collider.user_data == 14 {
                        egui::Color32::from_rgb(255, 165, 0) // Orange
                    } else if collider.user_data == 15 {
                        egui::Color32::from_rgb(255, 50, 50) // Red
                    } else if collider.user_data == 99 {
                        egui::Color32::from_rgb(0, 255, 255) // Cyan for Goal
                    } else if collider.user_data == 21 {
                        egui::Color32::from_rgb(0, 255, 255) // Slow - Cyan
                    } else if collider.user_data == 22 {
                        egui::Color32::from_rgb(255, 0, 255) // Normal - Magenta
                    } else if collider.user_data == 23 {
                        egui::Color32::from_rgb(128, 0, 128) // Fast - Purple
                    } else {
                        egui::Color32::DARK_GRAY
                    };

                    // If no rotation, draw aligned rect
                    if angle.abs() < 0.001 {
                        let rect_min = to_screen(
                            translation.x - half_extents.x,
                            translation.y + half_extents.y,
                        );
                        let rect_max = to_screen(
                            translation.x + half_extents.x,
                            translation.y - half_extents.y,
                        );
                        painter.rect_filled(
                            egui::Rect::from_min_max(rect_min, rect_max),
                            0.0,
                            color,
                        );

                        // Glow for Axis-Aligned Walls
                        let glow_color = egui::Color32::from_rgba_unmultiplied(
                            color.r(),
                            color.g(),
                            color.b(),
                            50,
                        );
                        painter.rect_stroke(
                            egui::Rect::from_min_max(rect_min, rect_max),
                            2.0,
                            egui::Stroke::new(4.0, glow_color),
                        );
                    } else {
                        // Rotated rect
                        let mut points = Vec::new();
                        // Local corners: (+x,+y), (-x,+y), (-x,-y), (+x,-y)
                        // But Rapier 2D cuboid is half_extents
                        let hx = half_extents.x;
                        let hy = half_extents.y;
                        let corners = [
                            point![-hx, -hy],
                            point![hx, -hy],
                            point![hx, hy],
                            point![-hx, hy],
                        ];

                        let transform = collider.position();
                        for p in corners {
                            let world_p = transform * p;
                            points.push(to_screen(world_p.x, world_p.y));
                        }

                        let glow_color = egui::Color32::from_rgba_unmultiplied(
                            color.r(),
                            color.g(),
                            color.b(),
                            50,
                        );
                        painter.add(egui::Shape::convex_polygon(
                            points,
                            color,
                            egui::Stroke::new(2.0, glow_color),
                        ));
                    }
                } else if let Some(tri) = shape.as_triangle() {
                    // Triangle Rendering
                    let color = if (collider.user_data >> 64) & 1 == 1 {
                        let r = (collider.user_data >> 48) as u8;
                        let g = (collider.user_data >> 40) as u8;
                        let b = (collider.user_data >> 32) as u8;
                        egui::Color32::from_rgb(r, g, b)
                    } else {
                        egui::Color32::YELLOW // Fallback
                    };

                    let a = tri.a;
                    let b = tri.b;
                    let c = tri.c;

                    let transform = collider.position();
                    let p1 = transform * a;
                    let p2 = transform * b;
                    let p3 = transform * c;

                    let pts = vec![
                        to_screen(p1.x, p1.y),
                        to_screen(p2.x, p2.y),
                        to_screen(p3.x, p3.y),
                    ];

                    let glow_color =
                        egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 50);
                    painter.add(egui::Shape::convex_polygon(
                        pts,
                        color,
                        egui::Stroke::new(2.0, glow_color),
                    ));
                }
            }

            // Particles
            for p in &self.state.particles {
                let pos = to_screen(p.x, p.y);
                let alpha = (p.life * 255.0).clamp(0.0, 255.0) as u8;
                let color = egui::Color32::from_rgba_unmultiplied(
                    p.color[0], p.color[1], p.color[2], alpha,
                );

                painter.circle_filled(pos, 2.0, color);
            }

            // Draw Balls
            for ball in &self.state.balls {
                let ball_handle = ball.handle;
                if let Some(rb) = self.state.physics.rigid_body_set.get(ball_handle) {
                    let pos = rb.translation();
                    let screen_pos = to_screen(pos.x, pos.y);
                    let color =
                        egui::Color32::from_rgb(ball.color[0], ball.color[1], ball.color[2]);

                    // Ball Glow
                    let glow_color = egui::Color32::from_rgba_unmultiplied(
                        ball.color[0],
                        ball.color[1],
                        ball.color[2],
                        128,
                    );
                    painter.circle_filled(screen_pos, 12.0, glow_color);

                    painter.circle(
                        screen_pos,
                        8.0,
                        color,
                        egui::Stroke::new(1.5, egui::Color32::WHITE), // Bright Outline
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
                        text_color,
                    );
                }
            }
        });
    }
}
