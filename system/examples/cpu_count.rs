// unix 获取 CPU 数量
// https://github.com/rust-lang/rust/pull/92697
fn main() {
    let stdout = std::process::Command::new("nproc").output().unwrap().stdout;
    let nproc = unsafe { String::from_utf8_unchecked(stdout) }
        .trim_end()
        .parse::<u8>()
        .unwrap();

    let mut cpuset = unsafe { std::mem::zeroed() };
    unsafe {
        libc::sched_getaffinity(1, std::mem::size_of::<libc::cpu_set_t>(), &mut cpuset);
        let num_cpu = libc::CPU_COUNT(&cpuset) as u8;
        assert_eq!(nproc, num_cpu);
        dbg!(num_cpu);
    }
}
