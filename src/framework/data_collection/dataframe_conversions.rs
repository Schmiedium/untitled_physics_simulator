use std::collections::HashMap;
use std::sync::RwLock;

use arrow::ffi;
use polars::prelude::*;
use polars::prelude::{ArrayRef, ArrowField};
use polars_arrow::export::arrow;
use pyo3::exceptions::PyValueError;
use pyo3::ffi::Py_uintptr_t;
use pyo3::prelude::*;
use pyo3::{
    types::{IntoPyDict, PyList},
    PyAny, PyObject, PyResult,
};

type DataframeStore = HashMap<String, Arc<RwLock<HashMap<String, DataFrame>>>>;

#[pyclass]
pub struct PySeries {
    #[pyo3(get, set)]
    name: String,
    #[pyo3(get, set)]
    data: PyObject,
}

/// Take an arrow array from python and convert it to a rust arrow array.
/// This operation does not copy data.
fn array_to_rust(arrow_array: &PyAny) -> PyResult<ArrayRef> {
    // prepare a pointer to receive the Array struct
    let array = Box::new(ffi::ArrowArray::empty());
    let schema = Box::new(ffi::ArrowSchema::empty());

    let array_ptr = &*array as *const ffi::ArrowArray;
    let schema_ptr = &*schema as *const ffi::ArrowSchema;

    // make the conversion through PyArrow's private API
    // this changes the pointer's memory and is thus unsafe. In particular, `_export_to_c` can go out of bounds
    arrow_array.call_method1(
        "_export_to_c",
        (array_ptr as Py_uintptr_t, schema_ptr as Py_uintptr_t),
    )?;

    unsafe {
        let field = ffi::import_field_from_c(schema.as_ref()).unwrap();
        let array = ffi::import_array_from_c(*array, field.data_type).unwrap();
        Ok(array)
    }
}

/// Arrow array to Python.
pub(crate) fn to_py_array(py: Python, pyarrow: &PyModule, array: ArrayRef) -> PyResult<PyObject> {
    let schema = Box::new(ffi::export_field_to_c(&ArrowField::new(
        "",
        array.data_type().clone(),
        true,
    )));
    let array = Box::new(ffi::export_array_to_c(array));

    let schema_ptr: *const ffi::ArrowSchema = &*schema;
    let array_ptr: *const ffi::ArrowArray = &*array;

    let array = pyarrow.getattr("Array")?.call_method1(
        "_import_from_c",
        (array_ptr as Py_uintptr_t, schema_ptr as Py_uintptr_t),
    )?;

    Ok(array.to_object(py))
}

pub fn py_series_to_rust_series(series: &PyAny) -> PyResult<Series> {
    // rechunk series so that they have a single arrow array
    let series = series.call_method0("rechunk")?;

    let name = series.getattr("name")?.extract::<String>()?;

    // retrieve pyarrow array
    let array = series.call_method0("to_arrow")?;

    // retrieve rust arrow array
    let array = array_to_rust(array)?;

    Series::try_from((name.as_str(), array)).map_err(|e| PyValueError::new_err(format!("{}", e)))
}

pub fn rust_series_to_py_series(series: &Series) -> PyResult<PyObject> {
    // ensure we have a single chunk
    let series = series.rechunk();
    let array = series.to_arrow(0);

    // acquire the gil
    Python::with_gil(|py| -> PyResult<PyObject> {
        // import pyarrow
        let pyarrow = py.import("pyarrow")?;

        // pyarrow array
        let pyarrow_array = to_py_array(py, pyarrow, array)?;

        // import polars
        let polars = py.import("polars")?;
        let out = polars.call_method1("from_arrow", (pyarrow_array,))?;
        Ok(out.to_object(py))
    })
}

pub fn series_to_arrow(series: &mut Series) -> PyResult<PySeries> {
    let series = series.rechunk();
    Python::with_gil(|py| -> PyResult<PySeries> {
        let pyarrow = py.import("pyarrow")?;
        let py_array = to_py_array(py, pyarrow, series.chunks()[0].clone())?;
        let py_series = PySeries {
            name: series.name().to_string(),
            data: py_array,
        };
        Ok(py_series)
    })
}

