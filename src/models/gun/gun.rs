use bevy::{
    prelude::{Component, EventReader, EventWriter, Plugin, Query, Res},
    reflect::{FromReflect, Reflect},
};
use bevy_rapier3d::{na::Vector3, prelude::Real};
use polars::df;
use polars::prelude::NamedFrom;

use crate::framework::{
    data_collection::records::{Record, UpdateRecordEvent},
    plugins::base_plugin::WorldTimer,
    ps_component::PSComponent,
    py_modules::entity_builder::Entity,
};
use pscomp_derive::PSComponent;
use pyo3::{pyclass, pymethods};

#[pyclass]
#[derive(Component, Reflect, FromReflect, Default, Clone, PSComponent)]
struct Gun {
    ammo_count: u32,
}

#[pymethods]
impl Gun {
    #[new]
    fn new(ammo_count: u32) -> Self {
        Gun { ammo_count }
    }

    pub fn attach_to_entity(&self, e: &mut Entity) -> pyo3::PyResult<Entity> {
        let res = self.clone()._attach_to_entity(e.to_owned());
        Ok(res)
    }
}

fn gun_update_record_event(
    //Query for testmodel and record
    test_models: Query<(&Gun, &Record)>,
    // EventWriter will take the event we construct and write to the system to be picked up later
    mut record_updates: EventWriter<UpdateRecordEvent>,
    // only here to have the time in one of the dataframe columns
    world_timer: Res<WorldTimer>,
) {
    //iterate over the results from the query
    for (t, record) in test_models.iter() {
        // construct dataframe to append with the df!() macro from polars, returns a Result so unwrap for now
        let new_row =
            polars::df!["Time" => [world_timer.timer.elapsed_secs()], "AmmoCount" => [t.ammo_count]]
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
    guns: Query<(&Record, &Gun)>,
    mut fire_missions: EventReader<FireMission>,
    mut record_updates: EventWriter<UpdateRecordEvent>,
) {
    for fm in fire_missions.iter() {
        let table_name = format!("FireMissions");
        let new_row = df!["Time" => [fm.time], 
                                        "X" => [fm.coordinates.x],
                                        "Y" => [fm.coordinates.y],
                                        "Z" => [fm.coordinates.z]]
        .unwrap();

        // record_updates.send(UpdateRecordEvent {
        //     record: fm.record.dataframes.clone(),
        //     table_name,
        //     new_row,
        // });
    }
}

fn send_fire_mission(
    world_timer: Res<WorldTimer>,
    mut outgoing_missions: EventWriter<FireMission>,
) {
    let fm = FireMission {
        time: world_timer.timer.elapsed_secs(),
        coordinates: Vector3::new(100.0, 4.0, 100.0),
    };
    outgoing_missions.send(fm);
}

struct FireMission {
    time: Real,
    coordinates: Vector3<Real>,
}

struct GunPlugin;

impl Plugin for GunPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<Gun>();
        app.add_event::<FireMission>();
        app.add_system(read_fire_mission);
        app.add_system(send_fire_mission);
        app.add_system(gun_update_record_event);
    }
}
