use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;

use crate::Object;

// expects objects to be sorted by index and subindex
pub fn write_eds(path: &Path, objects: &[Object]) -> io::Result<()> {
    let mut file = File::create(path).expect("Failed to create file");

    for object in objects {
        writeln!(file)?;
        if object.subindex == 0 {
            writeln!(file, "[{:04x}]", object.index)?; // TODO sub0 is also valid
        } else {
            writeln!(file, "[{:04x}sub{:02x}]", object.index, object.subindex)?;
        }
        if let Some(name) = &object.name {
            writeln!(file, "ParameterName={}", name)?;
        } else {
            writeln!(file, "ParameterName={}", object.ident.to_string())?;
        }
        if let Some(typ) = object.typ {
            writeln!(file, "DataType=0x{:02x}", typ as u8)?;
        }
        writeln!(
            file,
            "AccessType={}",
            if object.read_only {
                "ro"
            } else if object.write_only {
                "wo"
            } else {
                "rw"
            }
        )?; // TODO const
    }
    Ok(())
}
