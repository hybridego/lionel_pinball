use crate::game::physics::PhysicsEngine;
use rand::Rng;
use rapier2d::prelude::*;

fn get_elasticity_props(level: u8) -> (f32, u128) {
    match level {
        1 => (0.8, 11), // Blue - Low
        2 => (1.0, 12), // Green - Med
        3 => (1.5, 13), // Yellow - High
        4 => (2.0, 14), // Orange - Super
        5 => (3.0, 15), // Red - Extreme
        _ => (0.8, 11), // Default
    }
}

fn get_windmill_props(type_idx: u8) -> (f32, u128) {
    match type_idx {
        0 => (1.0, 21), // Slow - Cyan
        1 => (3.0, 22), // Normal - Magenta
        2 => (5.0, 23), // Fast - Purple
        _ => (3.0, 22),
    }
}

pub fn create_map(physics: &mut PhysicsEngine, width: f32, height: f32) {
    // Walls
    create_walls(physics, width, height);

    // Bottom Area Obstacles (Seesaws & Bumpers)
    create_bottom_obstacles(physics, width, height);

    // Default Pins
    create_pins(physics, width, height);
}

pub fn create_walls(physics: &mut PhysicsEngine, width: f32, height: f32) {
    // 1. External Walls (Left/Right)
    // Extend walls much higher to prevent escaping (e.g., total height)
    let _wall_h = height;
    let _wall_y_offset = height / 4.0; // Shift up because center is 0.0, height is 800 (so -400 to 400). default wall was height/2 (400) centered at 0 covers -200 to 200 ??? Wait.
                                       // Collider cuboid args are half-extents.
                                       // Original: proper wall height/2.0 as half-extent => total height 'height'.
                                       // Centered at 0.0, it covers -height/2 to height/2. (-400 to 400).
                                       // If balls spawn at top (~380), and bounce up, they might go over 400.
                                       // Let's make walls taller.

    // Left Wall
    // Center at -width/2 - 40.0. Half-extent 50.0. Inner edge = (-250 - 40) + 50 = -240.
    // Original: Center -250. Half-extent 10. Inner edge = -240.
    // So shift center by -40.0.
    // Left Wall (Massive to cover outside)
    // Inner edge at -width/2.0 + 10.0 (-240.0)
    // Extent 2000. Center = -240 - 2000 = -2240
    let wall_thickness = 2000.0;
    let inner_edge_offset = 10.0; // Overlap slightly into "margin" area?
                                  // Original inner edge was -240. width/2 is 250. -250 + 10 = -240.

    let left_inner = -width / 2.0 + inner_edge_offset;
    let left_center = left_inner - wall_thickness;

    let collider = ColliderBuilder::cuboid(wall_thickness, 3000.0) // Tall enough
        .translation(vector![left_center, 0.0])
        .friction(0.0)
        .collision_groups(InteractionGroups::new(super::GROUP_MAP, super::GROUP_BALL))
        .build();
    physics.collider_set.insert(collider);

    // Right Wall
    // Right Wall
    let right_inner = width / 2.0 - inner_edge_offset;
    let right_center = right_inner + wall_thickness;

    let collider = ColliderBuilder::cuboid(wall_thickness, 3000.0)
        .translation(vector![right_center, 0.0])
        .friction(0.0)
        .collision_groups(InteractionGroups::new(super::GROUP_MAP, super::GROUP_BALL))
        .build();
    physics.collider_set.insert(collider);

    // Wall Bumpers (Deflectors)
    // Small rotated boxes along the walls to kick balls back in
    // Wall Bumpers (Deflectors)
    // Small rotated boxes along the walls to kick balls back in
    let mut rng = rand::thread_rng();
    for i in 0..6 {
        let y = -200.0 + (i as f32) * 100.0;

        // Random restitution: 3.0 or 5.0
        // Random elasticity for wall bumpers (High to Extreme: 3-5)
        let level = rng.gen_range(3..=5);
        let (restitution, user_data) = get_elasticity_props(level);

        // Left Bumper
        let collider = ColliderBuilder::cuboid(10.0, 10.0) // Doubled size (was 5.0)
            .translation(vector![-width / 2.0 + 8.0, y]) // Protruding slightly
            .rotation(0.785) // 45 degrees
            .restitution(restitution) // Bouncy
            .friction(0.0)
            .user_data(user_data)
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .collision_groups(InteractionGroups::new(super::GROUP_MAP, super::GROUP_BALL))
            .build();
        physics.collider_set.insert(collider);

        let level_right = rng.gen_range(3..=5);
        let (restitution_right, user_data_right) = get_elasticity_props(level_right);

        // Right Bumper
        let collider = ColliderBuilder::cuboid(10.0, 10.0)
            .translation(vector![width / 2.0 - 8.0, y])
            .rotation(0.785)
            .restitution(restitution_right)
            .friction(0.0)
            .user_data(user_data_right)
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .collision_groups(InteractionGroups::new(super::GROUP_MAP, super::GROUP_BALL))
            .build();
        physics.collider_set.insert(collider);
    }

    // Top Wall (Lid)
    // Center: height/2 + 10 (original). Half-extent 10. Inner edge = h/2.
    // New: Half-extent 50. Inner edge h/2. Center = h/2 + 50.
    // Top Wall (Lid) - Moved Up
    // Original Inner: height/2.0 (400.0)
    // New Inner: height/2.0 + 300.0 (700.0) - Giving lots of headroom
    let top_inner = height / 2.0;
    let top_thickness = 700.0;
    let top_center = top_inner + top_thickness;

    let collider = ColliderBuilder::cuboid(width * 2.0, top_thickness) // Wide enough
        .translation(vector![0.0, top_center])
        .friction(0.0)
        .collision_groups(InteractionGroups::new(super::GROUP_MAP, super::GROUP_BALL))
        .build();
    physics.collider_set.insert(collider);

    // 2. Funnel / Guide Geometry
    // We want a clear funnel: \ / leading to a narrow chute | |
    // AND it must be perfectly connected to the side walls so nothing escapes.

    let chute_height = 40.0; // Shortened from 80.0
    let exit_gap = 22.0; // Slightly narrower? Or keep same.

    // Y Position:
    // Bottom of chute = absolute bottom + margin
    // Top of chute / Bottom of Funnel = Bottom of chute + chute_height
    // Top of Funnel = Bottom of Funnel + funnel_height
    // To make funnel steeper, we want more vertical distance for the same horizontal change.
    // If we shorten chute, the bottom of funnel drops.
    // So if we keep the "Top of Funnel" fixed or increase its height, it gets steeper.
    // Old funnel_height was 100.
    // Old chute_top_y = bottom_y + 80.
    // Old funnel_top_y = bottom_y + 80 + 100 = bottom_y + 180.
    // New chute_top_y = bottom_y + 40.
    // To keep funnel_top_y at bottom_y + 180 (same top start), we need funnel_height = 140.
    // This effectively extends the funnel DOWNWARDS into the space freed by the shorter chute.
    let funnel_height = 140.0;

    let bottom_y = -height / 2.0 + 20.0;
    let chute_top_y = bottom_y + chute_height;
    let funnel_top_y = chute_top_y + funnel_height;

    // --- 2a. Vertical Chute Walls ---
    // Left Chute Wall
    let collider = ColliderBuilder::cuboid(5.0, chute_height / 2.0)
        .translation(vector![
            -(exit_gap / 2.0 + 5.0),
            bottom_y + chute_height / 2.0
        ])
        .friction(0.0)
        .collision_groups(InteractionGroups::new(super::GROUP_MAP, super::GROUP_BALL))
        .build();
    physics.collider_set.insert(collider);

    // Right Chute Wall
    let collider = ColliderBuilder::cuboid(5.0, chute_height / 2.0)
        .translation(vector![
            (exit_gap / 2.0 + 5.0),
            bottom_y + chute_height / 2.0
        ])
        .friction(0.0)
        .collision_groups(InteractionGroups::new(super::GROUP_MAP, super::GROUP_BALL))
        .build();
    physics.collider_set.insert(collider);

    // --- 2b. Angled Funnel Walls ---
    // Connect Point A (Side Wall Inner Edge, Funnel Top Y) to Point B (Chute Outer Edge, Chute Top Y)
    // Side Wall Thickness = 10.0 (half) -> 20.0 wide. Center at width/2. Inner edge = width/2 - 10.0.
    // Chute Wall Thickness = 5.0 (half) -> 10.0 wide. Center at exit_gap/2 + 5.0. Outer edge = exit_gap/2 + 10.0 ???
    // Actually, let's just connect center-points or edges carefully.
    // Let's connect the Inner Edge of Side Wall to the Top Edge of Chute Wall.

    let side_wall_inner_x = width / 2.0 - 10.0;
    let _chute_wall_center_x = exit_gap / 2.0 + 5.0; // Center of 10-wide block
                                                     // We want to block from Side Wall to Chute Wall.
                                                     // Let's define the funnel wall as a rectangle connecting:
                                                     // P1: (side_wall_inner_x, funnel_top_y)
                                                     // P2: (chute_wall_center_x, chute_top_y) -> Actually, let's overlap slightly to avoid leaks.

    // Left Funnel Geometry
    let p1_x = -side_wall_inner_x;
    let p1_y = funnel_top_y;
    let p2_x = -(exit_gap / 2.0 + 5.0); // Center of left chute wall
    let p2_y = chute_top_y + 5.0; // Slightly overlapping top of chute

    let dx = p2_x - p1_x;
    let dy = p2_y - p1_y;
    let length = (dx * dx + dy * dy).sqrt();
    let angle = dy.atan2(dx);
    let cx = (p1_x + p2_x) / 2.0;
    let cy = (p1_y + p2_y) / 2.0;

    let collider = ColliderBuilder::cuboid(length / 2.0, 5.0)
        .translation(vector![cx, cy])
        .rotation(angle)
        .friction(0.0)
        .collision_groups(InteractionGroups::new(super::GROUP_MAP, super::GROUP_BALL))
        .build();
    physics.collider_set.insert(collider);

    // Right Funnel Geometry
    // Mirror X
    let p1_x_r = side_wall_inner_x;
    let p2_x_r = exit_gap / 2.0 + 5.0;

    let dx_r = p2_x_r - p1_x_r;
    let dy_r = p2_y - p1_y; // Same Y
    let length_r = (dx_r * dx_r + dy_r * dy_r).sqrt();
    let angle_r = dy_r.atan2(dx_r);
    let cx_r = (p1_x_r + p2_x_r) / 2.0;
    let cy_r = (p1_y + p2_y) / 2.0;

    let collider = ColliderBuilder::cuboid(length_r / 2.0, 5.0)
        .translation(vector![cx_r, cy_r])
        .rotation(angle_r)
        .friction(0.0)
        .collision_groups(InteractionGroups::new(super::GROUP_MAP, super::GROUP_BALL))
        .build();
    physics.collider_set.insert(collider);

    // 3. Floor (below) - Massive solid block
    // We want the TOP of the floor to be below the goal.
    // Chute ends at bottom_y (-380).
    // Let's place the floor at bottom_y - 20 = -400.
    let floor_thickness = 2000.0;
    let floor_top_y = bottom_y - 20.0;
    let floor_center_y = floor_top_y - floor_thickness / 2.0;

    // Single Massive Floor Block
    // Spans the entire width (and more)
    let collider = ColliderBuilder::cuboid(width * 2.0, floor_thickness / 2.0)
        .translation(vector![0.0, floor_center_y])
        .friction(0.0)
        .collision_groups(InteractionGroups::new(super::GROUP_MAP, super::GROUP_BALL))
        .build();
    physics.collider_set.insert(collider);

    // 4. Goal Sensor / Indicator
    // Located in the gap between chute end (-380) and floor top (-400).
    // Height 10. Center at -390.
    let goal_h = 10.0;
    let goal_y = bottom_y - 10.0; // -390.0

    let collider = ColliderBuilder::cuboid(exit_gap / 2.0, goal_h / 2.0)
        .translation(vector![0.0, goal_y])
        .sensor(true)
        .user_data(99) // Special ID for Goal Color
        .build();
    physics.collider_set.insert(collider);
}

