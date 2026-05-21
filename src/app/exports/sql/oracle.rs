//! Oracle SQL DDL exporter (Oracle 23c+ compatible).
//!
//! Two-phase generation:
//! 1. CREATE DOMAIN + CREATE TABLE (no FKs)
//! 2. ALTER TABLE ... ADD CONSTRAINT ... FOREIGN KEY
