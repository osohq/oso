use serde_reflection::{Samples, Tracer, TracerConfig};

use std::fs::File;
use std::io::Write;

use polar_core::error::{
    ErrorKind, FormattedPolarError, OperationalError, ParseError, RuntimeError,
};
use polar_core::events::QueryEvent;
use polar_core::messages::{Message, MessageKind};
use polar_core::terms::*;
use polar_core::traces::{Node, Trace};

mod go;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logging::log_to_stderr(log::LevelFilter::Info);
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
    tracer.trace_type::<FormattedPolarError>(&samples)?;
    tracer.trace_type::<ErrorKind>(&samples)?;
    tracer.trace_type::<ParseError>(&samples)?;
    tracer.trace_type::<OperationalError>(&samples)?;
    tracer.trace_type::<RuntimeError>(&samples)?;
    tracer.trace_type::<MessageKind>(&samples)?;
    tracer.trace_type::<Message>(&samples)?;

    // need to provide concrete values for numeric
    tracer.trace_value(&mut samples, &Numeric::from(0i64))?;
    tracer.trace_value(&mut samples, &Numeric::from(0.0f64))?;
    // TODO: tracing these results in an error.
    // serde reflection doesn't support untagged enums
    // tracer.trace_value(&mut samples, &Numeric::from(std::f64::NAN))?;
    // tracer.trace_value(&mut samples, &Numeric::from(std::f64::INFINITY))?;
    tracer.trace_type::<Numeric>(&samples)?;
    let registry = tracer.registry()?;

    // Create Python class definitions.
    let mut source = Vec::new();
    let config =
        serde_generate::CodeGeneratorConfig::new("oso".to_string()).with_serialization(false);
    let generator = serde_generate::python3::CodeGenerator::new(&config);
    generator.output(&mut source, &registry)?;

    let mut f = File::create("polar_types.py")?;
    f.write_all(&source)?;

    let mut f = File::create("../languages/go/pkg/polar_types.go")?;
    let source = go::Codegen::output(&registry)?;
    f.write_all(&source.as_bytes())?;
    Ok(())
}
