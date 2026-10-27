// Grado website. Plain ES modules, no build step.
// Server-rendered Markdown (our own content) is inserted as HTML; everything
// that comes from a player is inserted as text.

const app = document.getElementById("app");
let me = null;

// ---------- helpers ----------

function h(tag, attrs = {}, ...children) {
  const el = document.createElement(tag);
  for (const [key, value] of Object.entries(attrs)) {
    if (value === undefined || value === null || value === false) continue;
    if (key.startsWith("on")) el.addEventListener(key.slice(2), value);
    else if (key === "html") el.innerHTML = value;
    else if (key === "class") el.className = value;
    else el.setAttribute(key, value === true ? "" : value);
  }
  for (const child of children.flat()) {
    if (child === null || child === undefined || child === false) continue;
    el.append(child instanceof Node ? child : document.createTextNode(String(child)));
  }
  return el;
}

const svg = (tag, attrs = {}, ...children) => {
  const el = document.createElementNS("http://www.w3.org/2000/svg", tag);
  for (const [key, value] of Object.entries(attrs)) el.setAttribute(key, value);
  for (const child of children.flat()) el.append(child instanceof Node ? child : document.createTextNode(String(child)));
  return el;
};

async function api(path, options = {}) {
  const response = await fetch(path, {
    ...options,
    headers: { "Content-Type": "application/json", ...(options.headers || {}) },
    body: options.body ? JSON.stringify(options.body) : undefined,
  });
  const data = await response.json().catch(() => ({}));
  if (!response.ok) {
    const error = new Error(data.error || `Request failed (${response.status})`);
    error.status = response.status;
    throw error;
  }
  return data;
}

function render(...nodes) {
  app.replaceChildren(...nodes.flat(Infinity).filter(Boolean));
  window.scrollTo(0, 0);
}

function notice(text, kind = "") {
  return h("p", { class: `notice ${kind}`, role: kind === "error" ? "alert" : "status" }, text);
}

function copyButton(text) {
  const button = h("button", { type: "button", class: "secondary", onclick: async () => {
    try {
      await navigator.clipboard.writeText(text);
      button.textContent = "Copied";
    } catch {
      button.textContent = "Select and copy manually";
    }
    setTimeout(() => (button.textContent = "Copy"), 2000);
  } }, "Copy");
  return button;
}

const STATUS_LABEL = { completed: "Stamped ✓", active: "You are here", locked: "Locked" };
// Elevation of each checkpoint on the route profile, in % of the climb.
const ELEVATION = [6, 14, 28, 36, 50, 58, 72, 80, 100];

// ---------- pages ----------

async function signInPage() {
  const name = h("input", { type: "text", id: "name", autocomplete: "name", required: true });
  const email = h("input", { type: "email", id: "email", autocomplete: "email", required: true });
  const message = h("div");
  const form = h("form", { class: "card stack", onsubmit: async (event) => {
    event.preventDefault();
    try {
      await api("/api/auth/dev-login", { method: "POST", body: { name: name.value, email: email.value } });
      await boot();
    } catch (error) {
      message.replaceChildren(notice(error.message, "error"));
    }
  } },
    h("div", {}, h("label", { for: "name" }, "Your name"), name),
    h("div", {}, h("label", { for: "email" }, "Email"), email),
    h("button", { type: "submit" }, "Start the route"),
    message,
    h("p", { class: "muted small" }, "Development sign-in: no password. Real sign-in replaces this before going public."),
  );
  render(
    h("h1", {}, "Grado"),
    h("p", { class: "muted" }, "A route of checkpoints for building software with an AI, one good habit at a time."),
    form,
  );
}

