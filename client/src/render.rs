//! Build the HTML page by hand with `format!`.
//! No template engine. Every value from outside goes through `escape`.

use crate::api::ApiCall;
use crate::{FormValues, Op};

/// One form on the page.
struct FormSpec {
    op: Op,
    method: &'static str,
    path: &'static str,
    button: &'static str,
    /// (input name, placeholder)
    fields: &'static [(&'static str, &'static str)],
}

/// All forms, top to bottom. Add a line here to add a new operation.
const FORMS: &[FormSpec] = &[
    FormSpec {
        op: Op::List,
        method: "GET",
        path: "/users",
        button: "List users",
        fields: &[],
    },
    FormSpec {
        op: Op::Get,
        method: "GET",
        path: "/users/{id}",
        button: "Get user",
        fields: &[("id", "e.g. 1")],
    },
    FormSpec {
        op: Op::Create,
        method: "POST",
        path: "/users",
        button: "Create user",
        fields: &[("name", "e.g. Carol"), ("email", "e.g. carol@example.com")],
    },
    FormSpec {
        op: Op::Update,
        method: "PUT",
        path: "/users/{id}",
        button: "Update user",
        fields: &[
            ("id", "e.g. 1"),
            ("name", "empty = no change"),
            ("email", "empty = no change"),
        ],
    },
    FormSpec {
        op: Op::Delete,
        method: "DELETE",
        path: "/users/{id}",
        button: "Delete user",
        fields: &[("id", "e.g. 1")],
    },
];

/// The whole page. `last_op` and `values` come from the form the user just sent.
pub fn page(api_base: &str, last_op: Op, values: &FormValues, call: &ApiCall) -> String {
    // Iterator -> String: `collect()` joins every form into one string.
    let forms: String = FORMS
        .iter()
        .map(|spec| form(spec, last_op, values))
        .collect();

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>User CRUD Client</title>
<style>{CSS}</style>
</head>
<body>
<header>
  <h1>User CRUD Client</h1>
  <p>API server: <code>{api_base}</code></p>
</header>
<main>
  <section class="ops">
{forms}
  </section>
  <section class="result">
{result}
  </section>
</main>
</body>
</html>"#,
        api_base = escape(api_base),
        result = result_block(call),
    )
}

/// One form. Only the form the user just sent keeps its values.
/// So a "Create" does not leave text in the "Update" form by mistake.
fn form(spec: &FormSpec, last_op: Op, v: &FormValues) -> String {
    let fields: String = spec
        .fields
        .iter()
        .map(|&(name, placeholder)| {
            let value = if spec.op == last_op {
                escape(field_value(v, name))
            } else {
                String::new()
            };
            format!(
                r#"      <label><span>{name}</span><input name="{name}" value="{value}" placeholder="{placeholder}"></label>
"#
            )
        })
        .collect();

    let danger = if spec.op == Op::Delete {
        r#" class="danger""#
    } else {
        ""
    };
    format!(
        r#"    <form method="post" action="/run">
      <input type="hidden" name="op" value="{op}">
      <h2><span class="m {class}">{method}</span> {path}</h2>
{fields}      <button{danger}>{button}</button>
    </form>
"#,
        op = spec.op.as_str(),
        class = spec.method.to_lowercase(),
        method = spec.method,
        path = spec.path,
        button = spec.button,
    )
}

fn field_value<'a>(v: &'a FormValues, name: &str) -> &'a str {
    match name {
        "id" => &v.id,
        "name" => &v.name,
        "email" => &v.email,
        _ => "",
    }
}

