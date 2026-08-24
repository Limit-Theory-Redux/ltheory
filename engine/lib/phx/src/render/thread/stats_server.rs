//! HTTP dashboard for the render stats (`stats-server` feature).
//!
//! - `GET /`            — the self-contained HTML dashboard page
//! - `GET /stats.json`  — the latest [`StatsSnapshot`] as JSON
//! - `GET /profile.json`, `POST /profile/toggle` — producer profiler
//!
//! The sink holds the most recent [`StatsSnapshot`]; the renderer publishes
//! one per frame when attached (see `stats_snapshot.rs`). The snapshot's
//! `render` field is the executor's plain [`RenderStats`] verbatim.

#![cfg(feature = "stats-server")]

use std::sync::Arc;
use std::time::Duration;

use tracing::{error, info, warn};

use super::{CommandCategory, StatsSink, StatsSnapshot};
use crate::render::RenderStats;

/// Serialize the render-thread half of a snapshot to JSON.
///
/// Field set is small and stable, so this stays hand-rolled to keep the
/// feature's dependency footprint at just tiny_http.
fn render_stats_to_json(r: &RenderStats) -> String {
    let cats: Vec<String> = CommandCategory::ALL
        .iter()
        .map(|c| {
            format!(
                "\"{}\": {{\"count\": {}, \"time_us\": {}}}",
                c.name(),
                r.category_counts_last_frame[c.index()],
                r.category_time_us_last_frame[c.index()]
            )
        })
        .collect();

    format!(
        "{{\n\
         \x20 \"commands_processed\": {},\n\
         \x20 \"draw_calls\": {},\n\
         \x20 \"state_changes\": {},\n\
         \x20 \"frame_count\": {},\n\
         \x20 \"last_frame_time_us\": {},\n\
         \x20 \"commands_last_frame\": {},\n\
         \x20 \"draw_calls_last_frame\": {},\n\
         \x20 \"state_changes_last_frame\": {},\n\
         \x20 \"present_wait_us\": {},\n\
         \x20 \"texture_bind_calls_last_frame\": {},\n\
         \x20 \"texture_binds_skipped_last_frame\": {},\n\
         \x20 \"texture_cache_invalidations_last_frame\": {},\n\
         \x20 \"texture_invalidations_on_shader_bind_last_frame\": {},\n\
         \x20 \"texture_invalidations_on_shader_unbind_last_frame\": {},\n\
         \x20 \"draw_mesh_calls_last_frame\": {},\n\
         \x20 \"draw_immediate_calls_last_frame\": {},\n\
         \x20 \"draw_instanced_calls_last_frame\": {},\n\
         \x20 \"immediate_vertices_last_frame\": {},\n\
         \x20 \"instanced_data_items_last_frame\": {},\n\
         \x20 \"vertices_drawn_last_frame\": {},\n\
         \x20 \"uniform_cache_hits_last_frame\": {},\n\
         \x20 \"uniform_cache_misses_last_frame\": {},\n\
         \x20 \"texture_binds_skipped\": {},\n\
         \x20 \"recv_wait_us_last_frame\": {},\n\
         \x20 \"recv_wait_count_last_frame\": {},\n\
         \x20 \"shader_bind_commands_last_frame\": {},\n\
         \x20 \"shader_redundant_binds_last_frame\": {},\n\
         \x20 \"shader_distinct_programs_last_frame\": {},\n\
         \x20 \"categories\": {{\n{}\x20 }}\n\
         }}",
        r.commands_processed,
        r.draw_calls,
        r.state_changes,
        r.frame_count,
        r.last_frame_time_us,
        r.commands_last_frame,
        r.draw_calls_last_frame,
        r.state_changes_last_frame,
        r.present_wait_us,
        r.texture_bind_calls_last_frame,
        r.texture_binds_skipped_last_frame,
        r.texture_cache_invalidations_last_frame,
        r.texture_invalidations_on_shader_bind_last_frame,
        r.texture_invalidations_on_shader_unbind_last_frame,
        r.draw_mesh_calls_last_frame,
        r.draw_immediate_calls_last_frame,
        r.draw_instanced_calls_last_frame,
        r.immediate_vertices_last_frame,
        r.instanced_data_items_last_frame,
        r.vertices_drawn_last_frame,
        r.uniform_cache_hits_last_frame,
        r.uniform_cache_misses_last_frame,
        r.texture_binds_skipped,
        r.recv_wait_us_last_frame,
        r.recv_wait_count_last_frame,
        r.shader_bind_commands_last_frame,
        r.shader_redundant_binds_last_frame,
        r.shader_distinct_programs_last_frame,
        cats.join(",\n\x20\x20"),
    )
}

/// Serialize a full snapshot to JSON. Render-thread fields nest under
/// "render"; main-thread producer fields stay at the top level.
fn snapshot_to_json(s: &StatsSnapshot) -> String {
    let mut json = format!(
        "{{\n\
         \x20 \"server_time_us\": {},\n\
         \x20 \"main_thread_wait_us\": {},\n\
         \x20 \"send_blocked_us_last_frame\": {},\n\
         \x20 \"send_block_count_last_frame\": {},\n\
         \x20 \"channel_high_water\": {},\n\
         \x20 \"frames_in_flight\": {},\n\
         \x20 \"uniform_dedup_skips_last_frame\": {},\n",
        s.server_time_us,
        s.main_thread_wait_us,
        s.send_blocked_us_last_frame,
        s.send_block_count_last_frame,
        s.channel_high_water,
        s.frames_in_flight,
        s.uniform_dedup_skips_last_frame,
    );
    // Embed the render-thread stats as a nested object, minus the trailing
    // newline so the closing brace of the outer object lines up.
    let render_json = render_stats_to_json(&s.render);
    let trimmed = render_json.trim_end_matches('\n');
    json.push_str("  \"render\": ");
    json.push_str(trimmed);
    json.push_str("\n}\n");
    json
}

