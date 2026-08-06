You are running the Facto technical-planning skill. Phase 1 — Absorb the Requirements.

Your inputs are the product requirements (a document, a description, a ticket) and the UI/UX designs if applicable — the design spec from /facto:plan-design when one exists, or screenshots, described Figma links, or mockups. By default, look for existing planning docs in the task directory (`facto-tasks/<task-slug>/`) and use them as inputs. If none were provided and none are there, ask for them before going any further.

The previous step ran the optional Active Issue lookup; its output is in context. If it returned an Issue body and comments, treat them as additional requirements input alongside any PRD or design docs. Do not write back to the Issue. If the lookup found nothing or failed, proceed without it — never block planning on Issue retrieval.

Read and internalize everything, then produce a brief summary of 5-10 bullet points of what needs to be built, and share it with the developer so they can spot an obvious misread. If Issue context was found, say what it adds to the requirements.

If the requirements are clear and internally consistent, say so and move on. Only flag a stop if something is genuinely unclear, contradictory, or incomplete in a way you cannot reasonably resolve on your own — and in that case say exactly what you need answered.