pub fn create_pins(physics: &mut PhysicsEngine, width: f32, height: f32) {
    // Simple grid of pins
    let rows = 8; // Adjusted for 1.2x spacing (was 7 for 1.5x)
    let cols = 12; // Adjusted for 1.2x spacing (was 10 for 1.5x)
    let pin_radius = 5.0;

    // Safety Margin: Bumpers need ~20 space. Wall is at 250. Inner Bumper edge ~230.
    // Pin should be at max ~210.
    // Let's us 50 margin. Width 500. Margin 50 -> 400 space.
    // -200 to 200.
    let margin = 50.0;
    let grid_width = width - 2.0 * margin;

    let spacing_x = grid_width / (cols - 1) as f32; // cols-1 because we want to span exactly
    let spacing_y = (height / 2.0) / rows as f32;

    let mut spinner_count = 0;

    for r in 0..rows {
        for c in 0..cols {
            let x = -grid_width / 2.0
                + (c as f32 * spacing_x)
                + if r % 2 == 0 { 0.0 } else { spacing_x / 2.0 };
            // Shift offset row back to center if needed, or just let it be.
            // If r% odd, we add half spacing.
            // Let's cap X?
            if x.abs() > grid_width / 2.0 + 10.0 {
                continue;
            }

            let y = height / 2.0 - 100.0 - (r as f32 * spacing_y);

            // Random chance for a spinner instead of a pin
            let mut rng = rand::thread_rng();
            if spinner_count < 5 && rng.gen_bool(0.05) {
                // 5% chance, max 5
                spinner_count += 1;
                let spinner_len = if rng.gen_bool(0.5) { 40.0 } else { 80.0 };

                // Random Speed Type
                let type_idx = rng.gen_range(0..3);
                let (speed_mag, user_data) = get_windmill_props(type_idx);

                let speed = if rng.gen_bool(0.5) {
                    speed_mag
                } else {
                    -speed_mag
                };
                create_spinner(physics, x, y, spinner_len, speed, user_data);
                continue;
            }

            // Pin Type Logic (Levels 1-5)
            let roll = rng.gen_range(0..100);
            let level = if roll < 20 {
                1
            }
            // 20% Level 1
            else if roll < 50 {
                2
            }
            // 30% Level 2
            else if roll < 80 {
                3
            }
            // 30% Level 3
            else if roll < 95 {
                4
            }
            // 15% Level 4
            else {
                5
            }; // 5% Level 5

            let (restitution, user_data) = get_elasticity_props(level);

            let collider = ColliderBuilder::ball(pin_radius)
                .translation(vector![x, y])
                .restitution(restitution)
                .friction(0.0)
                .user_data(user_data)
                .active_events(ActiveEvents::COLLISION_EVENTS)
                .collision_groups(InteractionGroups::new(super::GROUP_MAP, super::GROUP_BALL))
                .build();

            physics.collider_set.insert(collider);
        }
    }
}

