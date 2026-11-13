//! Runs the agent's path (M5–M9 and M11, with a failed report) against the real `local` binary,
//! through an MCP client, the way an LLM client would.

use std::net::TcpListener;
use std::process::{Child, Command};
use std::time::Duration;

use rmcp::ServiceExt;
use rmcp::model::{CallToolRequestParams, CallToolResult, ClientConfig};
use rmcp::service::{RoleClient, RunningService};
use rmcp::transport::StreamableHttpClientTransport;
use rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig;
use serde_json::{Value, json};

const TOKEN: &str = "dev-token";
const PROJECT: &str = "reading-list";
const READY_ATTEMPTS: u32 = 100;
const READY_WAIT: Duration = Duration::from_millis(50);

/// The local binary, run where no `grado.toml` is, with only the environment the test sets.
fn server() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_local"));
    command.current_dir(env!("CARGO_TARGET_TMPDIR")).env_clear();
    command
}

/// The server process, stopped when the test ends.
struct Server {
    child: Child,
    addr: String,
}

impl Server {
    async fn start(stamp: &str) -> Self {
        let addr =
            TcpListener::bind("127.0.0.1:0").and_then(|listener| listener.local_addr()).unwrap().to_string();
        // Built right away, so `Drop` stops the process even if it never starts listening.
        let server = Self {
            child: server()
                .env("GRADO_ADDR", &addr)
                .env("GRADO_LOG__LEVEL", "off")
                .env("GRADO_LOG__FORMAT", "json")
                .env("GRADO_DEV__AGENT_TOKEN", TOKEN)
                .env("GRADO_DEV__STAMP", stamp)
                .spawn()
                .expect("the server starts"),
            addr,
        };
        for _ in 0..READY_ATTEMPTS {
            if tokio::net::TcpStream::connect(&server.addr).await.is_ok() {
                return server;
            }
            tokio::time::sleep(READY_WAIT).await;
        }
        panic!("the server didn't start listening on {}", server.addr);
    }

