use rayon::yield_now;
use rayon::ThreadPoolBuilder;
#[cfg(target_feature = "avx2")]
use rust_enjoyer::sponges_avx::SpongesAvx;

use rust_enjoyer::*;

use rayon::prelude::*;
use std::env;
use std::process;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

fn main() {
    #[cfg(target_feature = "avx2")]
    println!("AVX2 enabled");
    #[cfg(not(target_feature = "avx2"))]
    println!("AVX2 disabled");

    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        println!("Usage: <function name> <function params> <target selector>");
        process::exit(-1);
    }

    // remove any leading 0x
    let selector = args[3].to_lowercase();
    let selector = selector.trim_start_matches("0x");
    let selector = u32::from_str_radix(selector, 16)
        .expect("Invalid number")
        .to_be();
    let function_name = SmallString::new(&args[1]);
    let function_params = SmallString::new(&args[2]);

    if function_name.length + function_params.length >= 115 {
        println!("Total length of <function name> and <function params> must be under 115 bytes.");
        process::exit(-1);
    }

    if std::mem::size_of::<u64>() != 8 {
        println!("Incompatible architecture");
        println!("u64: {}", std::mem::size_of::<u64>());
        process::exit(-1);
    }

    println!("Function name: {}", args[1]);
    println!("Function params: {}", args[2]);
    println!("Target selector: {selector:#10x?}",);

    let num_threads = args
        .get(4)
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or_else(|| num_cpus::get());

    let end = 0xfffffffff0000000usize;
    let go = AtomicBool::new(true);

    #[cfg(target_feature = "avx2")]
    const STEP: usize = 4;
    #[cfg(not(target_feature = "avx2"))]
    const STEP: usize = 1;

    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .expect("Failed to create thread pool");

    println!("Starting mining with {num_threads} threads.");

    // pool.install(|| {
    (0..num_threads).into_par_iter().for_each(|thread_idx| {
        for (idx, nonce) in (thread_idx * STEP..end)
            .step_by(num_threads * STEP)
            .enumerate()
        {
            if !go.load(Ordering::Relaxed) {
                break;
            }

            #[cfg(not(target_feature = "avx2"))]
            {
                let mut s0 = Sponge::default();
                unsafe { s0.fill(&function_name, nonce as u64, &function_params) };

                if selector == unsafe { s0.compute_selectors() } {
                    let out = unsafe {
                        s0.fill_and_get_name(&function_name, nonce as u64, &function_params)
                    };
                    println!("Function found: {out}");

                    go.store(false, Ordering::Relaxed);
                }
            }
            #[cfg(target_feature = "avx2")]
            {
                let mut sponges = SpongesAvx::new(&function_name, nonce as u64, &function_params);

                let maybe_idx = sponges
                    .compute_selectors()
                    .iter()
                    .position(|&x| x == selector);

                // if nonce == 0xc4de884 {
                //     println!("thread {thread_idx} should have stopped!!!!");
                // }

                // Progress logging for thread 0
                if idx & 0x1fffff == 0 {
                    if thread_idx == 0 {
                        println!("{num_hashes:?} hashes done.", num_hashes = nonce);
                    }

                    if thread_idx == 33 {
                        println!(
                            "thread {thread_idx} {nonce:#10x?} {over}",
                            nonce = nonce,
                            thread_idx = thread_idx,
                            over = nonce > 0xc4de884
                        );
                    }
                }

                // if idx & 0xFF == 0 {}
                // yield_now();

                // if nonce > 0xc4de884 {
                //     println!("thread {thread_idx} stopping, went over {nonce:#10x?}");
                //     break;
                // }

                let Some(found_idx) = maybe_idx else {
                    continue;
                };

                // we found a match
                let out = Sponge::default().fill_and_get_name(
                    &function_name,
                    (nonce + found_idx) as u64,
                    &function_params,
                );
                println!("thread {thread_idx} Function found: {out}");

                go.store(false, Ordering::SeqCst);
            }
        }
    })
    // });
}
