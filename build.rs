use cambria::{Kind, Lens, Lenses, PrimitiveKind};
use std::process::Command;

fn write_rust(path: &str, rust: String) {
    std::fs::write(path, rust).unwrap();
    Command::new("rustfmt")
        .arg(path)
        .arg("--emit")
        .arg("files")
        .status()
        .unwrap();
}

fn main() {
    let file_lenses = vec![
        Lens::Make(Kind::Object),
        Lens::AddProperty("blake3_hash".into()),
        Lens::LensIn(
            "blake3_hash".into(),
            Box::new(Lens::Make(Kind::Primitive(PrimitiveKind::Text))),
        ),
        Lens::AddProperty("bao_hash".into()),
        Lens::LensIn(
            "bao_hash".into(),
            Box::new(Lens::Make(Kind::Primitive(PrimitiveKind::Text))),
        ),
        Lens::AddProperty("bytes_read".into()),
        Lens::LensIn(
            "bytes_read".into(),
            Box::new(Lens::Make(Kind::Primitive(PrimitiveKind::Number))),
        ),
        Lens::AddProperty("bytes_written".into()),
        Lens::LensIn(
            "bytes_written".into(),
            Box::new(Lens::Make(Kind::Primitive(PrimitiveKind::Number))),
        ),
        Lens::AddProperty("min_slice".into()),
        Lens::LensIn(
            "min_slice".into(),
            Box::new(Lens::Make(Kind::Primitive(PrimitiveKind::Number))),
        ),
        Lens::AddProperty("max_slice".into()),
        Lens::LensIn(
            "max_slice".into(),
            Box::new(Lens::Make(Kind::Primitive(PrimitiveKind::Number))),
        ),
        Lens::AddProperty("path".into()),
        Lens::LensIn(
            "path".into(),
            Box::new(Lens::Make(Kind::Primitive(PrimitiveKind::Text))),
        ),
        Lens::AddProperty("parent_rev".into()),
        Lens::LensIn(
            "parent_rev".into(),
            Box::new(Lens::Make(Kind::Primitive(PrimitiveKind::Text))),
        ),
        Lens::AddProperty("mime_type".into()),
        Lens::LensIn(
            "mime_type".into(),
            Box::new(Lens::Make(Kind::Primitive(PrimitiveKind::Text))),
        ),
        Lens::AddProperty("date_created".into()),
        Lens::LensIn(
            "date_created".into(),
            Box::new(Lens::Make(Kind::Primitive(PrimitiveKind::Number))),
        ),
        Lens::AddProperty("date_modified".into()),
        Lens::LensIn(
            "date_modified".into(),
            Box::new(Lens::Make(Kind::Primitive(PrimitiveKind::Number))),
        ),
        Lens::AddProperty("date_accessed".into()),
        Lens::LensIn(
            "date_accessed".into(),
            Box::new(Lens::Make(Kind::Primitive(PrimitiveKind::Number))),
        ),
        Lens::AddProperty("dropped".into()),
        Lens::LensIn(
            "dropped".into(),
            Box::new(Lens::Make(Kind::Primitive(PrimitiveKind::Boolean))),
        ),
        Lens::AddProperty("removed".into()),
        Lens::LensIn(
            "removed".into(),
            Box::new(Lens::Make(Kind::Primitive(PrimitiveKind::Boolean))),
        ),
    ];

    let file = cambria::precompile("File", Lenses::new(file_lenses));
    write_rust("src/schema/file.rs", file.to_string());
}