function routeProfile(levels) {
  const width = 800, height = 230, padX = 40, top = 30, bottom = 190;
  const points = levels.map((level, i) => ({
    x: padX + (i * (width - padX * 2)) / Math.max(levels.length - 1, 1),
    y: bottom - ((ELEVATION[i] ?? 100) / 100) * (bottom - top),
    level,
  }));
  const path = points.map((p, i) => `${i ? "L" : "M"}${p.x.toFixed(1)},${p.y.toFixed(1)}`).join(" ");
  const activeIndex = levels.findIndex((l) => l.status === "active");
  const reached = activeIndex === -1 ? points.length : activeIndex + 1;
  const donePath = points.slice(0, reached).map((p, i) => `${i ? "L" : "M"}${p.x.toFixed(1)},${p.y.toFixed(1)}`).join(" ");

  return svg("svg", { class: "profile", viewBox: `0 0 ${width} ${height}`, role: "img",
      "aria-label": `Route profile: ${levels.filter((l) => l.status === "completed").length} of ${levels.length} checkpoints stamped` },
    svg("path", { class: "ground", d: `${path} L${width - padX},${height} L${padX},${height} Z` }),
    svg("path", { class: "line", d: path }),
    reached > 1 ? svg("path", { class: "done-line", d: donePath }) : [],
    points.map((p) => {
      const locked = p.level.status === "locked";
      const circle = svg("circle", {
        class: `marker ${locked ? "locked" : ""}`,
        cx: p.x, cy: p.y, r: p.level.status === "active" ? 13 : 9,
        fill: locked ? "" : `var(--${p.level.environment})`,
      });
      const label = svg("text", { class: "label", x: p.x, y: p.y - 18, "text-anchor": "middle" }, String(p.level.number));
      const group = svg("g", {}, svg("title", {}, `${p.level.number}. ${p.level.title} (${STATUS_LABEL[p.level.status]})`), circle, label);
      if (locked) return group;
      const link = svg("a", { href: `#/level/${p.level.id}` }, group);
      return link;
    }),
  );
}

async function routePage() {
  const journey = await api("/api/journey");
  const active = journey.levels.find((l) => l.status === "active");
  const stamped = journey.levels.filter((l) => l.status === "completed").length;

  const legs = h("ul", { class: "legs" }, journey.levels.map((level) => {
    const inner = [
      h("span", { class: `badge env-${level.environment}`, "aria-hidden": "true" }, level.status === "completed" ? "✓" : level.number),
      h("div", {}, h("h3", {}, `${level.number}. ${level.title}`), h("p", {}, `${level.environment_label} · ${level.habit}`)),
      h("span", { class: "status" }, STATUS_LABEL[level.status]),
    ];
    return h("li", {}, level.status === "locked"
      ? h("div", { class: "leg locked" }, inner)
      : h("a", { class: `leg ${level.status}`, href: `#/level/${level.id}` }, inner));
  }));

  render(
    h("h1", {}, journey.finished ? `Summit reached, ${me.user.name}!` : `Hi ${me.user.name}`),
    h("p", { class: "muted" }, journey.finished
      ? "Every checkpoint is stamped. Your parked ideas are waiting for the next adventure."
      : `${stamped} of ${journey.levels.length} checkpoints stamped. ${active ? `Next up: ${active.title}.` : ""}`),
    h("div", { class: "card" }, routeProfile(journey.levels)),
    active ? h("p", {}, h("a", { class: "button", href: `#/level/${active.id}` }, `Continue: ${active.title} →`)) : null,
    legs,
  );
}

function startBox(level) {
  const command = `/mcp__${me.serverName || "grado"}__checkpoint`;
  return h("section", { class: "card stack" },
    h("h2", { style: "margin-top:0" }, "How to start"),
    h("ol", {},
      h("li", {}, "Open a terminal in your project folder and run ", h("code", {}, "claude"), "."),
      h("li", {}, "Type ", h("code", {}, command), " and press Enter. Your coach takes it from there."),
      h("li", {}, "When every check passes, your coach gives you a stamp. Enter it below."),
    ),
    h("p", { class: "muted small" }, "First time? Connect Claude Code on the ", h("a", { href: "#/connect" }, "Connect"), " page."),
  );
}

function stampForm(level) {
  const input = h("input", { type: "text", id: "stamp", placeholder: "STAMP-XXXX-XXXX", autocomplete: "off", spellcheck: "false" });
  const message = h("div");
  return h("form", { class: `card stack stamp-form ${level.ready_for_stamp ? "ready" : ""}`, onsubmit: async (event) => {
    event.preventDefault();
    try {
      const result = await api(`/api/levels/${level.id}/stamp`, { method: "POST", body: { stamp: input.value } });
      if (result.outcome === "redeemed") {
        message.replaceChildren(notice(result.message, "ok"));
        setTimeout(() => (location.hash = result.next_level_id ? `#/level/${result.next_level_id}` : "#/"), 1200);
      } else {
        message.replaceChildren(notice(result.message, "error"));
      }
    } catch (error) {
      message.replaceChildren(notice(error.message, "error"));
    }
  } },
    h("h2", { style: "margin-top:0" }, level.ready_for_stamp ? "Ready for your stamp" : "Got a stamp?"),
    h("div", { class: "row" },
      h("div", { class: "grow" }, h("label", { for: "stamp" }, "Checkpoint stamp"), input),
      h("button", { type: "submit" }, "Stamp it"),
    ),
    message,
  );
}

