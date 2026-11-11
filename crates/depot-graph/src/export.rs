use std::fs;
use std::path::Path;
use crate::graph::CodeGraph;

pub struct GraphExporter;

impl GraphExporter {
    pub fn export_all(
        graph: &mut CodeGraph,
        out_dir: &Path,
        codebase_name: &str,
    ) -> Result<(), String> {
        fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;

        let god_nodes = graph.compute_degrees_and_god_nodes(12);
        let num_communities = graph.detect_communities();

        // 1. graph.json (Graphify compatible)
        let json_path = out_dir.join("graph.json");
        let json_data = serde_json::to_string_pretty(graph).map_err(|e| e.to_string())?;
        fs::write(&json_path, &json_data).map_err(|e| e.to_string())?;

        // 2. GRAPH_REPORT.md
        let report_path = out_dir.join("GRAPH_REPORT.md");
        let report_content = Self::generate_report(graph, codebase_name, &god_nodes, num_communities);
        fs::write(&report_path, report_content).map_err(|e| e.to_string())?;

        // 3. ARCHITECTURE.mermaid
        let mermaid_path = out_dir.join("ARCHITECTURE.mermaid");
        let mermaid_content = Self::generate_mermaid(graph, 50);
        fs::write(&mermaid_path, mermaid_content).map_err(|e| e.to_string())?;

        // 4. graph.html (Standalone interactive force-directed canvas)
        let html_path = out_dir.join("graph.html");
        let html_content = Self::generate_html(graph, &json_data, codebase_name);
        fs::write(&html_path, html_content).map_err(|e| e.to_string())?;

        // 5. graph.svg (Static visual diagram)
        let svg_path = out_dir.join("graph.svg");
        let svg_content = Self::generate_svg(graph, 60);
        fs::write(&svg_path, svg_content).map_err(|e| e.to_string())?;

        // 6. graph.graphml (Gephi / yEd)
        let graphml_path = out_dir.join("graph.graphml");
        let graphml_content = Self::generate_graphml(graph);
        fs::write(&graphml_path, graphml_content).map_err(|e| e.to_string())?;

        // 7. cypher.txt (Neo4j / FalkorDB)
        let cypher_path = out_dir.join("cypher.txt");
        let cypher_content = Self::generate_cypher(graph);
        fs::write(&cypher_path, cypher_content).map_err(|e| e.to_string())?;

        Ok(())
    }

    fn generate_report(
        graph: &CodeGraph,
        codebase_name: &str,
        god_nodes: &[(String, usize)],
        num_communities: usize,
    ) -> String {
        let mut r = String::new();
        r.push_str(&format!("# Graph Report - {}\n\n", codebase_name));

        r.push_str("## Corpus Check\n");
        r.push_str(&format!(
            "- **Nodes:** {} · **Edges:** {} · **Communities:** {}\n",
            graph.nodes.len(),
            graph.edges.len(),
            num_communities
        ));
        r.push_str("- **Extraction Engine:** 100% Native Tree-Sitter AST & Justfile Integration\n\n");

        r.push_str("## God Nodes (Most connected abstractions)\n");
        for (idx, (node_id, degree)) in god_nodes.iter().enumerate() {
            let node = graph.nodes.iter().find(|n| &n.id == node_id);
            let label = node.map(|n| n.label.as_str()).unwrap_or(node_id.as_str());
            let kind = node.map(|n| n.file_type.as_str()).unwrap_or("unknown");
            let loc = node.map(|n| format!(" ({}:{})", n.source_file, n.source_location)).unwrap_or_default();
            r.push_str(&format!("{}. `{}` [{}] - {} connections{}\n", idx + 1, label, kind, degree, loc));
        }

        r.push_str("\n## Key Structural Connections\n");
        for edge in graph.edges.iter().take(20) {
            let src = graph.nodes.iter().find(|n| n.id == edge.source).map(|n| n.label.as_str()).unwrap_or(&edge.source);
            let tgt = graph.nodes.iter().find(|n| n.id == edge.target).map(|n| n.label.as_str()).unwrap_or(&edge.target);
            r.push_str(&format!("- `{}` --[{}]--> `{}` ({:?})\n", src, edge.relation, tgt, edge.provenance));
        }

        r.push_str("\n## Architecture Diagram (Mermaid Preview)\n\n```mermaid\n");
        r.push_str(&Self::generate_mermaid(graph, 35));
        r.push_str("\n```\n");

        r
    }

