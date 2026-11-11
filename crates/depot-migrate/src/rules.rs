// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Codemod rule definitions mapping legacy Node/CommonJS patterns to Bun v1.4.4 standards.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodemodRule {
    pub id: &'static str,
    pub description: &'static str,
    pub legacy_pattern: &'static str,
    pub replacement: &'static str,
}

pub const MIGRATION_RULES: &[CodemodRule] = &[
    CodemodRule {
        id: "NODE-BUILTIN-SCHEME",
        description: "Enforce node: prefix on Node.js core builtins",
        legacy_pattern: "require('fs')",
        replacement: "require('node:fs')",
    },
    CodemodRule {
        id: "FETCH-REPLACE-AXIOS",
        description: "Replace axios/node-fetch with native global fetch",
        legacy_pattern: "axios.get(",
        replacement: "fetch(",
    },
    CodemodRule {
        id: "BUN-SQLITE",
        description: "Replace sqlite3/better-sqlite3 with bun:sqlite",
        legacy_pattern: "require('sqlite3')",
        replacement: "require('bun:sqlite')",
    },
    CodemodRule {
        id: "BUN-SERVE-WS",
        description: "Replace ws library with native Bun.serve websocket",
        legacy_pattern: "new WebSocket.Server({",
        replacement: "Bun.serve({ websocket: {",
    },
    CodemodRule {
        id: "BUN-PASSWORD",
        description: "Replace bcrypt/argon2 with Bun.password",
        legacy_pattern: "bcrypt.hash(",
        replacement: "Bun.password.hash(",
    },
    CodemodRule {
        id: "LUA52-TABLE-UNPACK",
        description: "Migrate Lua 5.1 global unpack() to Lua 5.2 table.unpack()",
        legacy_pattern: "unpack(",
        replacement: "table.unpack(",
    },
    CodemodRule {
        id: "LUA52-BIT32",
        description: "Migrate LuaJIT/LuaBitOp bit.* calls to PUC-Rio Lua 5.2 bit32.*",
        legacy_pattern: "bit.band(",
        replacement: "bit32.band(",
    },
];