pub fn create_spinner(
    physics: &mut PhysicsEngine,
    x: f32,
    y: f32,
    length: f32,
    speed: f32,
    user_data: u128,
) {
    // 1. Static Anchor (invisible or small)
    let anchor_rb = RigidBodyBuilder::fixed().translation(vector![x, y]).build();
    let anchor_handle = physics.rigid_body_set.insert(anchor_rb);

    // 2. Dynamic Blade
    let blade_rb = RigidBodyBuilder::dynamic()
        .translation(vector![x, y])
        .build();
    let blade_handle = physics.rigid_body_set.insert(blade_rb);

    let collider = ColliderBuilder::cuboid(length / 2.0, 5.0)
        .restitution(0.5)
        .density(2.0)
        .friction(0.0)
        .user_data(user_data)
        .collision_groups(InteractionGroups::new(
            super::GROUP_SPINNER,
            super::GROUP_BALL,
        )) // Spinner hits ONLY balls
        .build();
    physics
        .collider_set
        .insert_with_parent(collider, blade_handle, &mut physics.rigid_body_set);

    // Cross blade (Vertical if first is horizontal)
    let collider2 = ColliderBuilder::cuboid(5.0, length / 2.0)
        .restitution(0.5)
        .density(2.0)
        .user_data(user_data)
        .active_events(ActiveEvents::COLLISION_EVENTS)
        .collision_groups(InteractionGroups::new(
            super::GROUP_SPINNER,
            super::GROUP_BALL,
        ))
        .build();
    physics
        .collider_set
        .insert_with_parent(collider2, blade_handle, &mut physics.rigid_body_set);

    // 3. Joint with Motor
    // In rapier, we can use specific joint builders
    let joint = RevoluteJointBuilder::new()
        .local_anchor1(point![0.0, 0.0])
        .local_anchor2(point![0.0, 0.0])
        .motor_velocity(speed, 1.0e8); // target velocity, max factor (Stronger)

    physics
        .impulse_joint_set
        .insert(anchor_handle, blade_handle, joint, true);
}