/// This is not a bevy system, but a function extracted from main for converting the data collected
/// during the sim into a format that can be pass back to python
pub fn dataframe_hashmap_to_python_dict(dfs: DataframeStore) -> PyResult<PyObject> {
    if dfs.is_empty() {
        println!("QUACK");
        return Python::with_gil(|py| -> PyResult<PyObject> {
            Ok("no data to return".to_object(py))
        });
    }

    // This is a somewhat arcane closure, which will be passed to a map function later
    // takes key, value pair from the dataframes hashmap and returns a tuple of name and python-polars dataframe
    let closure = |item: (String, DataFrame)| -> PyResult<(String, PyObject)> {
        // destructure input tuple
        let df: DataFrame = item.1;
        let key = item.0;

        // need to own names of the columns for iterator purposes
        let names = df.get_column_names_owned();

        // something about iterating over the dataframe to turn it into Apache Arrow Series and column names as Strings
        let (arrows_series_list, names_list): (Vec<PyObject>, Vec<String>) = df
            // generate Vec of Apache Arrow Series from dataframe object
            .columns(&names)
            // unwrap to handle errors. in the future should handle appropriately, but for now will always work
            .unwrap()
            // turn Vec of Apache Arrow Series into an iterator
            .into_iter()
            // generate iterater over tuples of Series with their respective names
            .zip(names.into_iter())
            // convert rust Series to python Series
            .map(|(s, n)| -> (PyObject, String) {
                (
                    //this function was copied was copied from reddit/stackoverflow/github
                    rust_series_to_py_series(s).unwrap(),
                    n,
                )
            })
            //gotta collect the output into a collection before we turn it into the tuple we want
            .collect::<Vec<(PyObject, String)>>()
            // It's a collection now, so we have to call into_iterator because we need ownership I think
            .into_iter()
            // unzip into the data structure we want
            .unzip();

        // This is a python tuple
        // it contains a list of Arrow Series and a List of their names
        let returning_frame = Python::with_gil(|py| -> PyResult<PyObject> {
            let arg = (
                PyList::new(py, arrows_series_list),
                PyList::new(py, names_list),
            );

            // making sure the python environment has polars
            let pl = py.import("polars")?;
            //construct polars DataFrame from Series and their names
            let out = pl.call_method1("DataFrame", arg)?;

            //Return Python formatted valid dataframe
            Ok(out.to_object(py))
        })?;

        Ok((key, returning_frame))
    };

    //End arcane closure

    // iterate over the hashmap passed in and return a python dictionary of names and dataframes
    let keys_values = dfs
        .into_iter()
        .map(|p| -> (String, Vec<(String, PyObject)>) {
            (
                p.0,
                p.1.write()
                    .unwrap()
                    .to_owned()
                    .into_iter()
                    .map(closure)
                    // map that result to the interior of the result, as a python object
                    .map(|py_res| -> (String, PyObject) {
                        match py_res {
                            Ok(x) => x,
                            Err(e) => {
                                let object: Py<PyAny> = Python::with_gil(|py| {
                                    e.print(py);
                                    "quack".to_string().to_object(py)
                                });
                                ("failure to return dataframe".to_string(), object)
                            }
                        }
                    })
                    .collect::<Vec<(String, PyObject)>>(),
            )
        })
        .map(|inner_dict| -> (String, PyObject) {
            // construct inner python dictionary object, and then return it
            (
                inner_dict.0,
                Python::with_gil(|py| -> PyObject {
                    (inner_dict.1.into_py_dict(py)).to_object(py)
                }),
            )
        })
        // .flatten()
        .collect::<Vec<(String, PyObject)>>();

    // construct python dictionary object, and then return it
    Python::with_gil(|py| -> PyResult<PyObject> {
        Ok((keys_values.into_py_dict(py)).to_object(py))
    })
}
