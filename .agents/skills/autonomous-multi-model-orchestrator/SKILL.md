---
name: autonomous-multi-model-orchestrator
description: >-
  Autonomous multi-model orchestration system combining Gemini Web (Neon High / Frontier
  Architect), Google AI Studio (Code Generator), and Antigravity (Flash High / Local Executor).
  Use this skill whenever local model tier restrictions exist (e.g. Jio-bundled accounts), when
  tackling large-scale architecture tasks requiring frontier reasoning without API costs, or
  when orchestrating heterogeneous models via browser automation and local CLI execution.
---

# Autonomous Multi-Model Orchestrator

This skill defines the operational protocol for running a three-tier, multi-model autonomous development team using browser automation (`/browser`) and Antigravity's local toolchain.

```
                    +------------------------------------------+
                    |    GEMINI WEB (Gemini 4 Neon High)       |
                    |             THE ORCHESTRATOR             |
                    |  * High-level reasoning & architecture   |
                    |  * DAG task decomposition & sequencing   |
                    |  * Milestone review & error diagnostics  |
                    +--------------------+---------------------+
                                         ^
                                         | [1. High-level Architecture & Plan]
                                         v
+-----------------------------------------------------------------------------------+
|                        ANTIGRAVITY (3.8 Flash High)                               |
|                              THE EXECUTOR                                         |
|  * Orchestrates the browser automation bridge (/browser)                          |
|  * Dispatches subtask specs to AI Studio & extracts code                          |
|  * Writes source code, runs build gates (cargo test, clippy, fmt)                 |
|  * Relays compiler errors back for self-healing & updates Neon on progress        |
+----------------------------------------+------------------------------------------+
                                         ^
                                         | [2. Implementation Specs & Raw Code]
                                         v
                    +--------------------+---------------------+
                    |       GOOGLE AI STUDIO (Gemini Pro)      |
                    |            THE CODE GENERATOR            |
                    |  * Pure code synthesis & heavy typing    |
                    |  * Zero boilerplate omissions            |
                    |  * High-token algorithmic implementations|
                    +------------------------------------------+
```

---

## 1. Role Definitions & Responsibilities

| Role | Platform / Interface | Model Tier | Core Responsibility |
| :--- | :--- | :--- | :--- |
| **The Orchestrator** | `gemini.google.com` (Gemini Advanced) | **Gemini 4 Neon (Thinking: High)** | System architecture, dependency trees, module interfaces, edge-case analysis, and strategic debugging. |
| **The Code Generator** | `aistudio.google.com` | **Gemini 2.5/Frontier Pro** | Full-file code generation, algorithmic logic, type-safe implementations, and zero-shortcut code synthesis. |
| **The Executor** | **Antigravity CLI / IDE** | **Gemini 3.8 Flash (Thinking: High)** | Browser navigation, DOM scraping/injection, file I/O, compilation, linting, test suites, and status reporting. |

---

## 2. Activation Triggers

Activate this skill when:
- The user mentions "multi-model orchestration", "browser orchestration", "Neon + AI Studio + Flash", or "bypass model restrictions".
- The user is operating under account-level tier locks (e.g., Jio promotional bundles lacking direct Neon/Hydrogen access inside Antigravity).
- A task demands frontier-level architectural reasoning (systems architecture, distributed protocols, complex algorithms) while preserving local execution speed.

---

## 3. End-to-End Orchestration Workflow

### Phase 1: Browser Environment Preparation
1. **Initialize Browser Session**:
   - Open `/browser` with an existing user data directory to retain authentication cookies for Google AI Studio and Gemini Web.
   - Verify active sessions on:
     - Tab 0: `https://gemini.google.com/app`
     - Tab 1: `https://aistudio.google.com/prompts/new_chat`
2. **Configure Model Settings**:
   - **Gemini Web**: Ensure Gemini Advanced is active with thinking effort set to **High**.
   - **AI Studio**: Select the highest-tier reasoning/coding model, set system instructions to "Senior Systems Engineer - Return pure production code with complete implementations", and disable safety filtering if working on systems FFI.

---

### Phase 2: The Orchestration Handshake (Architectural Plan)

