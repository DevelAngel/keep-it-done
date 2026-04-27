# Local LLM Server

## Why This Exists

You already have Claude. So why build a small inference box that sits in the corner of your network, drawing 12 watts, running a model a fraction of the size?

Because the two jobs are different.

When you ask Claude to design a system, weigh trade-offs, or write something with care, you want the frontier model. When you say "add laundry to my task list" or "summarise what's open this week," you want speed, privacy, and zero per-token cost. You want a service that's just *there*, like a kitchen tap. You don't think about it. It works.

That's the job this server takes. It's the lightweight backend for the `kid` task management system — handling task creation, mutation, summarisation, and report generation. It's not a replacement for frontier models. It's the cheap, private, offline-capable complement that makes the frontier feel optional for the small stuff.

## Table of Contents

- [Goals](#goals)
- [Hardware](#hardware)
- [Software Stack](#software-stack)
- [Update Strategy](#update-strategy)
- [Constraints](#constraints)

---

## Goals

You want four things from this box, and only these four:

It accepts task-related requests — create, modify, summarise, report. It emits well-formed JSON so it can drive the `kid` CLI directly, no glue code in the middle. It runs 24/7 on low power without phoning home to anyone. And it asks for almost nothing in maintenance.

What it does *not* do: complex reasoning, long-form writing, code generation. Those jobs go upstream to Claude. Trying to make a 3B model do them is like asking a kitchen scale to weigh a truck — possible in theory, painful in practice, wrong tool.

---

## Hardware

The board you pick decides everything downstream — power, performance ceiling, software stack, even which Linux distro you can run. Pick wrong and you're either burning idle watts for years or fighting your own model speed.

You picked the **NVIDIA Jetson Orin Nano 8GB Super Developer Kit**.

| Property   | Value                                   |
| ---------- | --------------------------------------- |
| Memory     | 8 GB unified (shared CPU/GPU)           |
| CUDA cores | 1024                                    |
| TDP        | 7–25 W (Super Mode)                     |
| Storage    | NVMe via M.2 (recommended over microSD) |

Two boards looked tempting on the way to this choice. Neither survived the brief.

**The Raspberry Pi 5.** Lovable hardware. Cheap, well-supported, sips power. But run a 3B model on it and you get 3–6 tokens per second — functional but slow, the kind of slow you notice on every interaction. Add a Hailo-8L AI HAT+ and the throughput problem goes away, except now you're stuck: there's no general LLM pipeline for it today, only fixed-graph compiled models. You'd be locked into whatever someone else has already shipped.

**A desktop GPU.** All the speed you'd ever need. Also ten times the continuous power draw, a tower-sized footprint, and fans you can hear from the next room. Wrong shape for a 24/7 closet appliance.

The Orin Nano sits in the gap. It has CUDA, so the same llama.cpp build that runs on a workstation runs here. It draws less than a lightbulb. It's small enough to forget about.

### The Jetson Orin Family — and Why Bigger Wasn't Better

Here's where it gets interesting.

The Jetson Orin family ships in three classes — Nano, NX, AGX — across seven module variants. They share a common SoC architecture and the same JetPack/CUDA software stack. Code written for one variant transfers cleanly to the others. So when you're picking a tier, you're not picking a software ecosystem. You're picking a power envelope, a memory ceiling, and a price tag.

You'd think more memory and more cores would just be *better*. They are — for the right job. They're not for this one. Let's walk it.

#### Orin Nano — the entry tier

Same SO-DIMM 260-pin form factor across both variants (69.6 × 45 mm). No on-module eMMC; you bring your own NVMe or microSD. Power range 7–25 W.

| Variant            | RAM    | CUDA / Tensor | TOPS (Sparse INT8)         | TDP                  | Preis Deutschland (ca.) |
| ------------------ | ------ | ------------- | -------------------------- | -------------------- | ----------------------- |
| Orin Nano 4GB      | 4 GB   | 512 / 16      | ~20 (legacy) / 34 (Super)  | 7–10 W               | Modul ~€200–280; Super Dev Kit ~€319–349 |
| Orin Nano 8GB      | 8 GB   | 1024 / 32     | 40 (legacy) / 67 (Super)   | 7–15 W (Super: 25 W) | Modul ~€280–340; Super Dev Kit ~€330–399 |

The **Orin Nano Super Developer Kit** — module plus reference carrier — has a $249 NVIDIA list price, but in Germany it lands at €330–399 at retailers like Reichelt, Welectron, Sertronics, and partners listed on Geizhals. The German street price runs noticeably above the US list because of VAT (19 %), distributor margin, and shipping. Welectron and Reichelt are typically at the lower end of that range. If you already own the older Orin Nano Dev Kit, you get Super Mode free via a JetPack 6.x firmware update. Same silicon, raised clocks, raised power envelope. NVIDIA gave existing owners a 1.7× performance bump for the price of a download.

#### Orin NX — the upgrade you can drop in

This is the variant that matters most for future-proofing. Same SO-DIMM 260-pin connector as the Orin Nano. Same carrier board. You unscrew one module, screw in the other, reflash. Done. On-module 16 GB eMMC. Power range 10–40 W in Super Mode.

| Variant      | RAM   | CUDA / Tensor | TOPS (Sparse INT8)            | TDP                  | Preis Deutschland (ca.) |
| ------------ | ----- | ------------- | ----------------------------- | -------------------- | ----------------------- |
| Orin NX 8GB  | 8 GB  | 1024 / 32     | 70 (legacy) / 117 (Super)     | 10–20 W              | Modul ~€570–650         |
| Orin NX 16GB | 16 GB | 1024 / 32     | 100 (legacy) / 157 (Super)    | 10–25 W (Super: 40 W) | Modul ~€790–870; mit Carrier ~€950–1.100 |

The 8GB has a 6-core CPU, the 16GB has 8 cores. Both share the same GPU as the Orin Nano. What you're really paying for here is memory headroom and clock budget — the room to run 7B–8B Q4 models with longer context. Super Mode at 157 TOPS on the 16GB needs JetPack 6.2 or newer and active cooling at sustained 40 W. Don't skip the heatsink.

#### AGX Orin — the top tier

Different form factor (100 × 87 mm), different connector. Not pin-compatible with Nano or NX. Migrating to AGX means redesigning the board. 64 GB on-module eMMC. 204.8 GB/s memory bandwidth — roughly 2× the Orin Nano.

| Variant             | RAM   | CUDA / Tensor | CPU     | TOPS (Sparse INT8) | TDP     | Preis Deutschland (ca.) |
| ------------------- | ----- | ------------- | ------- | ------------------ | ------- | ----------------------- |
| AGX Orin 32GB       | 32 GB | 1792 / 56     | 8-core  | 200                | 15–40 W | Modul ~€1.230–1.350; Dev Kit ~€1.900–3.200 |
| AGX Orin 64GB       | 64 GB | 2048 / 64     | 12-core | 275                | 15–60 W | Modul ~€1.500–1.800; Dev Kit ~€2.270–3.200 |
| AGX Orin Industrial | 64 GB | 2048 / 64     | 12-core | 248                | 15–75 W | Modul ~€2.000–2.500 (nur über Distributoren); ECC RAM, –40 °C bis +85 °C |

The price spread on Dev Kits is wide because some listings are bare boards while others bundle SSD, Wi-Fi card, heatsink, and power supply. Antratek's H01 Kit at €1.902 sits at the bare end; MyBotShop at €3.194 includes a more complete bundle.

The AGX 32GB and 64GB aren't just memory variants — the 64GB has a faster GPU clock (1.3 GHz vs 930 MHz), faster DLA, and 12 CPU cores instead of 8. The Industrial variant trades a bit of peak performance (248 vs 275 TOPS) for ECC memory and a –40 °C to +85 °C operating range. The AGX Dev Kit can also emulate any other Orin module via reflashing, which is genuinely useful if you're prototyping across the family.

#### What this means for the project

Now the picking gets easy.

The model you're running is Qwen 2.5-3B at Q4_K_M, weighing roughly 2 GB in VRAM, with a 4K context window. That fits on any 8 GB+ variant with room left over. You don't need 64 GB of memory to summarise a task list. You don't need 275 TOPS to emit JSON.

This means the **Orin Nano 8GB Super** is the right pick — chosen, ~€330–399 in Germany, lowest idle power, sufficient headroom. This means the **Orin NX 16GB** is your upgrade lane if the workload ever grows: drop-in, no carrier redesign, room for 7B–8B Q4 models with 8K–16K context, around €790–870 for the bare module. This means the **AGX Orin** tier is overkill — justified only if you're running parallel vision pipelines, multiple concurrent LLMs, or unquantised 13B+ models. For everything else, you'd be paying €2.000+ to leave most of the silicon idle.

The numbers are precise on purpose. Idle power scales with the TDP envelope. Acquisition cost scales faster than performance for this workload. For a box that runs 24/7 at low utilisation, the Nano tier wins on every axis that matters.

(Module prices fluctuate by region and distributor. Figures above are German street prices including VAT, excluding shipping, as of April 2026, drawn from Reichelt, Welectron, MyBotShop, Antratek, Sertronics, and Geizhals listings.)

---

## Models per Tier

The hardware decides which model can fit. The model decides whether the box is actually useful. Pick a model too small and your JSON comes back broken half the time; pick one too large and the box swaps, throttles, or refuses to load it.

For `kid`, the requirements are narrow on purpose. You need solid instruction following, reliable JSON output, and ideally native function calling. You don't need code generation, deep reasoning, or world knowledge — that work goes upstream to Claude. So the right model is the smallest one that hits structured output reliably, with the rest of the VRAM spent on a generous KV cache.

A note on memory math: VRAM usage roughly equals model file size plus the KV cache (which scales with context length). For Q4_K_M quants, that's about 0.5–0.6 GB per gigabyte of model weights for a 4K context, more for longer windows. Subtract OS overhead (~1–2 GB on Jetson) and you get the usable budget.

Recommendations below assume Q4_K_M (the GGUF default — best quality-per-byte ratio) and a 4K–8K context unless noted.

### Orin Nano 4GB

You have roughly 2–2.5 GB of usable VRAM. That's tight. You're picking from the very small models, and you'll feel it on anything beyond simple structured tasks.

| Model                         | Quant   | Approx. size | Notes                                              |
| ----------------------------- | ------- | ------------ | -------------------------------------------------- |
| Llama 3.2 1B Instruct         | Q4_K_M  | ~0.7 GB      | Native tool calling. Fast. Quality ceiling is real. |
| Qwen 2.5 1.5B Instruct        | Q4_K_M  | ~1.0 GB      | Strong instruction following for size.             |
| SmolLM3 1.7B                  | Q4_K_M  | ~1.1 GB      | Open recipe, dual-mode reasoning.                  |
| Qwen 2.5 3B Instruct          | Q3_K_M  | ~1.5 GB      | Aggressive quant, marginal quality.                |

Honest take: the 4GB tier is fine for proof-of-concept and very simple JSON, but `kid`'s task-creation prompts will start failing at the edges. If you have any choice, skip this tier.

### Orin Nano 8GB / Orin NX 8GB

About 5–6 GB of usable VRAM. This is the sweet spot for 3B–4B models, and where `kid` actually wants to live. All four picks below run comfortably with room left for KV cache.

| Model                         | Quant   | Approx. size | Notes                                              |
| ----------------------------- | ------- | ------------ | -------------------------------------------------- |
| **Qwen 2.5 3B Instruct**      | Q4_K_M  | ~2.0 GB      | **Current pick.** Native tool calling. Apache 2.0. Reliable JSON. |
| **SmolLM3 3B**                | Q4_K_M  | ~2.0 GB      | Beats Qwen 2.5 3B on most benchmarks. 64K context, /think mode. Apache 2.0. |
| **Ministral 3 3B Instruct** (2512) | Q4_K_M | ~2.2 GB | Mistral, December 2025. Designed for function calling and JSON. 256K context. |
| **Phi-4-mini 3.8B**           | Q4_K_M  | ~2.5 GB      | Microsoft, MIT license. Strong reasoning. 128K context. Less factual knowledge than Qwen. |
| Hermes 2 Pro – Mistral 7B     | Q4_K_M  | ~4.5 GB      | 91 % function-calling accuracy, 84 % JSON-mode. Tight fit. Slower. |

The current choice (Qwen 2.5 3B) is a safe baseline, but two newer options deserve a serious look. **SmolLM3** is the open-recipe upstart that benchmarks above Qwen 2.5 3B at the same size, with a longer context and a switchable reasoning mode you can toggle per request. **Ministral 3** is the model Mistral built explicitly for this kind of edge-agent workload — function calling and JSON aren't an afterthought, they're the design target.

For pure function-calling reliability, **Hermes 2 Pro** still leads its size class, but at 4.5 GB it eats most of the 8 GB box and leaves little for context. Worth it only if structured output is your single hard requirement.

### Orin NX 16GB

About 12–14 GB of usable VRAM. This is where you stop quantising aggressively and start running 7B–8B models at higher precision, or step up to 14B at Q4.

| Model                              | Quant   | Approx. size | Notes                                              |
| ---------------------------------- | ------- | ------------ | -------------------------------------------------- |
| Qwen 2.5 7B Instruct               | Q5_K_M  | ~5.0 GB      | Strong general-purpose, native tool calling.       |
| **Hermes 2 Pro – Mistral 7B**      | Q8_0    | ~7.5 GB      | Best-in-class function calling and JSON mode.      |
| Llama 3.1 8B Instruct              | Q5_K_M  | ~5.7 GB      | Native tool calling, broad ecosystem support.      |
| Qwen 3 8B                          | Q5_K_M  | ~5.8 GB      | Newer generation, dual-mode reasoning.             |
| Phi-4 14B                          | Q4_K_M  | ~8.5 GB      | Stronger reasoning. Tight on KV cache budget.      |

For `kid`, **Hermes 2 Pro 7B at Q8** is the sweet pick — full-precision quants of the model with the highest documented function-calling accuracy in this size class. If you'd rather stay on the Qwen line for consistency with the 8GB tier, Qwen 2.5 7B at Q5_K_M is a clean upgrade.

### AGX Orin 32GB

About 28 GB of usable VRAM. You can now run 24B–32B dense models at Q4–Q5, or MoE models that punch well above their active-parameter count.

| Model                              | Quant   | Approx. size | Notes                                              |
| ---------------------------------- | ------- | ------------ | -------------------------------------------------- |
| **Qwen 3 30B-A3B (MoE)**           | Q4_K_M  | ~18 GB       | Only 3B active params at inference — very fast on edge. |
| Mistral Small 3.1 24B              | Q5_K_M  | ~17 GB       | Strong instruction following. Apache 2.0.          |
| Gemma 3 27B                        | Q4_K_M  | ~16 GB       | Multimodal (4B+ variants).                         |
| Phi-4 14B                          | Q8_0    | ~16 GB       | Full-precision reasoning.                          |
| DeepSeek-R1-Distill-Qwen 32B       | Q4_K_M  | ~19 GB       | Reasoning-tuned. Overkill for `kid`.               |

**Qwen 3 30B-A3B** is the standout here. The MoE architecture means you load 30B of weights but only 3B are active per token, so generation speed feels like a 3B model with the quality of something much larger. For an always-on box, that's a genuinely good trade.

### AGX Orin 64GB

About 58 GB of usable VRAM. You can now run 70B-class models — the tier that competes with frontier APIs from a year ago.

| Model                              | Quant   | Approx. size | Notes                                              |
| ---------------------------------- | ------- | ------------ | -------------------------------------------------- |
| Llama 3.3 70B Instruct             | Q4_K_M  | ~46 GB       | Native tool calling. Strong general-purpose.       |
| Qwen 2.5 72B Instruct              | Q4_K_M  | ~50 GB       | Apache 2.0. Top open-weights generalist.           |
| **Hermes 4.3 36B**                 | Q6_K    | ~30 GB       | NousResearch's flagship for function calling. 512K context. |
| DeepSeek-R1-Distill-Llama 70B      | Q4_K_M  | ~45 GB       | Reasoning specialist.                              |
| Qwen 3 32B                         | Q8_0    | ~34 GB       | Full-precision Qwen 3 with generous KV cache.      |

For `kid` specifically, this tier is dramatic overkill — you'd be running a 70B model to draft a shopping list. The honest recommendation: if you're on a 64GB box, you're using it for more than `kid`, and the model choice should follow your other workloads. Hermes 4.3 36B is the function-calling specialist worth knowing about regardless.

### Summary

| Tier              | Recommended pick for `kid`        | Why                                              |
| ----------------- | --------------------------------- | ------------------------------------------------ |
| Orin Nano 4GB     | Qwen 2.5 1.5B Instruct (Q4)       | Smallest viable. Quality is marginal.            |
| Orin Nano 8GB     | **Qwen 2.5 3B** or **SmolLM3 3B** | Right size. Solid JSON. Current baseline.        |
| Orin NX 8GB       | Same as above                     | Pin-compatible upgrade from Nano. More headroom. |
| Orin NX 16GB      | **Hermes 2 Pro 7B (Q8)**          | Best function calling in class. Full precision.  |
| AGX Orin 32GB     | **Qwen 3 30B-A3B (MoE)**          | MoE speed + 30B-class quality. Overkill but fast.|
| AGX Orin 64GB     | Workload-dependent                | `kid` doesn't need this much. Pick by other use. |

---

## Software Stack

### Inference Engine

**llama.cpp** — compiled with CUDA support for Jetson (aarch64 + CUDA 12).

It runs as an HTTP server on port 8080. The model is `Qwen2.5-3B-Instruct` at Q4_K_M quantisation, occupying around 2 GB of VRAM. Context window: 4K tokens. That sounds tight if you're used to frontier models, but every task operation in `kid` fits comfortably inside it — you're never asking the model to reason over a novel.

### Optional UI

**Open WebUI** — a browser-based chat interface that talks to llama.cpp's OpenAI-compatible endpoint.

Useful for ad-hoc queries and prompt testing. Not required for the automated `kid` integration. Think of it as the diagnostic port — you don't need it day-to-day, but the day you do need it, you'll be glad it's already wired up.

### Base OS

**Ubuntu 22.04 LTS + JetPack 6.**

This isn't a preference. It's the only distribution with full CUDA driver support for Orin today. You might wish for Fedora IoT — the auto-update story is cleaner, the immutable filesystem is appealing — but as of April 2026, GPU and AI acceleration on Orin under Fedora simply don't work. You'd be picking aesthetics over functionality.

### Deployment

You run both components as **Podman Quadlets** — systemd-native container units. No Docker daemon, no separate compose tool, no extra orchestration layer. Each container is a first-class systemd service, with all the boot ordering, restart policies, and journal integration that comes free.

#### One-time prerequisites

Before any container can see the GPU, the host needs the NVIDIA Container Toolkit and a generated CDI (Container Device Interface) spec. NVIDIA Container Toolkit has supported Jetson since version 1.7.0 on Ubuntu 18.04 / 20.04 / 22.04, so the standard install steps work as documented.

```bash
# Install NVIDIA Container Toolkit (per NVIDIA's standard guide)
sudo apt install -y nvidia-container-toolkit

# Generate the CDI spec — the contract Podman uses to expose the GPU
sudo nvidia-ctk cdi generate --output=/etc/cdi/nvidia.yaml

# Verify
nvidia-ctk cdi list
# Expected: nvidia.com/gpu=all (and per-device entries)
```

You regenerate the CDI spec only when the driver changes — typically as part of a JetPack upgrade. Never on a daily basis.

#### Quadlet files

Drop these in `/etc/containers/systemd/`. systemd picks them up on `daemon-reload` and creates matching `.service` units.

`llama.container`:

```ini
[Unit]
Description=llama.cpp inference server
After=network-online.target
Wants=network-online.target

[Container]
ContainerName=llama
Image=ghcr.io/ggerganov/llama.cpp:server-cuda
AutoUpdate=registry
AddDevice=nvidia.com/gpu=all
SecurityLabelDisable=true
PublishPort=8080:8080
Volume=/opt/llama/models:/models:Z
Exec=--model /models/qwen2.5-3b-q4_k_m.gguf --port 8080 --host 0.0.0.0

[Service]
Restart=always
RestartSec=10

[Install]
WantedBy=default.target
```

`open-webui.container`:

```ini
[Unit]
Description=Open WebUI
After=llama.service
Wants=llama.service

[Container]
ContainerName=open-webui
Image=ghcr.io/open-webui/open-webui:latest
AutoUpdate=registry
PublishPort=3001:8080
Volume=/opt/open-webui/data:/app/backend/data:Z
Environment=OPENAI_API_BASE_URL=http://llama:8080/v1

[Service]
Restart=always
RestartSec=10

[Install]
WantedBy=default.target
```

Activate them:

```bash
sudo systemctl daemon-reload
sudo systemctl start llama.service open-webui.service
sudo systemctl enable --now podman-auto-update.timer
```

That last line is the whole update story — `podman-auto-update.timer` runs daily by default, pulls new images for any container marked `AutoUpdate=registry`, and rolls back automatically if the new image fails to start.

Two files. One reload. Nothing to babysit.

---

## Update Strategy

The OS and the application layer are decoupled on purpose.

| Layer         | Update mechanism                                                       |
| ------------- | ---------------------------------------------------------------------- |
| OS (Ubuntu)   | `unattended-upgrades` — security patches only, automatic               |
| llama.cpp     | `podman-auto-update.timer` — daily check, pull-and-restart on new image, automatic rollback on failure |
| Open WebUI    | same as above                                                          |
| CDI / GPU     | `nvidia-ctk cdi generate` after a JetPack upgrade — manual but rare    |
| Model weights | manual; only when a better model warrants replacement                  |

This is the original Fedora IoT goal — low-touch, automatic, safe — achieved without giving up GPU support. The OS patches itself. The containers update themselves. The model only changes when *you* decide it should change. Three tiers of update cadence, each matched to how often the underlying thing actually moves.

---

## Constraints

A few things to be honest about before you commit:

CUDA acceleration depends on NVIDIA's proprietary drivers. Pick a different OS to escape them and you lose the acceleration that justifies the hardware in the first place. The Jetson Orin Nano is not a drop-in replacement for a general-purpose Linux box. Initial setup needs the JetPack SDK and the `sdk-manager` tooling — plan for an afternoon of flashing and configuration, not a fifteen-minute install. And the model has a quality ceiling. 3B models handle structured tasks well, but nuanced natural language understanding degrades visibly compared to frontier models. Keep the complex prompts for Claude. This box is for the small, repetitive, structured work that doesn't need a brain the size of a planet.

That last point isn't a flaw. It's the design.

---

## Sources

Specifications, TOPS figures, power envelopes, and prices in the variants section drawn from:

- NVIDIA — *Jetson AGX Orin product page* (`nvidia.com/en-us/autonomous-machines/embedded-systems/jetson-orin/`)
- NVIDIA — *Buy the Latest Jetson Products* (`developer.nvidia.com/buy-jetson`)
- NVIDIA — *Jetson Modules, Support, Ecosystem, and Lineup* (`developer.nvidia.com/embedded/jetson-modules`)
- NVIDIA — *Jetson Orin Nano Super Developer Kit* product page
- NVIDIA — *Jetson AGX Orin Series Technical Brief v1.2* (PDF, July 2022)
- *Jetson AGX Orin Module Series Datasheet* (openzeka.com mirror)
- NVIDIA Blog — *NVIDIA Unveils Its Most Affordable Generative AI Supercomputer* (Dec 2024, Super Mode launch and pricing)
- ThinkRobotics — *Jetson AGX Orin 64GB Developer Kit Review* (Mar 2026, dense-vs-sparse TOPS clarification)
- ThinkRobotics — *Jetson Orin NX Module Review 2025* (Super Mode 157 TOPS conditions)
- DesignSpark / RS — *Jetson AGX Orin & Orin Nano: Features and Specifications*
- Hackster.io — *Hands-on Review of the AGX Orin Developer Kit* (32GB vs 64GB hardware deltas)
- e-con Systems — *NVIDIA Jetson Orin vs other Jetson modules*
- Geizhals.de price aggregator (German street prices including VAT, April 2026 snapshot)
- Reichelt, Welectron, MyBotShop, Antratek, Sertronics, Kubii, Silicon Highway, Arrow Germany (German distributor listings for module / dev-kit pricing)
- heise — *Halber Preis, mehr TOPS: Neues Nvidia Jetson Orin Nano Developer Kit* (Dec 2024, €249 list price announcement)

Model recommendations and benchmarks drawn from:

- BentoML — *The Best Open-Source Small Language Models in 2026* (SmolLM3, Phi-4-mini, Ministral 3 profiles)
- ML Journey — *Best Open-Source LLMs Under 7B Parameters (Run Locally in 2026)*
- Local AI Master — *Best Small AI Models to Run with Ollama (2026)*
- Awesome Agents — *Home GPU LLM Leaderboard: Open-Source Models by VRAM Tier* (Qwen 3 30B-A3B MoE benchmarks)
- Onyx AI — *Best Self-Hosted LLM Leaderboard 2026*
- imaurer/awesome-llm-json (GitHub) — Hermes 2 Pro function-calling and JSON-mode accuracy figures
- Oflight — *NousResearch Hermes Complete Guide — Hermes 4.3 36B, Function Calling & Hermes Agent (2026)*
- LM Studio model directory — Ministral 3 3B specifications
- Hermes Agent (Nous Research) docs — native tool-calling support matrix for llama.cpp
- NVIDIA Jetson AI Lab — *Models* compatibility matrix for Jetson Orin variants

Podman Quadlets and Jetson GPU integration drawn from:

- NVIDIA — *NVIDIA Container Toolkit Installation Guide* (Jetson support since v1.7.0 on Ubuntu 18/20/22)
- NVIDIA — *Support for Container Device Interface* (CDI generation, podman `--device nvidia.com/gpu=all`)
- Brandon Rozek — *Setting up Ollama with CUDA on Podman Quadlets* (Nov 2025; Quadlet `.container` reference)
- OneUptime — *How to Run NVIDIA GPU Containers with Podman* (March 2026, CDI workflow)
- Podman Desktop — *GPU container access* documentation
- `podman-systemd.unit(5)` — Quadlet directives reference (`AutoUpdate`, `AddDevice`, `PublishPort`)
- `podman-auto-update(1)` — daily image refresh and rollback semantics
