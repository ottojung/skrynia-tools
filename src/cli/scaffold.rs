use super::args::Arguments;
use anyhow::{Result, bail};
use std::{fs, path::Path};

pub fn create(args: &Arguments) -> Result<()> {
    let name = args.required("name")?;
    if !super::deploy::valid_name(name) {
        bail!("invalid app name");
    }
    let root = Path::new(args.required("directory")?);
    fs::create_dir(root)?;
    fs::create_dir(root.join("public"))?;
    fs::write(
        root.join("Makefile"),
        "build:\n\tmkdir -p build\n\tcp -R public/. build/\n",
    )?;
    fs::write(
        root.join("public/index.html"),
        format!(
            "<!doctype html>\n<html lang=\"en\"><meta charset=\"utf-8\"><title>{name}</title>\n<h1>{name}</h1></html>\n"
        ),
    )?;
    fs::write(
        root.join("README.md"),
        "Static Skrynia application. Run make build in this directory and deploy build/.\nNo storage grant is needed unless the app chooses to store data.\n",
    )?;
    println!("{}", root.display());
    Ok(())
}