function guidesSection(level) {
  if (!level.guides.length) return null;
  const list = h("div");
  const draw = () => {
    const opened = level.guides.filter((g) => g.html);
    const next = level.guides.find((g) => !g.html);
    list.replaceChildren(
      ...opened.map((g, i) => h("details", { class: "guide", open: i === opened.length - 1 },
        h("summary", {}, g.title), h("div", { html: g.html }))),
      next ? h("div", { class: "locked-guide" },
        h("button", { type: "button", class: "secondary", onclick: async () => {
          const guide = await api(`/api/levels/${level.id}/guides/next`, { method: "POST" });
          Object.assign(level.guides.find((g) => g.tier === guide.tier), guide);
          draw();
        } }, opened.length ? "Still stuck? Open the next guide" : "Stuck? Open a hint"),
      ) : null,
    );
  };
  draw();
  return h("section", {}, h("h2", {}, "Stuck?"),
    h("p", { class: "muted small" }, "Guides go from a small hint to a full walkthrough. Your coach can open them too."), list);
}

async function levelPage(id) {
  const level = await api(`/api/levels/${id}`);
  const pct = Math.round((level.checks_passed / level.checks_total) * 100);
  const completed = level.status === "completed";

  render(
    h("p", {}, h("a", { href: "#/" }, "← Route")),
    h("section", { class: `hero env-${level.environment}` },
      h("div", { class: "eyebrow" }, `Checkpoint ${level.number} · ${level.environment_label}`),
      h("h1", {}, level.title),
      h("p", {}, level.objective),
    ),
    h("div", { class: "facts" },
      h("div", { class: "card" }, h("strong", {}, "The habit"), level.habit),
      h("div", { class: "card" }, h("strong", {}, "You walk away with"), h("span", { html: level.produces })),
      level.estimated_minutes ? h("div", { class: "card" }, h("strong", {}, "About"), `${level.estimated_minutes} minutes`) : null,
    ),
    completed ? null : startBox(level),
    h("article", { html: level.page_html }),
    h("section", { class: "card" },
      h("h2", { style: "margin-top:0" }, "Progress"),
      h("div", { class: "progress", role: "progressbar", "aria-valuenow": pct, "aria-valuemin": 0, "aria-valuemax": 100 },
        h("span", { style: `width:${pct}%` })),
      h("p", { class: "small muted" }, `${level.checks_passed} of ${level.checks_total} checks passed. Your coach sees one check at a time; so do you.`),
      level.passed_titles.length ? h("ul", { class: "checklist" }, level.passed_titles.map((t) => h("li", {}, t))) : null,
    ),
    completed
      ? h("section", { class: "card" }, h("h2", { style: "margin-top:0" }, "Stamped ✓"),
          level.handoff ? [h("p", { class: "muted small" }, "Your handoff note:"), h("pre", {}, level.handoff)] : null)
      : stampForm(level),
    guidesSection(level),
  );
}

async function ideasPage() {
  const ideas = await api("/api/ideas");
  render(
    h("h1", {}, "Parked ideas"),
    h("p", { class: "muted" }, "Good ideas that belong to a later checkpoint. Your coach parks them so you don't lose them."),
    ideas.length
      ? h("ul", { class: "list-plain card" }, ideas.map((idea) => h("li", {},
          idea.body,
          h("div", { class: "muted small" }, idea.belongs_to_level ? `For ${idea.belongs_to_level}` : "Some day"))))
      : notice("Nothing parked yet."),
  );
}

