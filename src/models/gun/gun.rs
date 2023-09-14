use bevy::prelude::Event;
use bevy::{
    prelude::{
        Bundle, Commands, Component, EventReader, EventWriter, Plugin, Query, ReflectComponent,
        Res, Transform, With,
    },
    reflect::Reflect,
    transform::TransformBundle,
};
use bevy_rapier3d::dynamics::ExternalForce;
use bevy_rapier3d::prelude::{
    Ccd, Collider, ColliderMassProperties, ExternalImpulse, GravityScale, Real, RigidBody, Velocity,
};
use glam::Quat;
use polars::{df, prelude::NamedFrom};

use crate::framework::{
    data_collection::records::{Record, UpdateRecordEvent},
    plugins::base_plugin::WorldTimer,
    ps_component::PSComponent,
    py_modules::entity_builder::Entity,
};
use pscomp_derive::PSComponent;
use pyo3::{pyclass, pymethods};

#[pyclass]
#[derive(Component, Reflect, Default, Clone, PSComponent)]
#[reflect(Component)]
pub struct Gun {
    ammo_count: u32,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CanFire;

#[pymethods]
impl Gun {
    #[new]
    fn new(ammo_count: u32) -> Self {
        Gun { ammo_count }
    }

    pub fn attach_to_entity(&self, e: &mut Entity) -> pyo3::PyResult<Entity> {
        e.components.push(Box::new(CanFire));
        let res = self.clone()._attach_to_entity(e.to_owned());
        Ok(res)
    }
}

impl Gun {
    fn fire(&mut self, fm: &FireMission, t: &Transform) -> impl Bundle {
        let north = Quat::from_xyzw(1.0, 0.0, 0.0, 0.0);

        let fire_direction = (fm.elevation.inverse() * fm.azimuth).inverse()
            * north
            * (fm.elevation.inverse() * fm.azimuth);

        // construct the direction of fire
        let fire_velocity = fire_direction.xyz().normalize() * fm.muzzle_velocity;

        println!("{:?},{:?}", fire_direction, fire_velocity);
        // construct the impulse necessary for the desired muzzle velocity
        let impulse = ExternalImpulse {
            impulse: fire_velocity,
            torque_impulse: (160.0 * fire_direction.to_scaled_axis()),
        };

        let mut transform = Transform::from_translation(t.clone().translation);
        transform.rotate(fire_direction);

        // construct the record component to collect all the data here
        let mut bullet_record = Record::default();
        bullet_record.name = format!("Bullet");
        let bullet = (
            RigidBody::Dynamic,
            Collider::ball(1.0),
            ColliderMassProperties::Mass(100.0),
            impulse,
            ExternalForce::default(),
            Velocity::zero(),
            // Sensor,
            TransformBundle::from(transform),
            bullet_record,
            GravityScale(1.0),
            Ccd::enabled(),
        );

        // println!("{:?}", bullet.7);

        self.ammo_count -= 1;

        bullet
    }
}

fn gun_update_record_event(
    //Query for test model and record
    guns: Query<(&Gun, &Record)>,
    // EventWriter will take the event we construct and write to the system to be picked up later
    mut record_updates: EventWriter<UpdateRecordEvent>,
    // only here to have the time in one of the dataframe columns
    world_timer: Res<WorldTimer>,
) {
    //iterate over the results from the query
    for (t, record) in guns.iter() {
        // construct dataframe to append with the df!() macro from polars, returns a Result so unwrap for now
        let new_row =
            df!["Time" => [world_timer.timer.elapsed_secs()], "AmmoCount" => [t.ammo_count]]
                .unwrap();
        //table_name is the key for this dataframe in the record hashmap
        let table_name = format!("GunAmmo");

        //Construct the UpdateRecordEvent struct, and write it is as an event.
        record_updates.send(UpdateRecordEvent {
            record: record.dataframes.clone(),
            table_name,
            new_row,
        });
    }
}

fn read_fire_mission(
    mut commands: Commands,
    mut guns: Query<(bevy::prelude::Entity, &mut Gun, &Transform, Option<&Record>), With<CanFire>>,
    mut fire_missions: EventReader<FireMission>,
    mut record_updates: EventWriter<UpdateRecordEvent>,
    world_timer: Res<WorldTimer>,
) {
    // Match all guns that can fire with a fire mission by creating a zipped iterator
    // could destructure the iterator at x, but thought it might be more readable to destructure afterwards
    for x in fire_missions.iter().zip(guns.iter_mut()) {
        // destructure into tuple thing
        let fm = x.0;
        let mut gun = x.1;

        //call fire method on the gun
        let bullet = gun.1.fire(fm, gun.2);
        commands.spawn(bullet);
        commands.entity(gun.0).remove::<CanFire>();

        if let Some(r) = gun.3 {
            let table_name = format!("FireMissionsExecuted");
            let new_row = df!["FireMissionIssueTime" => [fm.time], "FireMissionExecutedTime" => [world_timer.timer.elapsed_secs()] ].unwrap();

            let update = UpdateRecordEvent {
                record: r.dataframes.clone(),
                table_name,
                new_row,
            };

            record_updates.send(update);
        }
    }
}

fn send_fire_mission(
    world_timer: Res<WorldTimer>,
    mut outgoing_missions: EventWriter<FireMission>,
) {
    let fm = FireMission {
        time: world_timer.timer.elapsed_secs(),
        muzzle_velocity: 50000.0,
        azimuth: bevy_rapier3d::math::Rot::from_rotation_y(0.0),
        elevation: bevy_rapier3d::math::Rot::from_rotation_z(2.0 * std::f32::consts::PI / 180.0),
    };
    outgoing_missions.send(fm);
}

#[derive(Debug, Event)]
struct FireMission {
    time: Real,
    muzzle_velocity: Real,
    azimuth: bevy_rapier3d::math::Rot,
    elevation: bevy_rapier3d::math::Rot,
}

pub struct GunPlugin;

impl Plugin for GunPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<Gun>();
        app.register_type::<CanFire>();
        app.add_event::<FireMission>();
        app.add_system(read_fire_mission);
        app.add_system(send_fire_mission);
        app.add_system(gun_update_record_event);
    }
}
