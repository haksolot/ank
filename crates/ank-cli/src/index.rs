//! Derived SQLite index, disposable, never the source of truth.
//!
//! Implemented by TASK-b2c3d4e5f6a7. Dispatch routes to no index: this is a
//! cache the verbs consult, not a verb. `context` and `find` are its first
//! callers, and each arrives with its own task.
