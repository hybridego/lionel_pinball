use rapier2d::prelude::*;

pub struct PhysicsEngine {
    pub gravity: Vector<f32>,
    pub integration_parameters: IntegrationParameters,
    pub island_manager: IslandManager,
    pub broad_phase: BroadPhaseMultiSap,
    pub narrow_phase: NarrowPhase,
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub physics_pipeline: PhysicsPipeline,
    pub query_pipeline: QueryPipeline,
    pub collision_recv: crossbeam_channel::Receiver<CollisionEvent>,
    #[allow(dead_code)]
    pub contact_force_recv: crossbeam_channel::Receiver<ContactForceEvent>,
    pub event_handler: ChannelEventCollector,
}

impl PhysicsEngine {
    pub fn new() -> Self {
        let gravity = vector![0.0, -9.81];
        let (collision_send, collision_recv) = crossbeam_channel::unbounded();
        let (contact_force_send, contact_force_recv) = crossbeam_channel::unbounded();
        let event_handler = ChannelEventCollector::new(collision_send, contact_force_send);

        Self {
            gravity,
            integration_parameters: IntegrationParameters::default(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhaseMultiSap::new(),
            narrow_phase: NarrowPhase::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            physics_pipeline: PhysicsPipeline::new(),
            query_pipeline: QueryPipeline::new(),
            event_handler,
            collision_recv,
            contact_force_recv,
        }
    }

    pub fn step(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &self.event_handler,
        );
    }

    pub fn drain_collision_events(&mut self) -> Vec<CollisionEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.collision_recv.try_recv() {
            events.push(event);
        }
        events
    }
}