    pub fn generate_mermaid(graph: &CodeGraph, limit_nodes: usize) -> String {
        let mut m = String::new();
        m.push_str("flowchart TD\n");

        let visible_nodes: Vec<&crate::graph::GraphNode> =
            graph.nodes.iter().take(limit_nodes).collect();
        let visible_ids: std::collections::HashSet<&str> =
            visible_nodes.iter().map(|n| n.id.as_str()).collect();

        for node in &visible_nodes {
            let clean_label = node.label.replace('"', "").replace('(', "").replace(')', "");
            m.push_str(&format!("    {}[\"{}\"]\n", node.id, clean_label));
        }

        for edge in &graph.edges {
            if visible_ids.contains(edge.source.as_str()) && visible_ids.contains(edge.target.as_str()) {
                m.push_str(&format!(
                    "    {} -->|{}| {}\n",
                    edge.source, edge.relation, edge.target
                ));
            }
        }

        m
    }

    pub fn generate_svg(graph: &CodeGraph, limit: usize) -> String {
        let visible_nodes: Vec<&crate::graph::GraphNode> = graph.nodes.iter().take(limit).collect();
        let width = 1400;
        let height = 900;
        let cx = width as f64 / 2.0;
        let cy = height as f64 / 2.0;
        let radius = 380.0;

        let mut node_coords: std::collections::HashMap<&str, (f64, f64)> = std::collections::HashMap::new();
        let count = visible_nodes.len().max(1);

        for (i, node) in visible_nodes.iter().enumerate() {
            let angle = (i as f64 / count as f64) * 2.0 * std::f64::consts::PI;
            let x = cx + radius * angle.cos();
            let y = cy + radius * angle.sin();
            node_coords.insert(&node.id, (x, y));
        }

        let mut svg = String::new();
        svg.push_str(&format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {} {}\" width=\"100%\" height=\"100%\" style=\"background:#0d1117;font-family:ui-monospace,SFMono-Regular,Consolas,monospace;\">",
            width, height
        ));
        svg.push_str("<defs><marker id=\"arrow\" viewBox=\"0 0 10 10\" refX=\"22\" refY=\"5\" markerWidth=\"6\" markerHeight=\"6\" orient=\"auto-start-reverse\"><path d=\"M 0 0 L 10 5 L 0 10 z\" fill=\"#58a6ff\"/></marker></defs>");

        // Edges
        for edge in &graph.edges {
            if let (Some(&(x1, y1)), Some(&(x2, y2))) = (node_coords.get(edge.source.as_str()), node_coords.get(edge.target.as_str())) {
                svg.push_str(&format!(
                    "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"#30363d\" stroke-width=\"1.5\" marker-end=\"url(#arrow)\" />",
                    x1, y1, x2, y2
                ));
            }
        }

        // Nodes
        for node in &visible_nodes {
            let &(x, y) = node_coords.get(node.id.as_str()).unwrap();
            let hue = (node.community * 67) % 360;
            let fill = format!("hsl({}, 70%, 50%)", hue);

            svg.push_str(&format!(
                "<g transform=\"translate({:.1}, {:.1})\"><circle r=\"12\" fill=\"{}\" stroke=\"#f0f6fc\" stroke-width=\"1.5\"/><text x=\"16\" y=\"4\" fill=\"#c9d1d9\" font-size=\"12\" font-weight=\"600\">{}</text></g>",
                x, y, fill, node.label
            ));
        }

