use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wincode::SchemaRead;

// --- unsafe approach (read_unaligned) ---

#[repr(C)]
#[derive(Clone, Copy)]
struct MakeIxUnsafe {
    amount_to_receive: u64,
    amount_to_give: u64,
    bump: u8,
    _padding: [u8; 7],
}

#[inline(always)]
fn load_unsafe(data: &[u8]) -> MakeIxUnsafe {
    unsafe { core::ptr::read_unaligned(data.as_ptr() as *const MakeIxUnsafe) }
}

// --- wincode approach (SchemaRead derive) ---

#[derive(SchemaRead)]
struct MakeIxWincode {
    amount_to_receive: u64,
    amount_to_give: u64,
    bump: u8,
    _padding: [u8; 7],
}

#[inline(always)]
fn load_wincode(data: &[u8]) -> MakeIxWincode {
    wincode::deserialize(data).unwrap()
}

// --- benchmark ---

fn bench_make_ix(c: &mut Criterion) {
    let amount_to_receive: u64 = 100_000_000;
    let amount_to_give: u64 = 500_000_000;
    let bump: u8 = 254;

    let data: Vec<u8> = [
        amount_to_receive.to_le_bytes().as_slice(),
        amount_to_give.to_le_bytes().as_slice(),
        &[bump],
        &[0u8; 7],
    ]
    .concat();

    c.bench_function("unsafe read_unaligned", |b| {
        b.iter(|| {
            let x = load_unsafe(black_box(&data));
            black_box((x.amount_to_receive, x.amount_to_give, x.bump))
        })
    });

    c.bench_function("wincode deserialize", |b| {
        b.iter(|| {
            let x = load_wincode(black_box(&data));
            black_box((x.amount_to_receive, x.amount_to_give, x.bump))
        })
    });
}

criterion_group!(benches, bench_make_ix);
criterion_main!(benches);
