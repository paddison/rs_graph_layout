use std::collections::HashMap;

use pyo3::prelude::*;

static PYTHON_FILE: &str = include_str!("python/graph_layout.py");

pub fn graph_layout(edges: Vec<(u32, u32)>) -> PyResult<()> {
    let arg2 = 40;
    let arg3 = false;

    let ret: PyResult<Vec<HashMap<i32, (i32, i32)>>> = Python::with_gil(|py| {
        let fun: Py<PyAny> = PyModule::from_code_bound(py, PYTHON_FILE, "", "")?
            .getattr("graph_layout")?
            .into();

        let args = (edges, arg2, arg3);
        let ret: Vec<HashMap<i32, (i32, i32)>> = fun.call1(py, args)?.extract(py)?;
        Ok(ret)
    });

    if ret.is_err() {
        let err = ret.unwrap_err();
        panic!("{err} something went wrong with python");
    }

    Ok(())
}
