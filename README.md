# VivaTeach Planner 🦾📅

👉 **Live demo:** <https://vivatech.remo-lab.com/>

📧 **Feedback?** Please share your thoughts at **vivatech@remolab.fr**

> Navigating VivaTech's schedule can feel like decoding a circuit board with **no labels**. I needed something faster, smarter, and more tailored—so I built **VivaTeach Planner**. I was literally _cooking dinner while cooking up this code_ because I wanted a planner that **thinks with me**.

---

## 🎯 What It Does

✅ Returns **up to 10 tailored session picks per day** based on your interests.  
🧭 Surfaces **high-value talks and booths** in seconds.  
💬 This is **alpha**—DM me for more recs; your feedback shapes what comes next!

> **Heads-up:** Because this is part of my *learning-in-public* journey, the service runs with **extremely verbose logging** (every request, tool call, and agent step). If that's too noisy, just set the `RUST_LOG` env var to something like `info` or `warn` when you run the service to quiet things down.

---

## 🛠️  Stack Behind the Scenes

| Layer                 | Tech / Service | Why it was chosen |
|-----------------------|----------------|-------------------|
| **Runtime Intelligence** | OpenAI **GPT-4o** | Fast, reliable, and robust for real-time reasoning |
| **Agentic Logic**     | [**Rig**](https://github.com/rigprompt/rig) | Lightweight, modular framework for orchestrating tool-calling LLM agents |
| **Web / API**         | **Rust** + [**Axum**](https://github.com/tokio-rs/axum) | Performant, type-safe HTTP layer |
| **Serverless Deploy** | [**Shuttle**](https://www.shuttle.rs/) | Zero-config, Rust-native deployments |
| **Database**          | [**Neon**](https://neon.tech) (Postgres) | Serverless with branching for safe experimentation |
| **Authentication**    | **Supabase** | Quick auth flows & session handling |
| **Embeddings**        | `mxbai-embed-large-v1` via **Ollama** (local) → Hugging Face (cloud) | Cheap to build locally, scalable remotely |
| **Debugging Copilot** | **Claude 4** | Sharp and contextual sanity checks |

---

## 🖼️  High-Level Architecture

```mermaid
flowchart TD
    subgraph Client
        A[POST /generate-plan] -->|objective JSON| B
    end

    subgraph Backend (Axum + Rig)
        B[generate_plan_handler]
        B --> C[Rig Agent (GPT-4o)]
        C --tool: query_vivatech_api--> D[External VivaTech RAG API]
        C --(optional tool)--> E[assess_event_timeliness]
        C --> F[Action Plan]
    end

    D -->|JSON sources| C
    E -->|Urgency metadata| C
    F -->|string/JSON| G[HTTP Response]
```

---

## 📂 Project Layout

```
vivaagent-opensource/
├─ src/
│  ├─ main.rs          # 🚪 Axum service entry-point & Shuttle glue
│  ├─ tools.rs         # 🛠️  Rig tool implementations
│  └─ models.rs        # 🗂️  Domain structs & helper fns
├─ tests/              # ✅ (coming soon) integration & unit tests
└─ Cargo.toml          # 📦 Rust dependencies & metadata
```

---

## ⚡ Quick Start (Local)

### Prerequisites

* **Rust** >= 1.79 (`rustup default stable`)
* **cargo-shuttle** CLI → `cargo install shuttle-launcher`
* An **OpenAI API key** with GPT-4 access
* (Optional) [**Ollama**](https://ollama.ai/) if you want to run embeddings locally

### 1 · Clone & enter

```bash
git clone https://github.com/<you>/vivaagent-opensource.git
cd vivaagent-opensource
```

### 2 · Configure secrets

Create a **`Secrets.toml`** (used by Shuttle) _or_ export env vars directly.

```toml
# Secrets.toml
OPENAI_API_KEY   = "sk-..."
VIVATECH_API_URL = "https://vivatech-rag-v2-n1hk.shuttle.app/query"
# Optional fine-tuning
API_TIMEOUT_SECONDS = "30"
CONFERENCE_DATE     = "2025-06-11"
```

### 3 · Run locally

```bash
cargo shuttle run        # spins up http://localhost:8000
```

---

## 🔌  API Usage

### Endpoint

`POST /generate-plan`

### Request Payload

```json
{
  "objective": "Find AI sessions about climate tech on Friday"
}
```

### Example Response

```text
1. 🌱 **AI for a Greener Planet** — Friday 10:00, Stage 3  
   Why attend: Top researchers share carbon-negative ML techniques.

2. 🤖 **Robotics in Sustainability** — Friday 13:30, Hall B  
   Why attend: Live demo of autonomous waste-sorting bots.

… up to 10 picks …
```

---

## 🧩  Internals

### Key Files

* **`src/main.rs`** – Axum route `/generate-plan`, sets up the Rig agent and forwards the user objective.
* **`src/tools.rs`** – Implements two Rig tools:
  * `query_vivatech_api` → Hits the external RAG endpoint to search sessions/partners.
  * `assess_event_timeliness` → Parses dates & classifies urgency (Immediate / Soon / Normal).
* **`src/models.rs`** – Domain models (`GeneratePlanRequest`, `VivatechSource`, etc.).

### Env Vars Used

| Variable               | Required | Purpose                           |
|------------------------|----------|-----------------------------------|
| `OPENAI_API_KEY`       | ✅       | Calls GPT-4o for planning logic    |
| `VIVATECH_API_URL`     | ✅       | Endpoint for VivaTech RAG search  |
| `API_TIMEOUT_SECONDS`  | ❌       | HTTP timeout for external calls   |
| `CONFERENCE_DATE`      | ❌       | Override reference date for tools |

---

## 🚀 Deploying to Shuttle

```bash
# 1. Log in / sign up
cargo shuttle login

# 2. Provision an app
cargo shuttle init --name viva-teach-planner

# 3. Deploy
cargo shuttle deploy
```

Shuttle automatically provisions a Postgres instance if you need one later. Add your secrets via the dashboard or `Shuttle.toml`.

---

## 🧭  Roadmap -> <!-- Roadmap section removed -->

## 📝 License

MIT © 2025 — Happy hacking & see you at VivaTech!

---

## 🌐  Companion Services

This repo focuses on the **planning micro-service**, but two other services complete the picture:

| Service | Purpose | Env Var / URL |
|---------|---------|---------------|
| **VivaTech RAG Agent** | Exposes a `/query` endpoint that talks **directly to the Postgres + embeddings DB**. Our `query_vivatech_api` tool calls this private service to fetch relevant sessions & partner info. | `VIVATECH_API_URL` (e.g. `https://vivatechblablabla.shuttle.app/query`) (private) |
| **Session Manager / Queue** | A minimal **private** front-end layer that handles **session token issuance**, **rate-limiting**, and a small in-memory **FIFO queue** so we don't overload GPT-4. Think "traffic cop" for end-users. | Live at <https://vivatech.remo-lab.com/> (private) |

> The front-end session manager is available publicly at **https://vivatech.remo-lab.com/**. It is intentionally kept thin—HTML + a sprinkle of JS—so it can be swapped or scaled independently (Cloudflare Workers, Vercel Edge, etc.). For now it simply meters requests and shows a "you're in line" screen if usage spikes.

If you'd like to run these companion services locally you'll find them in their own repositories:

* [`vivatech-rag-service`](https://github.com/<you>/vivatech-rag-service) – Rust, Shuttle, Neon
* [`vivatech-session-manager`](https://github.com/<you>/vivatech-session-manager) – TypeScript, Bun, Cloudflare KV

---