        svg.push_str("</svg>");
        svg
    }

    pub fn generate_graphml(graph: &CodeGraph) -> String {
        let mut g = String::new();
        g.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        g.push_str("<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\">\n");
        g.push_str("  <key id=\"label\" for=\"node\" attr.name=\"label\" attr.type=\"string\"/>\n");
        g.push_str("  <key id=\"file_type\" for=\"node\" attr.name=\"file_type\" attr.type=\"string\"/>\n");
        g.push_str("  <key id=\"source_file\" for=\"node\" attr.name=\"source_file\" attr.type=\"string\"/>\n");
        g.push_str("  <key id=\"community\" for=\"node\" attr.name=\"community\" attr.type=\"int\"/>\n");
        g.push_str("  <key id=\"relation\" for=\"edge\" attr.name=\"relation\" attr.type=\"string\"/>\n");
        g.push_str("  <key id=\"provenance\" for=\"edge\" attr.name=\"provenance\" attr.type=\"string\"/>\n");
        g.push_str("  <graph id=\"G\" edgedefault=\"directed\">\n");

        for n in &graph.nodes {
            g.push_str(&format!("    <node id=\"{}\">\n", n.id));
            g.push_str(&format!("      <data key=\"label\">{}</data>\n", escape_xml(&n.label)));
            g.push_str(&format!("      <data key=\"file_type\">{}</data>\n", escape_xml(&n.file_type)));
            g.push_str(&format!("      <data key=\"source_file\">{}</data>\n", escape_xml(&n.source_file)));
            g.push_str(&format!("      <data key=\"community\">{}</data>\n", n.community));
            g.push_str("    </node>\n");
        }

        for (i, e) in graph.edges.iter().enumerate() {
            g.push_str(&format!("    <edge id=\"e{}\" source=\"{}\" target=\"{}\">\n", i, e.source, e.target));
            g.push_str(&format!("      <data key=\"relation\">{}</data>\n", escape_xml(&e.relation)));
            g.push_str(&format!("      <data key=\"provenance\">{:?}</data>\n", e.provenance));
            g.push_str("    </edge>\n");
        }

        g.push_str("  </graph>\n</graphml>\n");
        g
    }

    pub fn generate_cypher(graph: &CodeGraph) -> String {
        let mut c = String::new();
        c.push_str("// Neo4j & FalkorDB Cypher Export\n");
        c.push_str("CREATE CONSTRAINT IF NOT EXISTS FOR (n:Node) REQUIRE n.id IS UNIQUE;\n\n");

        for n in &graph.nodes {
            let label = n.label.replace('\'', "\\'");
            let file = n.source_file.replace('\'', "\\'");
            c.push_str(&format!(
                "MERGE (:Node {{id: '{}', label: '{}', type: '{}', file: '{}', loc: '{}', community: {}}});\n",
                n.id, label, n.file_type, file, n.source_location, n.community
            ));
        }

        c.push_str("\n");
        for e in &graph.edges {
            let rel = e.relation.to_uppercase().replace(|c: char| !c.is_alphanumeric(), "_");
            c.push_str(&format!(
                "MATCH (s:Node {{id: '{}'}}), (t:Node {{id: '{}'}}) MERGE (s)-[:{} {{provenance: '{:?}'}}]->(t);\n",
                e.source, e.target, rel, e.provenance
            ));
        }

        c
    }

    pub fn generate_html(_graph: &CodeGraph, json_data: &str, title: &str) -> String {
        let template = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Graphify: __TITLE__</title>
<style>
* { box-sizing: border-box; margin: 0; padding: 0; }
body { background: #0b0f19; color: #e2e8f0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; overflow: hidden; height: 100vh; display: flex; }
#canvas-container { flex: 1; position: relative; height: 100%; cursor: grab; }
#canvas-container:active { cursor: grabbing; }
canvas { display: block; width: 100%; height: 100%; }
#sidebar { width: 340px; background: #111827; border-left: 1px solid #1f2937; padding: 18px; display: flex; flex-direction: column; gap: 14px; overflow-y: auto; }
h1 { font-size: 1.1rem; color: #60a5fa; font-weight: 700; margin-bottom: 4px; }
.search-box { width: 100%; padding: 8px 12px; background: #1f2937; border: 1px solid #374151; border-radius: 6px; color: #f9fafb; font-size: 0.9rem; }
.controls { display: flex; gap: 8px; }
button { flex: 1; padding: 7px 10px; background: #2563eb; color: #fff; border: none; border-radius: 6px; cursor: pointer; font-size: 0.82rem; font-weight: 600; }
button:hover { background: #1d4ed8; }
.card { background: #1f2937; border-radius: 6px; padding: 12px; border: 1px solid #374151; font-size: 0.85rem; line-height: 1.5; }
.card h3 { font-size: 0.95rem; color: #93c5fd; margin-bottom: 6px; }
.badge { display: inline-block; padding: 2px 6px; border-radius: 4px; font-size: 0.72rem; font-weight: 600; text-transform: uppercase; margin-right: 4px; }
.badge-struct { background: #065f46; color: #34d399; }
.badge-function { background: #1e40af; color: #93c5fd; }
.badge-file { background: #374151; color: #d1d5db; }
.badge-recipe { background: #92400e; color: #fcd34d; }
.stats { font-size: 0.8rem; color: #9ca3af; }
</style>
</head>
<body>
<div id="canvas-container">
  <canvas id="graph-canvas"></canvas>
</div>
<div id="sidebar">
  <div>
    <h1>Graphify Architecture</h1>
    <div class="stats" id="stats-display">Loading graph...</div>
  </div>
  <input type="text" id="search-input" class="search-box" placeholder="Filter symbol (e.g. Engine, main)...">
  <div class="controls">
    <button id="btn-pause">Pause Physics</button>
    <button id="btn-reset">Reset View</button>
  </div>
  <div id="selection-card" class="card">
    <h3>Node Details</h3>
    <p style="color:#9ca3af;">Click on any node in the graph to inspect definition, lines, and relationships.</p>
  </div>
</div>

<script id="graph-data" type="application/json">
__GRAPH_JSON__
</script>

<script>
(function() {
  const rawData = JSON.parse(document.getElementById('graph-data').textContent);
  const canvas = document.getElementById('graph-canvas');
  const ctx = canvas.getContext('2d');
  const statsDisplay = document.getElementById('stats-display');
  const selectionCard = document.getElementById('selection-card');
  const searchInput = document.getElementById('search-input');
  const btnPause = document.getElementById('btn-pause');
  const btnReset = document.getElementById('btn-reset');

  statsDisplay.textContent = `${rawData.nodes.length} nodes · ${rawData.edges.length} edges`;

  let width = canvas.width = canvas.parentElement.clientWidth;
  let height = canvas.height = canvas.parentElement.clientHeight;

  window.addEventListener('resize', () => {
    width = canvas.width = canvas.parentElement.clientWidth;
    height = canvas.height = canvas.parentElement.clientHeight;
  });

  const nodes = rawData.nodes.map((n, i) => {
    const angle = (i / rawData.nodes.length) * 2 * Math.PI;
    const r = Math.min(width, height) * 0.35;
    return {
      id: n.id,
      label: n.label,
      type: n.file_type,
      file: n.source_file,
      loc: n.source_location,
      community: n.community,
      degree: n.degree || 1,
      signature: n.signature,
      x: width / 2 + r * Math.cos(angle) + (Math.random() - 0.5) * 50,
      y: height / 2 + r * Math.sin(angle) + (Math.random() - 0.5) * 50,
      vx: 0,
      vy: 0,
      radius: Math.max(5, Math.min(18, 5 + Math.sqrt(n.degree || 1) * 3))
    };
  });

  const nodeMap = new Map();
  nodes.forEach(n => nodeMap.set(n.id, n));

  const edges = rawData.edges.map(e => ({
    source: nodeMap.get(e.source),
    target: nodeMap.get(e.target),
    relation: e.relation
  })).filter(e => e.source && e.target);

  let camera = { x: 0, y: 0, zoom: 1 };
  let isRunning = true;
  let hoveredNode = null;
  let selectedNode = null;
  let draggedNode = null;
  let isPanning = false;
  let lastMouse = { x: 0, y: 0 };
  let searchTerm = '';

  function getColor(c) {
    return `hsl(${(c * 67) % 360}, 75%, 55%)`;
  }

  function simulate() {
    if (!isRunning) return;
    const repel = 180;
    const springLen = 60;
    const springK = 0.04;

    for (let i = 0; i < nodes.length; i++) {
      for (let j = i + 1; j < nodes.length; j++) {
        let dx = nodes[j].x - nodes[i].x;
        let dy = nodes[j].y - nodes[i].y;
        let distSq = dx * dx + dy * dy + 1;
        if (distSq < 90000) {
          let dist = Math.sqrt(distSq);
          let force = repel / distSq;
          let fx = (dx / dist) * force;
          let fy = (dy / dist) * force;
          nodes[i].vx -= fx;
          nodes[i].vy -= fy;
          nodes[j].vx += fx;
          nodes[j].vy += fy;
        }
      }
    }

    edges.forEach(e => {
      let dx = e.target.x - e.source.x;
      let dy = e.target.y - e.source.y;
      let dist = Math.sqrt(dx * dx + dy * dy) || 1;
      let force = (dist - springLen) * springK;
      let fx = (dx / dist) * force;
      let fy = (dy / dist) * force;
      e.source.vx += fx;
      e.source.vy += fy;
      e.target.vx -= fx;
      e.target.vy -= fy;
    });

    nodes.forEach(n => {
      if (n === draggedNode) return;
      n.x += n.vx;
      n.y += n.vy;
      n.vx *= 0.85;
      n.vy *= 0.85;
    });
  }

  function render() {
    ctx.clearRect(0, 0, width, height);
    ctx.save();
    ctx.translate(camera.x, camera.y);
    ctx.scale(camera.zoom, camera.zoom);

    // Edges
    edges.forEach(e => {
      ctx.beginPath();
      ctx.moveTo(e.source.x, e.source.y);
      ctx.lineTo(e.target.x, e.target.y);
      ctx.strokeStyle = '#1e293b';
      ctx.lineWidth = 1;
      ctx.stroke();
    });

    // Nodes
    nodes.forEach(n => {
      const match = searchTerm && n.label.toLowerCase().includes(searchTerm.toLowerCase());
      ctx.beginPath();
      ctx.arc(n.x, n.y, n.radius, 0, Math.PI * 2);
      ctx.fillStyle = getColor(n.community);
      ctx.fill();

      if (n === selectedNode || match) {
        ctx.strokeStyle = match ? '#f59e0b' : '#38bdf8';
        ctx.lineWidth = 3;
        ctx.stroke();
      }

      if (camera.zoom > 0.8 || n === selectedNode || n === hoveredNode || match) {
        ctx.font = '10px ui-monospace, sans-serif';
        ctx.fillStyle = '#cbd5e1';
        ctx.fillText(n.label, n.x + n.radius + 3, n.y + 3);
      }
    });

    ctx.restore();
    simulate();
    requestAnimationFrame(render);
  }

  function toWorld(px, py) {
    return {
      x: (px - camera.x) / camera.zoom,
      y: (py - camera.y) / camera.zoom
    };
  }

  canvas.addEventListener('mousedown', e => {
    const w = toWorld(e.clientX, e.clientY);
    const hit = nodes.find(n => Math.hypot(n.x - w.x, n.y - w.y) < n.radius + 3);
    if (hit) {
      draggedNode = hit;
      selectedNode = hit;
      showDetails(hit);
    } else {
      isPanning = true;
      lastMouse = { x: e.clientX, y: e.clientY };
    }
  });

  window.addEventListener('mousemove', e => {
    if (draggedNode) {
      const w = toWorld(e.clientX, e.clientY);
      draggedNode.x = w.x;
      draggedNode.y = w.y;
    } else if (isPanning) {
      camera.x += e.clientX - lastMouse.x;
      camera.y += e.clientY - lastMouse.y;
      lastMouse = { x: e.clientX, y: e.clientY };
    }
  });

  window.addEventListener('mouseup', () => {
    draggedNode = null;
    isPanning = false;
  });

  canvas.addEventListener('wheel', e => {
    e.preventDefault();
    const factor = e.deltaY < 0 ? 1.15 : 0.87;
    camera.zoom = Math.max(0.2, Math.min(5, camera.zoom * factor));
  });

  searchInput.addEventListener('input', e => {
    searchTerm = e.target.value.trim();
  });

  btnPause.addEventListener('click', () => {
    isRunning = !isRunning;
    btnPause.textContent = isRunning ? 'Pause Physics' : 'Resume Physics';
  });

  btnReset.addEventListener('click', () => {
    camera = { x: 0, y: 0, zoom: 1 };
  });

  function showDetails(node) {
    let out = `<h3>${escapeHtml(node.label)}</h3>`;
    out += `<div><span class="badge badge-${node.type}">${node.type}</span> Community ${node.community}</div>`;
    out += `<div style="margin-top:8px;"><strong>File:</strong> <code>${escapeHtml(node.file)}</code></div>`;
    out += `<div><strong>Span:</strong> <code>${node.loc}</code></div>`;
    out += `<div><strong>Degree:</strong> ${node.degree}</div>`;
    if (node.signature) {
      out += `<div style="margin-top:6px;"><strong>Signature:</strong><pre style="background:#0f172a;padding:6px;border-radius:4px;overflow-x:auto;">${escapeHtml(node.signature)}</pre></div>`;
    }
    selectionCard.innerHTML = out;
  }

  function escapeHtml(str) {
    return (str || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  }

  requestAnimationFrame(render);
})();
</script>
</body>
</html>
"##;

        template
            .replace("__TITLE__", title)
            .replace("__GRAPH_JSON__", json_data)
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
