use crate::framework::{
    data_collection::records::{Record, UpdateRecordEvent},
    plugins::base_plugin::WorldTimer,
    ps_component::PSComponent,
    py_modules::entity_builder::Entity,
};
use bevy::{
    prelude::{Component, EventWriter, Plugin, Query, ReflectComponent, Res},
    reflect::{FromReflect, Reflect},
};
use polars::prelude::NamedFrom;
use pscomp_derive::PSComponent;
use pyo3::{pyclass, pymethods};

/// Test model demonstrating how to create a component and expose it to python, have it record a table,
/// and have implement some systems for behavior. Also goes over how to package this into a plugin and
/// so that it becomes callable/usable

/// pyclass macro marks this as a class in the python API
/// The derive macro
/// the reflect component macro is required by
#[pyclass]
#[derive(Component, Clone, Reflect, FromReflect, Default, PSComponent)]
#[reflect(Component)]
pub struct TestModel {
    test: String,
}

/// pymethods macro are the python class methods that are callable from python. A seperae impl block can
/// be constructed for pure rust functions
#[pymethods]
impl TestModel {
    /// new macro marks this as the python constructor
    #[new]
    fn new(name: String) -> Self {
        TestModel { test: name }
    }

    /// This method has to be implemented in order for the add_component method of entity_builder::entity
    /// to work properly. A python error will be raised if it is not.
    pub fn attach_to_entity(&self, e: &mut Entity) -> pyo3::PyResult<Entity> {
        let res = self.clone()._attach_to_entity(e.to_owned());
        Ok(res)
    }
}

/// This function queries for entities that have TestModel and Record components,
/// constructs the dataframe to appaned, and constructs the event for updating the
/// appropriate record and writes the event to be picked up by another system
/// This method is a template and can be copied pretty much wholesale
/// To adapt it, change the value of new_row to a new dataframe, and change table_name
/// can be extended to not update every timestep, write a dataframe with any columns you want, etc
pub fn test_model_update_record_event(
    //Query for testmodel and record
    test_models: Query<(&TestModel, &Record)>,
    // EventWriter will take the event we construct and write to the system to be picked up later
    mut record_updates: EventWriter<UpdateRecordEvent>,
    // only here to have the time in one of the dataframe columns
    world_timer: Res<WorldTimer>,
) {
    //iterate over the results from the query
    for (t, record) in test_models.iter() {
        // construct dataframe to append with the df!() macro from polars, returns a Result so unwrap for now
        let new_row =
            polars::df!["Time" => [world_timer.timer.elapsed_secs()], "Value" => [t.test.clone()]]
                .unwrap();
        //table_name is the key for this dataframe in the record hashmap
        let table_name = format!("TestModel");

        //Construct the UpdateRecordEvent struct, and write it is as an event.
        record_updates.send(UpdateRecordEvent {
            record: record.dataframes.clone(),
            table_name,
            new_row,
        });
    }
}

/// declare a unit struct TestPlugin that we can implement the Plugin trait for
/// make public to be used elswhere in the program
pub struct TestPlugin;

/// Plugin trait lets us organize all our data and systems, and then just add this to a plugin group later
/// this is for organization, you can add a bunch of classes to one plugin if you like, or separate all
/// of them out into separate plugins.
///
/// This plugin must then be added to a plugin in plugin_group.rs
impl Plugin for TestPlugin {
    /// the only method this trait needs to implement
    fn build(&self, app: &mut bevy::prelude::App) {
        //add our record update function as a system to the app
        app.add_system(test_model_update_record_event);
        //register the type we made, this is necessary for the Reflect trait to work, you will hit a panic
        //if you don't register the type. This type must also be registered in simulation_builder.rs
        app.register_type::<TestModel>();
    }
}
