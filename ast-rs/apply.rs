use std::{
    fs,
    io::{self, BufRead, BufReader},
    path::Path,
};

const DEFINITION_MARKERS: (&str, &str) = (
    "// DESERIALIZATION DEFINITION START",
    "// DESERIALIZATION DEFINITION END",
);
const IMPLEMENTATION_MARKERS: (&str, &str) = (
    "// DESERIALIZATION IMPLEMENTATION START",
    "// DESERIALIZATION IMPLEMENTATION END",
);

pub fn definition(filepath: impl AsRef<Path>, replacement: &str) -> io::Result<()> {
    replace_in_file(filepath, DEFINITION_MARKERS, replacement)
}

pub fn implementation(filepath: impl AsRef<Path>, replacement: &str) -> io::Result<()> {
    replace_in_file(filepath, IMPLEMENTATION_MARKERS, replacement)
}

fn replace_in_file(
    filepath: impl AsRef<Path>,
    markers: (&str, &str),
    replacement: &str,
) -> io::Result<()> {
    #[derive(PartialEq, Debug)]
    enum State {
        Before,
        Inbetween,
        After,
    }

    let file = fs::File::open(&filepath)?;
    let r = BufReader::new(file);
    let mut buf = Vec::with_capacity(4096);
    let mut state = State::Before;
    for line in r.lines().map_while(Result::ok) {
        match state {
            State::Before => {
                if line.trim_end() == markers.0 {
                    state = State::Inbetween;
                }
            }
            State::Inbetween => {
                if line.trim_end() != markers.1 {
                    continue;
                }
                buf.extend_from_slice(replacement.as_bytes());
                buf.push(b'\n');
                state = State::After;
            }
            State::After => (),
        }
        buf.extend_from_slice(line.as_bytes());
        buf.push(b'\n');
    }
    assert_eq!(state, State::After);

    fs::write(&filepath, buf)
}
