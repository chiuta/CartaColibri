//! CartaColibri — a 100% local AI library. MIT licensed, clean-room.
//!
//! * [`library`] — RAG-fetch: incremental SQLite-FTS5 index, filters, groups
//! * [`carta`]   — RAG-CARTA: summarize / extract / synthesize with citations
//! * [`grab`]    — web-grab: web pages into the library
//! * [`api`], [`server`] — the local UI

pub mod api;
pub mod carta;
pub mod extract;
pub mod grab;
pub mod library;
pub mod server;
