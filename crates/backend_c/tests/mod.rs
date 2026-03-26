mod reference_project {
    #[test]
    fn interop() -> Result<(), Box<dyn std::error::Error>> {
        interoptopus_c::generate("hello_world_c", &hello_world_c::inventory(), "tests/reference_project/hello_world.h")?;
        Ok(())
    }
}