pub fn create_seesaw(physics: &mut PhysicsEngine, x: f32, y: f32, width: f32) {
    // 1. Static Pivot
    let pivot = RigidBodyBuilder::fixed().translation(vector![x, y]).build();
    let pivot_handle = physics.rigid_body_set.insert(pivot);

    // 2. Dynamic Plank
    let plank = RigidBodyBuilder::dynamic()
        .translation(vector![x, y])
        .build();
    let plank_handle = physics.rigid_body_set.insert(plank);

    let collider = ColliderBuilder::cuboid(width / 2.0, 3.0)
        .restitution(0.2)
        .friction(0.5) // Grip
        .density(2.0)
        .collision_groups(InteractionGroups::new(super::GROUP_MAP, super::GROUP_BALL))
        .build();
    physics
        .collider_set
        .insert_with_parent(collider, plank_handle, &mut physics.rigid_body_set);

    // 3. Joint (Free rotation)
    let joint = RevoluteJointBuilder::new()
        .local_anchor1(point![0.0, 0.0])
        .local_anchor2(point![0.0, 0.0]);
    // .limits([-0.5, 0.5]); // Removed for 360 degree rotation

    physics
        .impulse_joint_set
        .insert(pivot_handle, plank_handle, joint, true);
}

pub fn create_bottom_obstacles(physics: &mut PhysicsEngine, _width: f32, _height: f32) {
    // Coordinate reference:
    // Funnel Top is roughly where pin grid ends.
    // Grid y: height / 2.0 - 100.0 - (10 * spacing) ~ 200 - 100 - (10*40) = -300 ?
    // Let's check calculations.
    // Height=800. Top=400, Bottom=-400.
    // Funnel Top Y in create_walls = chute_top + 100
    // chute_top = bottom_y + 80.
    // bottom_y = -400 + 20 = -380.
    // chute_top = -300.
    // funnel_top = -200.

    // So the gap is roughly -120 to -280.

    // Seesaws
    // Moved up to avoid blocking goal
    // Two top
    create_seesaw(physics, -80.0, -160.0, 70.0);
    create_seesaw(physics, 80.0, -160.0, 70.0);

    // One bottom center
    create_seesaw(physics, 0.0, -220.0, 80.0);

    // Funnel Bumpers (Elastic Pins on Funnel Walls)
    // Funnel walls go from Side(-240, -200) to Chute(-16, -340).
    // Midpoint approx Y = -270.
    // X at Y=-270 is approx +/- 128.

    let mut rng = rand::thread_rng();

    // Left Slope Bumper
    let level_left = rng.gen_range(3..=5);
    let (restitution_left, user_data_left) = get_elasticity_props(level_left);

    let collider = ColliderBuilder::ball(10.4)
        .translation(vector![-128.0, -270.0])
        .restitution(restitution_left)
        .user_data(user_data_left)
        .active_events(ActiveEvents::COLLISION_EVENTS)
        .collision_groups(InteractionGroups::new(super::GROUP_MAP, super::GROUP_BALL))
        .build();
    physics.collider_set.insert(collider);

    // Right Slope Bumper
    let level_right = rng.gen_range(3..=5);
    let (restitution_right, user_data_right) = get_elasticity_props(level_right);

    let collider = ColliderBuilder::ball(10.4)
        .translation(vector![128.0, -270.0])
        .restitution(restitution_right)
        .user_data(user_data_right)
        .active_events(ActiveEvents::COLLISION_EVENTS)
        .collision_groups(InteractionGroups::new(super::GROUP_MAP, super::GROUP_BALL))
        .build();
    physics.collider_set.insert(collider);
}
