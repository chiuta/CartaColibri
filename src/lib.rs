//! CartaColibri — a 100% local AI library. MIT licensed, clean-room.
//!
//! * [`library`] — RAG-fetch: incremental SQLite-FTS5 index, filters, groups
//! * [`carta`]   — RAG-CARTA: summarize / extract / synthesize with citations
//! * [`grab`]    — web-grab: web pages into the library
//! * [`api`], [`server`] — the local UI

pub mod api;
pub mod carta;
pub mod doctor;
pub mod extract;
pub mod feeds;
pub mod grab;
pub mod import;
pub mod library;
pub mod organize;
pub mod server;
pub mod tour;
