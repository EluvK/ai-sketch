# AI Flow Synth

AI workflow orchestration framework for AI tools projects.

## Concepts

The low-level concepts of the framework are primarily referenced from [PocketFlow](https://the-pocket.github.io/PocketFlow/#core-abstraction), a very neat framework for AI workflow orchestration.

- Node handles simple (LLM) tasks.
- Flow connects nodes through Actions (labeled edges).
<!-- - Shared Store enables communication between nodes within flows.
- Batch nodes/flows allow for data-intensive tasks.
- Async nodes/flows allow waiting for asynchronous tasks.
- (Advanced) Parallel nodes/flows handle I/O-bound tasks. -->

## What I want to archive

besides the most basic concepts, the framework also provides many advanced features:

- Middle results: caching / visualization / editing / retrying.
  - User can interfere the result of any nodes in the flow and edit it before passing to the next node.
- Workflow Management: workflow can be edited and saved as a template.
- Tasks Management: task is a runtime for a workflow.
- LLM Management, nodes can use multiple LLMs, if the primary LLM is not available, the node can fallback to other LLMs.
