use serde_reflection::{Samples, Tracer, TracerConfig};

use std::fs::File;
use std::io::Write;

use polar_core::error::*;
use polar_core::events::QueryEvent;
use polar_core::messages::{Message, MessageKind};
use polar_core::resource_block::Declaration;
use polar_core::sources::Source;
use polar_core::terms::*;
use polar_core::traces::{Node, Trace};

mod codegen;
mod go;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logging::log_to_stderr(log::LevelFilter::Info);
    // Obtain the Serde formats of enum types.
    // The main one we can about is `QueryEvent`.
    // All others are nested inside `QueryEvent`.
    let mut tracer = Tracer::new(TracerConfig::default());
    let mut samples = Samples::new();
    tracer.trace_type::<QueryEvent>(&samples)?;
    tracer.trace_type::<Term>(&samples)?;
    tracer.trace_type::<Value>(&samples)?;
    tracer.trace_type::<Operator>(&samples)?;
    tracer.trace_type::<Pattern>(&samples)?;
    tracer.trace_type::<Node>(&samples)?;
    tracer.trace_type::<Trace>(&samples)?;
    tracer.trace_type::<ParseError>(&samples)?;
    tracer.trace_type::<OperationalError>(&samples)?;
    tracer.trace_type::<RuntimeError>(&samples)?;
    tracer.trace_type::<ValidationError>(&samples)?;
    tracer.trace_type::<FormattedPolarError>(&samples)?;
    tracer.trace_type::<ErrorKind>(&samples)?;
    tracer.trace_type::<MessageKind>(&samples)?;
    tracer.trace_type::<Message>(&samples)?;
    tracer.trace_type::<Source>(&samples)?;
    tracer.trace_type::<Declaration>(&samples)?;

    // need to provide concrete values for numeric
    tracer.trace_value(&mut samples, &Numeric::from(0i64))?;
    tracer.trace_value(&mut samples, &Numeric::from(0.0f64))?;
    // TODO: tracing these results in an error.
    // serde reflection doesn't support untagged enums
    // This means we don't generate the correct type for floats
    // tracer.trace_value(&mut samples, &Numeric::from(std::f64::NAN))?;
    // tracer.trace_value(&mut samples, &Numeric::from(std::f64::INFINITY))?;
    tracer.trace_type::<Numeric>(&samples)?;
    let registry = tracer.registry()?;

    // Uncomment for Go branch
    let mut f = File::create("../languages/go/types/polar_types.go")?;
    let source = codegen::Codegen::new("go", &go::Go)?.output(&registry)?;
    f.write_all(source.as_bytes())?;

    Ok(())
}