async function connectPage() {
  const info = await api("/api/connect");
  const result = h("div");
  const label = h("input", { type: "text", id: "label", placeholder: "My laptop" });
  const tokens = h("ul", { class: "list-plain" }, info.tokens.map((t) => h("li", { class: "row" },
    h("span", { class: "grow" }, t.label, h("span", { class: "muted small" },
      ` · created ${new Date(t.created_at * 1000).toLocaleDateString()}${t.last_used_at ? ` · last used ${new Date(t.last_used_at * 1000).toLocaleString()}` : " · never used"}`)),
    h("button", { type: "button", class: "secondary", onclick: async () => {
      if (!confirm(`Disconnect "${t.label}"? Claude Code using it will stop working.`)) return;
      await api(`/api/tokens/${t.id}`, { method: "DELETE" });
      connectPage();
    } }, "Disconnect"))));

  render(
    h("h1", {}, "Connect Claude Code"),
    h("p", { class: "muted" }, "Your coach runs in Claude Code and talks to Grado to see your checkpoint and check your work. Connect it once per computer."),
    h("form", { class: "card stack", onsubmit: async (event) => {
      event.preventDefault();
      const created = await api("/api/tokens", { method: "POST", body: { label: label.value } });
      result.replaceChildren(h("div", { class: "stack" },
        notice("Run this once in a terminal. It's shown only now; if you lose it, create a new one.", "ok"),
        h("pre", { class: "token" }, created.command),
        copyButton(created.command),
        h("p", { class: "small muted" }, "Then open Claude Code in your project folder and type ",
          h("code", {}, `/mcp__${info.server_name}__checkpoint`), "."),
      ));
    } },
      h("div", { class: "row" },
        h("div", { class: "grow" }, h("label", { for: "label" }, "Name this computer"), label),
        h("button", { type: "submit" }, "Create connection command")),
      result,
    ),
    h("h2", {}, "Connected computers"),
    info.tokens.length ? h("div", { class: "card" }, tokens) : notice("None yet."),
  );
}

async function gmPage(playerId) {
  if (!playerId) {
    const players = await api("/api/admin/players");
    render(
      h("h1", {}, "Game master"),
      h("p", { class: "muted" }, "Everyone on the route, and every piece of evidence their coach submitted."),
      h("table", {},
        h("thead", {}, h("tr", {}, h("th", {}, "Player"), h("th", {}, "Stamped"), h("th", {}, "Submissions"), h("th", {}, "Last activity"))),
        h("tbody", {}, players.map((p) => h("tr", {},
          h("td", {}, h("a", { href: `#/gm/${p.id}` }, p.name), h("div", { class: "muted small" }, p.email)),
          h("td", {}, p.levels_completed),
          h("td", {}, p.attempts),
          h("td", {}, p.last_activity ? new Date(p.last_activity * 1000).toLocaleString() : "—"))))),
    );
    return;
  }
  const data = await api(`/api/admin/players/${playerId}`);
  render(
    h("p", {}, h("a", { href: "#/gm" }, "← Players")),
    h("h1", {}, data.user.name),
    h("p", { class: "muted" }, data.journey.map((l) => `${l.number}${l.status === "completed" ? "✓" : l.status === "active" ? "●" : ""}`).join("  ")),
    h("h2", {}, "Submissions"),
    data.attempts.length ? data.attempts.map((a) => h("details", { class: "guide" },
      h("summary", {}, `${a.passed ? "✓" : "✗"} ${a.level_id} / ${a.criterion_id}`,
        h("span", { class: "muted small" }, ` · ${a.grader} · ${new Date(a.created_at * 1000).toLocaleString()}`)),
      h("p", {}, h("strong", {}, "Feedback: "), a.feedback),
      h("pre", {}, a.evidence))) : notice("No submissions yet."),
    h("h2", {}, "Handoff notes"),
    Object.keys(data.handoffs).length
      ? Object.entries(data.handoffs).map(([level, body]) => h("details", { class: "guide" }, h("summary", {}, level), h("pre", {}, body)))
      : notice("None yet."),
    h("h2", {}, "Parked ideas"),
    data.ideas.length ? h("ul", {}, data.ideas.map((i) => h("li", {}, i.body))) : notice("None yet."),
  );
}

// ---------- router ----------

async function route() {
  if (!me) return signInPage();
  const [, page, arg] = location.hash.replace(/^#\/?/, "/").split("/");
  document.querySelectorAll("nav a").forEach((a) => a.classList.toggle("active", a.getAttribute("href") === `#/${page || ""}`));
  try {
    if (page === "level" && arg) await levelPage(arg);
    else if (page === "ideas") await ideasPage();
    else if (page === "connect") await connectPage();
    else if (page === "gm" && me.is_admin) await gmPage(arg);
    else await routePage();
  } catch (error) {
    if (error.status === 401) { me = null; return boot(); }
    render(notice(error.message, "error"), h("p", {}, h("a", { href: "#/" }, "Back to the route")));
  }
}

async function boot() {
  try {
    me = await api("/api/me");
    const connect = await api("/api/connect");
    me.serverName = connect.server_name;
  } catch {
    me = null;
  }
  document.getElementById("nav").hidden = !me;
  document.getElementById("nav-gm").hidden = !me?.is_admin;
  route();
}

document.getElementById("logout").addEventListener("click", async () => {
  await api("/api/auth/logout", { method: "POST" });
  me = null;
  location.hash = "#/";
  boot();
});
window.addEventListener("hashchange", route);
boot();
