use hyperlight_host::GuestBinary;
use hyperlight_host::sandbox::SandboxConfiguration;
#[cfg(feature = "gdb")]
use hyperlight_host::sandbox::config::DebugInfo;

const GUEST_PATH: &str = "./guest/target/x86_64-unknown-none/release/guest";

const JS_SCRIPT: &str = r#"
function fibonacci(n) {
    if (n < 2) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

fibonacci(30);
"#;

fn main() -> anyhow::Result<()> {
    let guest = GuestBinary::FilePath(GUEST_PATH.to_string());

    let mut config = SandboxConfiguration::default();
    config.set_heap_size(32 * 1024 * 1024); // 32 MiB
    config.set_stack_size(1024 * 1024); // 128 KiB

    #[cfg(feature = "gdb")]
    {
        let debug_info = DebugInfo { port: 8080 };
        config.set_guest_debug_info(debug_info);
    }

    // create the sandbox
    let sbox = hyperlight_host::UninitializedSandbox::new(guest, Some(config))?;

    // initialize the sandbox
    let mut sbox = sbox.evolve()?;

    // call a guest function
    let _ = sbox.call_guest_function_by_name::<i32>("Init", JS_SCRIPT.to_string())?;

    let now = std::time::Instant::now();
    let n: i32 = sbox.call_guest_function_by_name::<i32>("Exec", ())?;

    let elapsed = now.elapsed();
    println!("fib(30) = {n}");
    println!("Execution time: {:.2?}", elapsed);

    Ok(())
}
