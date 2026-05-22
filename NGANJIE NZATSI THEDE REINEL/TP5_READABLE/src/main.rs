use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

/// TP6: Generic I/O abstraction over multiple input sources.
trait Readable {
    fn source_name(&self) -> &str;
    fn read_to_string(&mut self) -> io::Result<String>;
}

struct FileReadable {
    path: PathBuf,
    label: String,
}

impl FileReadable {
    fn new(path: impl Into<PathBuf>) -> Self {
        let path_buf = path.into();
        let label = format!("file:{}", path_buf.display());
        Self {
            path: path_buf,
            label,
        }
    }
}

impl Readable for FileReadable {
    fn source_name(&self) -> &str {
        &self.label
    }

    fn read_to_string(&mut self) -> io::Result<String> {
        fs::read_to_string(&self.path)
    }
}

struct MemoryReadable {
    label: String,
    content: String,
}

impl MemoryReadable {
    fn new(label: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            content: content.into(),
        }
    }
}

impl Readable for MemoryReadable {
    fn source_name(&self) -> &str {
        &self.label
    }

    fn read_to_string(&mut self) -> io::Result<String> {
        Ok(self.content.clone())
    }
}

struct StdinReadable {
    label: String,
}

impl StdinReadable {
    fn new() -> Self {
        Self {
            label: "stdin".to_string(),
        }
    }
}

impl Readable for StdinReadable {
    fn source_name(&self) -> &str {
        &self.label
    }

    fn read_to_string(&mut self) -> io::Result<String> {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        Ok(input)
    }
}

/// Generic static dispatch (`R` known at compile time).
fn read_static<R: Readable>(source: &mut R) -> io::Result<String> {
    source.read_to_string()
}

/// Dynamic dispatch via trait objects.
fn read_dynamic(sources: &mut [Box<dyn Readable>]) -> Vec<(String, io::Result<String>)> {
    let mut out = Vec::with_capacity(sources.len());
    for source in sources {
        let name = source.source_name().to_string();
        let content = source.read_to_string();
        out.push((name, content));
    }
    out
}

fn print_summary(source: &str, content: &str) {
    let bytes = content.len();
    let lines = content.lines().count();
    println!("[{source}] {bytes} bytes, {lines} lines");
}

fn run() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let include_stdin = args.iter().any(|arg| arg == "--stdin");
    let maybe_path = args.iter().find(|arg| !arg.starts_with("--"));

    println!("TP6 - Abstraction I/O generique");

    // 1) Generic static call.
    let mut memory = MemoryReadable::new("memory:lesson", "hello from memory\nline2");
    let static_content = read_static(&mut memory)?;
    print_summary(memory.source_name(), &static_content);

    // 2) Dynamic collection with trait objects.
    let mut sources: Vec<Box<dyn Readable>> = Vec::new();
    sources.push(Box::new(MemoryReadable::new(
        "memory:dynamic",
        "dynamic buffer\nsecond line",
    )));

    if let Some(path) = maybe_path {
        sources.push(Box::new(FileReadable::new(path)));
    } else {
        println!("No file path argument provided, skipping file source.");
    }

    if include_stdin {
        println!("Reading from stdin (Ctrl+Z then Enter to finish on Windows)...");
        sources.push(Box::new(StdinReadable::new()));
    }

    let results = read_dynamic(&mut sources);
    for (source, result) in results {
        match result {
            Ok(content) => print_summary(&source, &content),
            Err(err) => eprintln!("[{source}] error: {err}"),
        }
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("fatal error: {err}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{FileReadable, MemoryReadable, read_dynamic, read_static};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_file() -> PathBuf {
        let base = std::env::temp_dir();
        let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => d.as_nanos(),
            Err(_) => 0,
        };
        base.join(format!("tp6_readable_{}_{}.txt", std::process::id(), nanos))
    }

    #[test]
    fn static_generic_reads_memory() {
        let mut src = MemoryReadable::new("mem", "abc\ndef");
        let res = read_static(&mut src);

        match res {
            Ok(content) => assert_eq!(content, "abc\ndef"),
            Err(err) => panic!("expected Ok, got error: {err}"),
        }
    }

    #[test]
    fn dynamic_collection_reads_multiple_sources() {
        let file_path = unique_temp_file();
        let write_res = fs::write(&file_path, "from file");
        if let Err(err) = write_res {
            panic!("failed to create test file: {err}");
        }

        let mut sources: Vec<Box<dyn super::Readable>> = Vec::new();
        sources.push(Box::new(MemoryReadable::new("m", "from mem")));
        sources.push(Box::new(FileReadable::new(&file_path)));

        let results = read_dynamic(&mut sources);
        assert_eq!(results.len(), 2);

        match &results[0].1 {
            Ok(content) => assert_eq!(content, "from mem"),
            Err(err) => panic!("memory source should succeed: {err}"),
        }
        match &results[1].1 {
            Ok(content) => assert_eq!(content, "from file"),
            Err(err) => panic!("file source should succeed: {err}"),
        }

        let rm_res = fs::remove_file(file_path);
        if let Err(err) = rm_res {
            panic!("failed to remove temp file: {err}");
        }
    }
}
