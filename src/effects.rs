//! The effect lattice, section 8 of the specification.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Effects(pub u32);

impl Effects {
    pub const NONE: Effects = Effects(0);
    pub const ALLOCATES: Effects = Effects(1 << 0);
    pub const REFCOUNTS: Effects = Effects(1 << 1);
    pub const BLOCKS: Effects = Effects(1 << 2);
    pub const SHARED_MUTABLE: Effects = Effects(1 << 3);
    pub const NONDETERMINISTIC: Effects = Effects(1 << 4);
    pub const PANICS: Effects = Effects(1 << 5);
    pub const FFI: Effects = Effects(1 << 6);
    pub const UNBOUNDED_STACK: Effects = Effects(1 << 7);
    pub const ALL: Effects = Effects(0xff);

    pub const NAMES: &'static [(&'static str, Effects)] = &[
        ("allocates", Effects::ALLOCATES),
        ("refcounts", Effects::REFCOUNTS),
        ("blocks", Effects::BLOCKS),
        ("shared_mutable", Effects::SHARED_MUTABLE),
        ("nondeterministic", Effects::NONDETERMINISTIC),
        ("panics", Effects::PANICS),
        ("ffi", Effects::FFI),
        ("unbounded_stack", Effects::UNBOUNDED_STACK),
    ];

    pub fn from_name(s: &str) -> Option<Effects> {
        Effects::NAMES.iter().find(|(n, _)| *n == s).map(|(_, e)| *e)
    }
    pub fn union(self, o: Effects) -> Effects {
        Effects(self.0 | o.0)
    }
    pub fn contains(self, o: Effects) -> bool {
        self.0 & o.0 == o.0
    }
    pub fn intersect(self, o: Effects) -> Effects {
        Effects(self.0 & o.0)
    }
    pub fn without(self, o: Effects) -> Effects {
        Effects(self.0 & !o.0)
    }
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
    pub fn names(self) -> Vec<&'static str> {
        Effects::NAMES.iter().filter(|(_, e)| self.contains(*e)).map(|(n, _)| *n).collect()
    }
    /// Render as a signature suffix: `allocates panics`
    pub fn render(self) -> String {
        self.names().join(" ")
    }
    /// Render the negative form: `!allocates !refcounts` for effects not present.
    pub fn render_negative(self) -> String {
        Effects::NAMES.iter().filter(|(_, e)| !self.contains(*e)).map(|(n, _)| format!("!{}", n)).collect::<Vec<_>>().join(" ")
    }
}
