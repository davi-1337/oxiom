use arbitrary::{Arbitrary, Unstructured};
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};

fn main() {
    let seed: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);

    let mut rng = StdRng::seed_from_u64(seed);
    let mut buf = vec![0u8; 8192];
    rng.fill_bytes(&mut buf);
    let mut u = Unstructured::new(&buf);
    match oxiom_generator::FuzzProgram::arbitrary(&mut u) {
        Ok(program) => {
            let html = oxiom_serializer::serialize(
                &program.font_faces,
                &program.css_rules,
                &program.dom,
                &program.script,
                &program.keyframes,
                &program.at_rules,
            );
            println!("{}", html);
        }
        Err(e) => eprintln!("Error: {:?}", e),
    }
}
