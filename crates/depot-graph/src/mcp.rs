use std::io::{self, BufRead, Write};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::graph::CodeGraph;
use crate::query::{GraphQuery, QueryOptions};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

pub struct McpServer;

impl McpServer {
    pub fn run_stdio(graph: CodeGraph) -> io::Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line in stdin.lock().lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let req: JsonRpcRequest = match serde_json::from_str(trimmed) {
                Ok(r) => r,
                Err(e) => {
                    let err_resp = JsonRpcResponse {
                        jsonrpc: "2.0",
                        id: None,
                        result: None,
                        error: Some(json!({ "code": -32700, "message": e.to_string() })),
                    };
                    writeln!(stdout, "{}", serde_json::to_string(&err_resp).unwrap())?;
                    stdout.flush()?;
                    continue;
                }
            };

            let resp = Self::handle_request(&graph, req);
            if let Some(r) = resp {
                writeln!(stdout, "{}", serde_json::to_string(&r).unwrap())?;
                stdout.flush()?;
            }
        }

        Ok(())
    }

    fn handle_request(graph: &CodeGraph, req: JsonRpcRequest) -> Option<JsonRpcResponse> {
        let id = req.id;
        match req.method.as_str() {
            "initialize" => {
                let res = json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "depot-graph-mcp",
                        "version": "0.2.0"
                    }
                });
                Some(JsonRpcResponse {
                    jsonrpc: "2.0",
                    id,
                    result: Some(res),
                    error: None,
                })
            }
            "notifications/initialized" => None,
            "tools/list" => {
                let tools = json!({
                    "tools": [
                        {
                            "name": "graph_query",
                            "description": "Traverse code knowledge graph around a concept or symbol with BFS/DFS and token budget",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "term": { "type": "string", "description": "Search concept or symbol" },
                                    "dfs": { "type": "boolean", "description": "Use DFS instead of BFS" },
                                    "max_tokens": { "type": "integer", "description": "Budget token limit (default: 2000)" }
                                },
                                "required": ["term"]
                            }
                        },
                        {
                            "name": "graph_path",
                            "description": "Find shortest structural path between two concepts or symbols in codebase",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "start": { "type": "string", "description": "Starting symbol name" },
                                    "end": { "type": "string", "description": "Target symbol name" }
                                },
                                "required": ["start", "end"]
                            }
                        },
                        {
                            "name": "graph_explain",
                            "description": "Retrieve complete dossier of a symbol (source file, exact lines, callers, callees, community)",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "term": { "type": "string", "description": "Symbol name or node id" }
                                },
                                "required": ["term"]
                            }
                        },
                        {
                            "name": "graph_stats",
                            "description": "Get summary metrics of the loaded knowledge graph",
                            "inputSchema": {
                                "type": "object",
                                "properties": {}
                            }
                        }
                    ]
                });
                Some(JsonRpcResponse {
                    jsonrpc: "2.0",
                    id,
                    result: Some(tools),
                    error: None,
                })
            }
            "tools/call" => {
                let params = req.params.unwrap_or(Value::Null);
                let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or(json!({}));

                let content = match tool_name {
                    "graph_query" => {
                        let term = args.get("term").and_then(|v| v.as_str()).unwrap_or("");
                        let dfs = args.get("dfs").and_then(|v| v.as_bool()).unwrap_or(false);
                        let max_tokens = args.get("max_tokens").and_then(|v| v.as_u64()).unwrap_or(2000) as usize;
                        let opts = QueryOptions { dfs, max_tokens, max_depth: 3 };
                        GraphQuery::query(graph, term, &opts)
                    }
                    "graph_path" => {
                        let start = args.get("start").and_then(|v| v.as_str()).unwrap_or("");
                        let end = args.get("end").and_then(|v| v.as_str()).unwrap_or("");
                        match GraphQuery::path(graph, start, end) {
                            Ok(res) => res,
                            Err(e) => format!("Error: {}", e),
                        }
                    }
                    "graph_explain" => {
                        let term = args.get("term").and_then(|v| v.as_str()).unwrap_or("");
                        match GraphQuery::explain(graph, term) {
                            Ok(res) => res,
                            Err(e) => format!("Error: {}", e),
                        }
                    }
                    "graph_stats" => {
                        format!(
                            "Graph Statistics:\n- Nodes: {}\n- Edges: {}\n- Directed: {}\n",
                            graph.nodes.len(),
                            graph.edges.len(),
                            graph.directed
                        )
                    }
                    _ => format!("Unknown tool: {}", tool_name),
                };

                let res = json!({
                    "content": [
                        {
                            "type": "text",
                            "text": content
                        }
                    ]
                });

                Some(JsonRpcResponse {
                    jsonrpc: "2.0",
                    id,
                    result: Some(res),
                    error: None,
                })
            }
            _ => Some(JsonRpcResponse {
                jsonrpc: "2.0",
                id,
                result: None,
                error: Some(json!({ "code": -32601, "message": "Method not found" })),
            }),
        }
    }
}
