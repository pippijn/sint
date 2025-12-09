use crate::beam_search::SearchStatus;
use cursive::view::{Nameable, Resizable};
use cursive::views::{Dialog, LinearLayout, Panel, ScrollView, TextView};
use cursive::{Cursive, CursiveExt};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn run(status: Arc<Mutex<SearchStatus>>) {
    let mut siv = Cursive::default();

    // UI Layout
    let stats_view = TextView::new("Initializing...").with_name("stats");
    let path_view = TextView::new("No path found yet.").with_name("path");

    let layout = LinearLayout::horizontal()
        .child(
            Panel::new(stats_view)
                .title("Search Status")
                .fixed_width(40),
        )
        .child(
            Panel::new(ScrollView::new(path_view))
                .title("Best Path So Far")
                .full_width(),
        );

    siv.add_layer(Dialog::around(layout).title("SINT FTL Solver"));

    // Background Update Thread
    let cb_sink = siv.cb_sink().clone();
    let status_clone = status.clone();

    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(500));

            let s = status_clone.lock().unwrap();
            let depth = s.depth;
            let score = s.best_score;
            let nodes = s.nodes_visited;
            let finished = s.finished;
            let frontier = s.current_frontier_size;

            let mut path_text = String::new();
            if let Some(node) = &s.best_node {
                path_text.push_str(&format!("Phase: {:?}\n", node.state.phase));
                path_text.push_str(&format!("Hull: {}\n", node.state.hull_integrity));
                path_text.push_str(&format!(
                    "Boss: {} (Lvl {})\n",
                    node.state.enemy.hp, node.state.boss_level
                ));
                path_text.push_str("\n--- Action History ---\n");

                for (pid, action) in &node.path {
                    path_text.push_str(&format!("{}: {:?}\n", pid, action));
                }
            }

            let stats_text = format!(
                "Depth: {}\nScore: {}\nNodes: {}\nFrontier: {}\nStatus: {}",
                depth,
                score,
                nodes,
                frontier,
                if finished { "FINISHED" } else { "RUNNING" }
            );

            // Send callback to Main Thread
            cb_sink
                .send(Box::new(move |s: &mut Cursive| {
                    s.call_on_name("stats", |v: &mut TextView| {
                        v.set_content(stats_text);
                    });
                    s.call_on_name("path", |v: &mut TextView| {
                        v.set_content(path_text);
                    });
                }))
                .unwrap();

            if finished {
                // Keep running to show result, but maybe slow down updates
            }
        }
    });

    siv.run();
}
