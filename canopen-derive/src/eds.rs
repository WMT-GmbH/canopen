use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;

use crate::Object;

// expects objects to be sorted by index and subindex
pub fn write_eds(path: &Path, objects: &[Object]) -> io::Result<()> {
    let mut file = File::create(path).expect("Failed to create file");

    write_preamble(&mut file)?;

    let mut object_iter = objects.iter().peekable();
    let mut is_compound = false;

    while let Some(object) = object_iter.next() {
        match object_iter.peek() {
            Some(next_object) if object.index == next_object.index => {
                if !is_compound {
                    write_compound_object_top_level(&mut file, object)?;
                }
                is_compound = true;
            }
            _ => {
                is_compound = false;
            }
        }

        write_object(&mut file, object, is_compound)?;
    }
    Ok(())
}

fn write_preamble(file: &mut File) -> io::Result<()> {
    writeln!(file, "[DeviceInfo]")?;
    Ok(())
}

fn write_compound_object_top_level(file: &mut File, object: &Object) -> io::Result<()> {
    // TODO don't hardcode ObjectType and ParameterName
    writeln!(file)?;
    writeln!(file, "[{:X}]", object.index)?;
    writeln!(file, "ParameterName={}", object.name())?;
    writeln!(file, "ObjectType=0x09")?;
    Ok(())
}

fn write_object(file: &mut File, object: &Object, is_compound: bool) -> io::Result<()> {
    writeln!(file)?;
    if object.subindex == 0 && !is_compound {
        writeln!(file, "[{:X}]", object.index)?;
    } else {
        writeln!(file, "[{:X}sub{:X}]", object.index, object.subindex)?;
    }
    writeln!(file, "ParameterName={}", object.name())?;
    if let Some(typ) = object.typ {
        writeln!(file, "DataType=0x{:04X}", typ as u8)?;
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
    Ok(())
}