1. **Submit Architectural Briefing to Gemini Web (Neon High)**:
   Deliver the user task wrapped in the **Orchestrator Protocol Wrapper**:
   ```markdown
   [ROLE: ORCHESTRATOR]
   You are the lead software architect. I am your autonomous local executor (Flash High).
   I will write files, compile, and run tests locally.
   
   Project Goal: <USER_GOAL>
   Constraints:
   - Modular workspace architecture (max 500 lines per file).
   - Zero clippy warnings, strict formatting, comprehensive test coverage.
   
   Output Required:
   1. System Architecture & Component Dependency Graph (DAG).
   2. Numbered Implementation Steps.
   3. For Step 1, provide a self-contained "Code Generator Specification" that I will pass directly to Google AI Studio.
   ```

2. **Retrieve Plan & First Spec**:
   - Wait for Gemini Web generation to complete.
   - Extract the full response text.
   - Parse the structured specification for Step 1.

---

### Phase 3: The Code Generation Step (AI Studio)

1. **Dispatch to Google AI Studio**:
   - Switch to Tab 1 (`aistudio.google.com`).
   - Inject the specification extracted from Gemini Web:
   ```markdown
   [ROLE: CODE GENERATOR]
   Implement the following module specification completely.
   Requirements:
   - Provide the complete, unabbreviated code. Do not use placeholders (e.g. '// TODO', '// existing code').
   - Wrap the file in explicit XML path tags:
     <FILE path="path/to/module.rs">
     ... code ...
     </FILE>
   
   Specification from Architect:
   <INJECTED_SPECIFICATION>
   ```
2. **Extract Synthesized Code**:
   - Wait for streaming completion.
   - Copy or extract the content inside `<FILE path="...">` blocks.

---

### Phase 4: Local Assembly & Verification (Antigravity Flash)

1. **Write Files**:
   - Write the generated files to the target workspace paths using `write_file` or `edit_file`.
2. **Execute Mandatory Verification Gates**:
   - **Formatting**: `cargo fmt --all -- --check`
   - **Static Analysis**: `cargo clippy --all-targets --all-features`
   - **Test Execution**: `cargo test -j 2`

---

### Phase 5: The Feedback Loop (Self-Healing or Progression)

#### Case A: Compilation / Test Failure
1. Capture stdout/stderr compiler errors and backtraces.
2. Format a **Remediation Packet**:
   ```markdown
   [STATUS: VERIFICATION FAILED]
   File: <FILE_PATH>
   Error Output:
   ```
   <COMPILER_OR_TEST_OUTPUT>
   ```
   Provide the corrected full file implementation.
   ```
3. Send to **AI Studio** for quick algorithmic fixes, or to **Gemini Web** if the failure indicates an architectural flaw.
4. Apply the fix and re-run verification.

#### Case B: Verification Success
1. Switch back to **Gemini Web** (Tab 0).
2. Report success:
   ```markdown
   [STATUS: STEP <X> COMPLETED & VERIFIED]
   - Built: <FILES_CREATED>
   - Tests Passed: <TEST_SUMMARY>
   Please evaluate milestone and provide the Code Generator Specification for Step <X+1>.
   ```
3. Repeat until all steps in the DAG are completed.

---

## 4. Failure Modes & Mitigation Strategies

| Failure Mode | Root Cause | Mitigation Strategy |
| :--- | :--- | :--- |
| **DOM Selector Drift** | Google UI updates breaking automated click/type selectors | Rely on semantic accessibility selectors or visual canvas coordinate mapping rather than brittle CSS classes. |
| **Session / Auth Expiry** | Google re-authenticating or demanding CAPTCHA | Maintain persistent browser context (`--user-data-dir`); prompt user for manual sign-in if an explicit auth gate appears. |
| **Token Truncation** | Large code modules exceeding web chat output windows | Have the Orchestrator split modules into submodules strictly under 300–400 lines before sending to AI Studio. |
| **Hallucinated Diffs** | AI Studio omitting unchanged methods with comments | Enforce the `<FILE path="...">` rule requiring full, valid file contents on every generation. |

---

## 5. Security & Safety Rules

- **Zero Credential Leaks**: Never paste API keys, private tokens, or user secrets into public or web-based prompts.
- **Strict Sandboxing**: Always review generated code before executing raw binary scripts outside standard build harnesses (`cargo test`).
- **Preserve Documentation**: All existing docstrings, copyright headers, and architectural invariants must be preserved during module replacement.
