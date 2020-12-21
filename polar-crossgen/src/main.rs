use serde::{Deserialize, Serialize};
use serde_reflection::{Registry, Samples, Tracer, TracerConfig};

use std::fs::File;
use std::io::Write;

use polar_core::events::QueryEvent;
use polar_core::traces::{Node, Trace};
use polar_core::terms::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Obtain the Serde format of `Term`. (In practice, formats are more often read from a file.)
    let mut tracer = Tracer::new(TracerConfig::default());
    let mut samples = Samples::new();
    tracer.trace_type::<QueryEvent>(&samples)?;
    tracer.trace_type::<Term>(&samples)?;
    tracer.trace_type::<Value>(&samples)?;
    tracer.trace_type::<Operator>(&samples)?;
    tracer.trace_type::<Pattern>(&samples)?;
    tracer.trace_type::<Node>(&samples)?;
    tracer.trace_type::<Trace>(&samples)?;
    // need to provide concrete values for numeric
    tracer.trace_value(&mut samples, &Numeric::from(0i64))?;
    tracer.trace_value(&mut samples, &Numeric::from(0.0f64))?;
    tracer.trace_type::<Numeric>(&samples)?;
    let registry = tracer.registry()?;

    // Create Python class definitions.
    let mut source = Vec::new();
    let config = serde_generate::CodeGeneratorConfig::new("testing".to_string());
    let generator = serde_generate::python3::CodeGenerator::new(&config);
    generator.output(&mut source, &registry)?;

    let mut f = File::create("polar_types.py")?;
    f.write_all(&source);

    // Create Go class definitions.
    let mut source = Vec::new();
    let config = serde_generate::CodeGeneratorConfig::new("testing".to_string());
    let generator = serde_generate::golang::CodeGenerator::new(&config);
    generator.output(&mut source, &registry)?;

    let mut f = File::create("polar_types.go")?;
    f.write_all(&source);
    Ok(())
}
