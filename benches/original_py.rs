/*
 AYUDAME/TEMANEJO toolset
--------------------------

 (C) 2024, HLRS, University of Stuttgart
 All rights reserved.
 This software is published under the terms of the BSD license:

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:
    * Redistributions of source code must retain the above copyright
      notice, this list of conditions and the following disclaimer.
    * Redistributions in binary form must reproduce the above copyright
      notice, this list of conditions and the following disclaimer in the
      documentation and/or other materials provided with the distribution.
    * Neither the name of the <organization> nor the
      names of its contributors may be used to endorse or promote products
      derived from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL <COPYRIGHT HOLDER> BE LIABLE FOR ANY
DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
(INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/
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
