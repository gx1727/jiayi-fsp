//! 场景演示 —— 跑一遍规范场景并打印观测结果。
//!
//! 用法：
//!
//! ```text
//! cargo run -p fsp-sim            # 跑全部场景
//! cargo run -p fsp-sim 空洞       # 只跑名字含「空洞」的场景
//! ```

use fsp_sim::scenarios;

fn main() {
    let filter = std::env::args().nth(1);
    let mut failed = 0usize;

    for scenario in scenarios::all() {
        if let Some(f) = &filter {
            if !scenario.name.contains(f.as_str()) {
                continue;
            }
        }
        println!("──────────────────────────────────────────────────────────────");
        println!("场景：{}", scenario.name);
        println!("规范：{} —— {}", scenario.spec, scenario.description);

        let mut world = (scenario.setup)();
        world.client.start(&mut world.sim);
        world.run(scenario.duration_ms);

        let c = &world.client;
        let s = &world.server;
        println!("  仿真时刻          {} ms", world.sim.now_ms());
        println!("  发送：数据报 {} 个，完成 = {}", c.datagrams_sent, c.completed);
        println!(
            "  发送：收到裁决 {} 次（其中空数组 {} 次），索要 {} 次，暂停 {} 次",
            c.verdicts_seen, c.empty_verdicts_seen, c.requests_sent, c.pause_count
        );
        println!(
            "  发送：补发的块 {:?}",
            c.send_counts
                .iter()
                .enumerate()
                .filter(|(_, n)| **n > 1)
                .map(|(i, n)| (i, *n))
                .collect::<Vec<_>>()
        );
        println!(
            "  接收：裁决 {} 次（其中空数组 {} 次），收到索要 {} 次",
            s.verdicts_sent, s.empty_verdicts_sent, s.requests_seen
        );
        println!(
            "  接收：丢弃损坏包 {} 个，幂等丢弃 {} 个",
            s.corrupt_dropped, s.duplicate_dropped
        );
        println!(
            "  数据面：投递 {} 个，丢弃 {} 个",
            world.sim.datagrams_delivered, world.sim.datagrams_dropped
        );
        println!(
            "  控制面：投递 {} 条，丢弃 {} 条",
            world.sim.control_delivered, world.sim.control_dropped
        );
        println!("  续传应答：{:?}", c.resume_acks);
        println!("  文件一致：{}", world.content_matches());

        let ok = (scenario.pass)(&world);
        if !ok {
            failed += 1;
        }
        println!("  结果：{}", if ok { "✅ 通过" } else { "❌ 未通过" });
    }

    println!("──────────────────────────────────────────────────────────────");
    if failed == 0 {
        println!("全部场景通过");
    } else {
        println!("{failed} 个场景未通过");
        std::process::exit(1);
    }
}
