use image::{
    Pixel,
    EncodableLayout,
    ImageBuffer,
    GenericImageView,
    imageops::{resize, FilterType},
    io::Reader as ImageLoader,
};

use std::{io, ops::Deref};

#[derive(Copy, Clone, Debug)]
enum Operation {
    Shrink,
    Enlarge,
}

#[derive(Clone, Debug)]
struct Opt {
    images: Vec<String>,
    operation: Operation,
    threshold: u32,
}

impl Opt {
    fn from_args() -> Opt {
        use clap::{crate_authors, crate_description, crate_version, value_t, App, Arg, ArgGroup};

        let m = App::new("resize")
            .version(crate_version!())
            .author(crate_authors!())
            .about(crate_description!())
            .arg(Arg::with_name("image").takes_value(true).multiple(true))
            .arg(Arg::with_name("up").short("u").long("up"))
            .arg(Arg::with_name("down").short("d").long("down"))
            .arg(
                Arg::with_name("threshold")
                    .short("s")
                    .long("size")
                    .required(true)
                    .takes_value(true),
            )
            .group(ArgGroup::with_name("operation").arg("up").arg("down"))
            .get_matches();

        Opt {
            threshold: value_t!(m.value_of("threshold"), u32).unwrap_or_else(|e| e.exit()),
            images: m
                .values_of("image")
                .into_iter()
                .flatten()
                .map(|x| x.to_string())
                .collect(),
            operation: if m.is_present("up") {
                Operation::Enlarge
            } else {
                Operation::Shrink
            },
        }
    }
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();

    for image in opt.images {
        match opt.operation {
            Operation::Enlarge => enlarge(&image, opt.threshold)?.write()?,
            Operation::Shrink => shrink(&image, opt.threshold)?.write()?,
        }
    }

    Ok(())
}

/// A writable image buffer.
trait Writable {
    fn write(&self, path: &str) -> io::Result<()>;
}

impl<P, Container> Writable for ImageBuffer<P, Container>
where
    P: Pixel + 'static,
    P::Subpixel: 'static,
    [P::Subpixel]: EncodableLayout,
    Container: Deref<Target = [P::Subpixel]>,
{
    fn write(&self, path: &str) -> io::Result<()> {
        self.save(path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}

enum Resize<'a> {
    Resize {
        path: &'a str,
        buffer: Box<dyn Writable>,
    },
    Noop,
}

impl Resize<'_> {
    fn write(&self) -> io::Result<()> {
        match self {
            Resize::Resize { path, buffer } => buffer.write(path),
            Resize::Noop => Ok(()),
        }
    }
}

fn enlarge(_image: &str, _threshold: u32) -> io::Result<Resize> {
    panic!("No idea who in their right mind would implement this, or how!")
}

fn shrink(image: &str, threshold: u32) -> io::Result<Resize> {
    let buffer = ImageLoader::open(image)?.decode().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let (width, height) = buffer.dimensions();

    if width > threshold {
        let width = threshold;
        let height = (threshold as f64 / width as f64).floor() as u32;
        Ok(Resize::Resize {
            path: image,
            buffer: Box::new(resize(&buffer, width, height, FilterType::Lanczos3)),
        })
    } else if height > threshold {
        let height = threshold;
        let width = (threshold as f64 / height as f64).floor() as u32;
        Ok(Resize::Resize {
            path: image,
            buffer: Box::new(resize(&buffer, width, height, FilterType::Lanczos3)),
        })
    } else {
        Ok(Resize::Noop)
    }
}