/// Run the dashboard server until the process exits.
pub fn run_stats_server(port: u16, sink: StatsSink) {
    let addr = format!("127.0.0.1:{port}");
    let server = match tiny_http::Server::http(&addr) {
        Ok(s) => s,
        Err(e) => {
            error!("Stats dashboard: cannot bind {addr}: {e}");
            return;
        }
    };
    info!("Stats dashboard listening on http://{addr}");

    // Polling-friendly: requests are rare (browser polls at 2 Hz), so a short
    // non-blocking loop keeps the thread responsive without busy-spinning.
    loop {
        match server.recv_timeout(Duration::from_millis(200)) {
            Ok(Some(request)) => {
                let url = request.url().to_string();
                let method = request.method().clone();
                let response: tiny_http::Response<std::io::Cursor<Vec<u8>>> = match (method, url.as_str()) {
                    (tiny_http::Method::Get, "/") => {
                        let body = include_str!("stats_dashboard.html");
                        tiny_http::Response::from_data(body.as_bytes().to_vec())
                            .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
                    }
                    (tiny_http::Method::Get, "/stats.json") => {
                        let snapshot = sink.lock().unwrap_or_else(|p| p.into_inner()).clone();
                        let body = snapshot_to_json(&snapshot);
                        tiny_http::Response::from_data(body.into_bytes())
                            .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
                    }
                    // Live memory report (Lua object census). Rows are
                    // (category, count, bytes) pushed by the Lua
                    // MemoryReporter module; a rising category is a leak.
                    (tiny_http::Method::Get, "/memory.json") => {
                        let (frame, entries) = crate::system::memory_report::snapshot();
                        let mut body = String::from("{\"frame\":");
                        body.push_str(&frame.to_string());
                        body.push_str(",\"entries\":[");
                        for (i, e) in entries.iter().enumerate() {
                            if i > 0 {
                                body.push(',');
                            }
                            body.push_str(&format!(
                                "{{\"category\":\"{}\",\"count\":{},\"bytes\":{}}}",
                                e.category.replace('\\', "\\\\").replace('"', "\\\""),
                                e.count,
                                e.bytes
                            ));
                        }
                        body.push_str("]}");
                        tiny_http::Response::from_data(body.into_bytes())
                            .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
                    }
                    // Live producer-scope table (Lua-side profiler). Same
                    // filtering as the F10 console print, served read-only.
                    (tiny_http::Method::Get, "/profile.json") => {
                        let scopes = crate::system::Profiler::snapshot();
                        let enabled = crate::system::Profiler::is_enabled();
                        let mut body = String::from("{\"enabled\":");
                        body.push_str(if enabled { "true" } else { "false" });
                        body.push_str(",\"scopes\":[");
                        for (i, s) in scopes.iter().enumerate() {
                            if i > 0 {
                                body.push(',');
                            }
                            body.push_str(&format!(
                                "{{\"name\":\"{}\",\"parent\":\"{}\",\"scope_pct\":{:.1},\"cumul_pct\":{:.0},\"total_ms\":{:.0},\"min_ms\":{:.2},\"max_ms\":{:.2},\"mean_ms\":{:.2}}}",
                                s.name.replace('\\', "\\\\").replace('"', "\\\""),
                                s.parent.replace('\\', "\\\\").replace('"', "\\\""),
                                s.scope_pct,
                                s.cumul_pct,
                                s.total_ms,
                                s.min_ms,
                                s.max_ms,
                                s.mean_ms,
                            ));
                        }
                        body.push_str("]}");
                        tiny_http::Response::from_data(body.into_bytes())
                            .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
                    }
                    // Dashboard toggle for the producer profiler. The request
                    // is picked up on the main thread's next safe point
                    // (Application:onPreRender, same as F10) - never toggled
                    // here, where disable() could panic mid-scope.
                    (tiny_http::Method::Get, "/profile/toggle") => {
                        crate::system::Profiler::request_toggle();
                        tiny_http::Response::from_data(b"{\"ok\":true}".to_vec())
                            .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
                    }
                    _ => tiny_http::Response::from_data(b"not found".to_vec()).with_status_code(404),
                };
                if let Err(e) = request.respond(response) {
                    warn!("Stats dashboard: failed to respond: {e}");
                }
            }
            Ok(None) => {}
            Err(_) => {}
        }
    }
}

/// Spawn the dashboard server on a dedicated thread. Returns immediately.
pub fn spawn_stats_server(port: u16, sink: StatsSink) {
    std::thread::Builder::new()
        .name("StatsDashboard".into())
        .spawn(move || run_stats_server(port, sink))
        .expect("Failed to spawn stats dashboard server");
}

/// Convenience: shared sink + spawned server in one call (used by the engine).
pub fn start_stats_server(port: u16) -> Arc<std::sync::Mutex<StatsSnapshot>> {
    let sink: StatsSink = Arc::new(std::sync::Mutex::new(StatsSnapshot::default()));
    spawn_stats_server(port, sink.clone());
    sink
}
