// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Project Archetype Classifier (Node, Electron, VSCode, Discord, TS, Unreal, .NET, Rust).

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectArchetype {
    ElectronDesktopApp,
    VsCodeWorkbench,
    DiscordClientOrBot,
    TypeScriptCompilerOrSdk,
    UnrealEngineMonorepo,
    MicrosoftDotNetNodeInterop,
    LegacyNodePackage,
    SovereignBunRustMonorepo,
}

impl ProjectArchetype {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ElectronDesktopApp => "electron-desktop",
            Self::VsCodeWorkbench => "vscode-workbench",
            Self::DiscordClientOrBot => "discord-client-bot",
            Self::TypeScriptCompilerOrSdk => "typescript-compiler-sdk",
            Self::UnrealEngineMonorepo => "unreal-engine-monorepo",
            Self::MicrosoftDotNetNodeInterop => "microsoft-dotnet-node",
            Self::LegacyNodePackage => "legacy-node-package",
            Self::SovereignBunRustMonorepo => "sovereign-bun-rust-monorepo",
        }
    }

    pub const fn target_stack(self) -> &'static str {
        match self {
            Self::ElectronDesktopApp => "Zed GPUI / Tauri Shell + Bun Native Sidecar",
            Self::VsCodeWorkbench => "Zed GPUI DirectX 12 / Vulkan Workbench + Bun Extension Host",
            Self::DiscordClientOrBot => "Bun.serve WebSocket + Rust Audio FFI",
            Self::TypeScriptCompilerOrSdk => "Bun v1.4.4 check/fmt + Bun build --compile AOT",
            Self::UnrealEngineMonorepo => "Rust LTO C-ABI Kernel + Bun UBT Orchestrator",
            Self::MicrosoftDotNetNodeInterop => "Bun bun:ffi + [UnmanagedCallersOnly] C-ABI",
            Self::LegacyNodePackage => "Bun v1.4.4 Full-Rust + bun:sqlite + bun test",
            Self::SovereignBunRustMonorepo => "Bun v1.4.4 + Rust 2024 LTO + depot",
        }
    }
}

/// Detects the project archetype from directory contents and package.json.
pub fn detect_archetype(root: &Path) -> ProjectArchetype {
    let pkg_json_path = root.join("package.json");
    if pkg_json_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&pkg_json_path) {
            let lower = content.to_ascii_lowercase();
            if lower.contains("\"electron\"") {
                return ProjectArchetype::ElectronDesktopApp;
            }
            if lower.contains("vscode") || lower.contains("\"@types/vscode\"") {
                return ProjectArchetype::VsCodeWorkbench;
            }
            if lower.contains("discord.js") || lower.contains("\"@discordjs/") {
                return ProjectArchetype::DiscordClientOrBot;
            }
            if lower.contains("\"typescript\"") && lower.contains("\"compiler\"") {
                return ProjectArchetype::TypeScriptCompilerOrSdk;
            }
        }
    }

    if root.join("bun.lock").exists() || root.join("bunfig.toml").exists() {
        return ProjectArchetype::SovereignBunRustMonorepo;
    }

    if root.join("package.json").exists() {
        return ProjectArchetype::LegacyNodePackage;
    }

    ProjectArchetype::SovereignBunRustMonorepo
}
