//! Two `effect!` blocks in the same module both default to the `Op` root, so
//! the sentry type that guards each root gets defined twice. The redefinition
//! error is what names the conflicting root back to the user.

use algae::prelude::*;

effect! {
    Console::Print (String) -> ();
    Console::ReadLine -> String;
}

effect! {
    Math::Add ((i32, i32)) -> i32;
    Math::Multiply ((i32, i32)) -> i32;
}

fn main() {}