/// The right side: the raw request and the raw response.
fn result_block(call: &ApiCall) -> String {
    let request_body = match &call.request_body {
        Some(b) => format!("<pre>{}</pre>", escape(b)),
        None => r#"<p class="muted">(no body)</p>"#.to_string(),
    };

    let response = match &call.result {
        Ok(res) => {
            // Pick a color by status class: 2xx, 4xx, 5xx.
            let class = if res.status.is_success() {
                "ok"
            } else if res.status.is_client_error() {
                "warn"
            } else {
                "err"
            };
            let body = if res.body.is_empty() {
                r#"<p class="muted">(empty body)</p>"#.to_string()
            } else {
                format!("<pre>{}</pre>", escape(&res.body))
            };
            // `StatusCode`'s Display prints "201 Created".
            format!(
                r#"<h2>Response <span class="badge {class}">{status}</span> <small>{ms} ms</small></h2>
    {body}"#,
                status = res.status,
                ms = call.elapsed_ms,
            )
        }
        Err(e) => format!(
            r#"<h2>Response <span class="badge err">request failed</span> <small>{ms} ms</small></h2>
    <pre>{e}</pre>"#,
            ms = call.elapsed_ms,
            e = escape(e),
        ),
    };

    format!(
        r#"    <h2>Request</h2>
    <pre class="line">{method} {url}</pre>
    {request_body}
    {response}"#,
        method = call.method,
        url = escape(&call.url),
    )
}

/// Replace the 5 HTML special chars. This stops `<script>` in a name from running.
pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

const CSS: &str = r#"
:root { --bg:#f6f7f9; --card:#fff; --fg:#1b1f24; --muted:#6b7280; --line:#d8dde3;
        --pre:#0f172a; --pre-fg:#e2e8f0; --ok:#15803d; --warn:#b45309; --err:#b91c1c; --accent:#2563eb; }
@media (prefers-color-scheme: dark) {
  :root { --bg:#0d1117; --card:#161b22; --fg:#e6edf3; --muted:#8b949e; --line:#30363d; --pre:#010409; --pre-fg:#e6edf3; }
}
* { box-sizing: border-box; }
body { margin:0; font:15px/1.5 system-ui, sans-serif; background:var(--bg); color:var(--fg); }
header { padding:16px 24px; border-bottom:1px solid var(--line); }
header h1 { margin:0; font-size:20px; }
header p { margin:4px 0 0; color:var(--muted); overflow-wrap:anywhere; }
main { display:grid; grid-template-columns: minmax(280px, 380px) 1fr; gap:20px; padding:20px 24px; }
main > * { min-width:0; }
@media (max-width: 800px) { main { grid-template-columns: 1fr; padding:16px; } header { padding:16px; } }
.ops { display:flex; flex-direction:column; gap:12px; }
form, .result { background:var(--card); border:1px solid var(--line); border-radius:8px; padding:12px 14px; }
form h2, .result h2 { margin:0 0 8px; font-size:14px; font-family:ui-monospace, monospace; }
.result h2 { margin-top:14px; } .result h2:first-child { margin-top:0; }
label { display:flex; align-items:center; gap:8px; margin:6px 0; font-size:13px; color:var(--muted); }
label span { width:40px; flex:none; }
input { flex:1; min-width:0; padding:6px 8px; border:1px solid var(--line); border-radius:6px; background:var(--bg); color:var(--fg); font:inherit; }
button { margin-top:4px; padding:6px 12px; border:0; border-radius:6px; background:var(--accent); color:#fff; font:inherit; cursor:pointer; }
button.danger { background:var(--err); }
.m { display:inline-block; min-width:56px; padding:1px 6px; border-radius:4px; color:#fff; font-size:12px; text-align:center; }
.m.get { background:#2563eb; } .m.post { background:#15803d; } .m.put { background:#b45309; } .m.delete { background:#b91c1c; }
pre { margin:0 0 8px; padding:10px 12px; background:var(--pre); color:var(--pre-fg); border-radius:6px; overflow-x:auto;
      font:13px/1.45 ui-monospace, SFMono-Regular, Menlo, monospace; }
pre.line { white-space:pre-wrap; word-break:break-all; }
.badge { padding:2px 8px; border-radius:999px; color:#fff; font-size:12px; }
.badge.ok { background:var(--ok); } .badge.warn { background:var(--warn); } .badge.err { background:var(--err); }
small, .muted { color:var(--muted); font-weight:normal; }
.muted { margin:0 0 8px; font-size:13px; }
"#;
