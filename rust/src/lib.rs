use godot::prelude::*;

struct BazookaExtension;

#[gdextension]
unsafe impl ExtensionLibrary for BazookaExtension {}
