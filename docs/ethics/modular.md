**STRICT MODULAR DESIGN** — Everything must follow a modular architecture. Every feature, component, service, utility, and responsibility must belong to a clearly defined module with a single purpose.

- **Separation of concerns:** Each module must have one clear responsibility. Do not mix unrelated logic.
- **Small, focused modules:** Avoid monolithic files, oversized classes, and components that handle multiple responsibilities.
- **Clear boundaries:** Modules must expose well-defined interfaces and hide their internal implementation details.
- **Loose coupling:** Modules should depend on abstractions or public interfaces rather than directly accessing another module's internals.
- **Reusability:** Extract shared logic into dedicated modules instead of duplicating code.
- **Independent testing:** Each module must be testable in isolation, with dependencies properly controlled.
- **No unnecessary modules:** Do not split code into tiny modules without a clear benefit. Modularity must improve maintainability, not create needless complexity.
- **Before creating or modifying code:** Inspect the existing architecture, identify the appropriate module, and follow established patterns. Do not introduce a parallel structure or bypass existing module boundaries.

**Non-negotiable:** Do not implement features as monolithic blocks or allow responsibilities to accumulate in a single module. If a module becomes too large or handles unrelated concerns, refactor it into smaller, cohesive modules. All architectural decisions must preserve clear boundaries, maintainability, testability, and the ability to extend the system without breaking unrelated modules.
