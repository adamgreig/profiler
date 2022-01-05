use std::time::Duration;
use std::process::Command;
use std::collections::HashMap;
use std::env;
use std::thread;
use probe_rs::Probe;

fn main() -> anyhow::Result<()> {
    let probes = Probe::list_all();
    if probes.is_empty() {
        anyhow::bail!("No probes detected");
    } else if probes.len() > 1 {
        anyhow::bail!("Multiple probes detected");
    }
    let elf = env::args().nth(1).unwrap();
    let probe = probes[0].open()?;
    let mut session = probe.attach("armv6m")?;
    let mut core = session.core(0)?;
    let mut samples = Vec::with_capacity(1000);
    for _ in 0..samples.capacity() {
        let info = core.halt(Duration::from_millis(10))?;
        core.run()?;
        samples.push(info.pc);
        thread::sleep(Duration::from_millis(10));
    }
    let mut functions = Vec::new();
    for sample in samples {
        let output = Command::new("arm-none-eabi-addr2line")
            .arg("-f")
            .arg("-C")
            .arg("-e")
            .arg(&elf)
            .arg(format!("{:08X}", sample))
            .output()?;
        let output = String::from_utf8_lossy(&output.stdout);
        let function = output.lines().nth(0).unwrap();
        functions.push(function.to_string());
    }

    let mut counts: HashMap<String, usize> = HashMap::new();
    for f in functions {
        *counts.entry(f).or_default() += 1;
    }
    let mut counts: Vec<(&String, &usize)> = counts.iter().collect();
    counts.sort_by(|a, b| b.1.cmp(a.1));
    for (f, c) in counts.iter() {
        println!("{:5} {}", c, f);
    }
    Ok(())
}
