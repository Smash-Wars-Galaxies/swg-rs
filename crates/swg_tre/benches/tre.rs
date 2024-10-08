use divan::AllocProfiler;

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    divan::main();
}

pub mod read {
    use divan::Bencher;
    use std::io::{prelude::*, Cursor};
    use swg_tre::TreArchive;

    fn get_input() -> Vec<u8> {
        std::fs::read(format!(
            "{}/resources/smash_02.tre",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap()
    }

    #[divan::bench]
    fn open(bencher: Bencher) {
        bencher.with_inputs(get_input).bench_refs(|data| {
            divan::black_box(TreArchive::new(Cursor::new(data)).unwrap());
        });
    }

    #[divan::bench]
    fn access_file(bencher: Bencher) {
        bencher
            .with_inputs(|| TreArchive::new(Cursor::new(get_input())).unwrap())
            .bench_refs(|tre| {
                divan::black_box(tre.by_index(0).unwrap());
            });
    }

    #[divan::bench(sample_count = 1)]
    fn read_file_first(bencher: Bencher) {
        let mut tre = TreArchive::new(Cursor::new(get_input())).unwrap();
        bencher.bench_local(move || {
            let mut buffer = Vec::new();

            let mut file = tre.by_index(0).unwrap();
            file.read_to_end(&mut buffer).unwrap();
        });
    }

    #[divan::bench(sample_count = 1)]
    fn read_file_all(bencher: Bencher) {
        let mut tre = TreArchive::new(Cursor::new(get_input())).unwrap();

        bencher.bench_local(move || {
            let mut buffer = Vec::new();
            for i in 0..tre.len() {
                let mut file = tre.by_index(i).unwrap();
                file.read_to_end(&mut buffer).unwrap();
                buffer.clear();
            }
        });
    }
}
