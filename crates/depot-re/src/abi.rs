// SPDX-License-Identifier: Apache-2.0 OR MIT
//! ABI & Symbol Classifier for C++, Unreal Engine 5.8, Rust, and MSVC symbols.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CppAbiClass {
    PlainOrC,
    ItaniumCpp,
    MsvcCpp,
    RustMangled,
    UnrealEngineReflection,
}

impl CppAbiClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PlainOrC => "c",
            Self::ItaniumCpp => "itanium",
            Self::MsvcCpp => "msvc",
            Self::RustMangled => "rust",
            Self::UnrealEngineReflection => "unreal",
        }
    }
}

/// Classifies a symbol or byte slice into its ABI family.
pub fn classify_symbol_abi(symbol: &str) -> CppAbiClass {
    if symbol.starts_with("_R") || symbol.starts_with("__R") || symbol.starts_with("_ZN") && symbol.contains("17h") {
        return CppAbiClass::RustMangled;
    }
    if symbol.starts_with('?') || symbol.starts_with("??") {
        return CppAbiClass::MsvcCpp;
    }
    if symbol.starts_with("_Z") {
        return CppAbiClass::ItaniumCpp;
    }
    if symbol.starts_with("Z_Construct_")
        || symbol.starts_with("StaticRegisterNatives")
        || symbol.contains("_Statics")
        || symbol.starts_with("UClass_")
        || symbol.starts_with("UScriptStruct_")
    {
        return CppAbiClass::UnrealEngineReflection;
    }
    CppAbiClass::PlainOrC
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_classification() {
        assert_eq!(classify_symbol_abi("printf"), CppAbiClass::PlainOrC);
        assert_eq!(classify_symbol_abi("?hello@@YAHXZ"), CppAbiClass::MsvcCpp);
        assert_eq!(classify_symbol_abi("_Z4funcv"), CppAbiClass::ItaniumCpp);
        assert_eq!(classify_symbol_abi("_RNvCs1234_4main4test"), CppAbiClass::RustMangled);
        assert_eq!(classify_symbol_abi("Z_Construct_UClass_AHeroActor"), CppAbiClass::UnrealEngineReflection);
    }
}