    async fn agent(&self, token: &str) -> Agent {
        let config = StreamableHttpClientTransportConfig::with_uri(format!("http://{}/mcp", self.addr))
            .auth_header(token);
        let client = ClientConfig::default()
            .serve(StreamableHttpClientTransport::from_config(config))
            .await
            .expect("the client connects");
        Agent { client }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

struct Agent {
    client: RunningService<RoleClient, ClientConfig>,
}

impl Agent {
    async fn call(&self, tool: &'static str, arguments: Value) -> CallToolResult {
        let Value::Object(arguments) = arguments else { panic!("tool arguments are an object") };
        self.client
            .call_tool(CallToolRequestParams::new(tool).with_arguments(arguments))
            .await
            .expect("the tool call reaches the server")
    }

    /// The JSON a successful call returned.
    async fn ok(&self, tool: &'static str, arguments: Value) -> Value {
        let result = self.call(tool, arguments).await;
        assert_ne!(result.is_error, Some(true), "{tool} was refused: {:?}", result.content);
        result.structured_content.expect("a structured response")
    }

    /// The text of a refused call.
    async fn refused(&self, tool: &'static str, arguments: Value) -> String {
        let result = self.call(tool, arguments).await;
        assert_eq!(result.is_error, Some(true), "{tool} wasn't refused");
        result.content.iter().filter_map(|block| block.as_text()).map(|text| text.text.clone()).collect()
    }
}

fn report(step: &str, verdicts: &[(&str, bool)], handoff_note: Option<&str>) -> Value {
    json!({
        "project": PROJECT,
        "step": step,
        "verdicts": verdicts.iter().map(|(criterion, pass)| json!({ "criterion": criterion, "pass": pass })).collect::<Vec<_>>(),
        "handoff_note": handoff_note,
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn an_agent_finishes_a_step_and_the_next_one_opens() {
    let server = Server::start("off").await;
    let agent = server.agent(TOKEN).await;

    // M5: find the project, then get the briefing.
    let projects = agent.ok("list_my_projects", json!({})).await;
    assert_eq!(projects["projects"], json!([{ "id": PROJECT, "title": "Reading list" }]));
    let briefing = agent.ok("get_briefing", json!({ "project": PROJECT })).await;
    assert_eq!(briefing["current"]["id"], "books");
    assert_eq!(briefing["current"]["state"], "open");
    assert_eq!(briefing["next"]["id"], "finished");

    // M7: the criteria, with the fresh-reviewer suggestion.
    let criteria = agent.ok("get_criteria", json!({ "project": PROJECT })).await;
    assert_eq!(criteria["step"], "books");
    let ids: Vec<&str> =
        criteria["criteria"].as_array().unwrap().iter().map(|c| c["id"].as_str().unwrap()).collect();
    assert_eq!(ids, ["book-saved", "title-required"]);
    assert!(criteria["next_action"].as_str().unwrap().contains("fresh agent"));

    // Failed report: recorded, and the agent is told to stop.
    let failed = agent
        .ok("send_report", report("books", &[("book-saved", true), ("title-required", false)], None))
        .await;
    assert_eq!(failed["state"], "open");
    assert_eq!(failed["failed"], json!(["title-required"]));
    assert!(failed["next_action"].as_str().unwrap().contains("Stop and tell the person"));

    // M8–M9: all pass needs a handoff note.
    let all_pass = [("book-saved", true), ("title-required", true)];
    let refusal = agent.refused("send_report", report("books", &all_pass, None)).await;
    assert!(refusal.contains("handoff note"), "{refusal}");

    // M11: with the stamp off the step is done and the next one opens.
    let done =
        agent.ok("send_report", report("books", &all_pass, Some("Books are saved to books.json."))).await;
    assert_eq!(done["state"], "done");
    assert_eq!(done["next_step"], "finished");
    assert!(done["next_action"].as_str().unwrap().contains("fresh agent"));

    // E4: a retried report gets the same answer.
    let again =
        agent.ok("send_report", report("books", &all_pass, Some("Books are saved to books.json."))).await;
    assert_eq!(again, done);

    let briefing = agent.ok("get_briefing", json!({ "project": PROJECT })).await;
    assert_eq!(briefing["current"]["id"], "finished");
    assert_eq!(briefing["handoff_note"], "Books are saved to books.json.");

    // E14: no stamps when the stamp is off.
    let refusal = agent.refused("get_stamp", json!({ "project": PROJECT, "step": "books" })).await;
    assert!(refusal.contains("doesn't use stamps"), "{refusal}");
}

#[tokio::test(flavor = "multi_thread")]
async fn with_the_stamp_on_an_approved_step_gets_a_stamp() {
    let server = Server::start("on").await;
    let agent = server.agent(TOKEN).await;

    let all_pass = [("book-saved", true), ("title-required", true)];
    let approved = agent.ok("send_report", report("books", &all_pass, Some("Books are saved."))).await;
    assert_eq!(approved["state"], "approved");

    // M9 and E10: a stamp, and a new one voids the old.
    let first = agent.ok("get_stamp", json!({ "project": PROJECT, "step": "books" })).await;
    let second = agent.ok("get_stamp", json!({ "project": PROJECT, "step": "books" })).await;
    assert_ne!(first["stamp"], second["stamp"]);

    let briefing = agent.ok("get_briefing", json!({ "project": PROJECT })).await;
    assert_eq!(briefing["current"]["state"], "approved");
}

#[test]
fn the_server_wont_start_on_an_incomplete_configuration() {
    let output = server().env("GRADO_DEV__STAMP", "yes").output().expect("the server runs");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("has 5 problems"), "{stderr}");
    assert!(stderr.contains("addr (GRADO_ADDR): missing"), "{stderr}");
}

#[tokio::test(flavor = "multi_thread")]
async fn an_unknown_connection_is_refused() {
    let server = Server::start("off").await;
    let agent = server.agent("someone-else").await;

    // E8.
    let refusal = agent.refused("list_my_projects", json!({})).await;
    assert!(refusal.contains("unknown or was revoked"), "{refusal}");
}
