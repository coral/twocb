use serde::Serialize;
use std::ffi::c_void;
use ts_rs::TS;

/// Declares `WorldStateBuffer` and generates `register_v8_accessors()`.
///
/// Fields prefixed with `accessor` get an automatic read-only V8 property on the
/// `world` object.  Fields prefixed with `accessor(name = "js_name")` use a custom
/// JS property name.  All other fields are part of the struct but invisible to JS
/// (register custom accessors manually if needed).
///
/// Adding a new `accessor some_field: f64,` line is all that's needed — no JS changes.
macro_rules! define_world_state {
    // ── entry point ──────────────────────────────────────────────────
    (
        consts { $($cname:ident = $cval:expr;)* }
        fields { $($rest:tt)* }
    ) => {
        $(pub const $cname: usize = $cval;)*
        define_world_state!(@parse [] [] $($rest)*);
    };

    // ── accessor field (default JS name = field name) ────────────────
    (@parse [$($fields:tt)*] [$($acc:tt)*]
        accessor $field:ident : f64, $($rest:tt)*
    ) => {
        define_world_state!(@parse
            [$($fields)* pub $field: f64,]
            [$($acc)* ($field, stringify!($field)),]
            $($rest)*
        );
    };

    // ── accessor field with explicit JS name ─────────────────────────
    (@parse [$($fields:tt)*] [$($acc:tt)*]
        accessor(name = $jsname:literal) $field:ident : f64, $($rest:tt)*
    ) => {
        define_world_state!(@parse
            [$($fields)* pub $field: f64,]
            [$($acc)* ($field, $jsname),]
            $($rest)*
        );
    };

    // ── non-accessor field (any type) ────────────────────────────────
    (@parse [$($fields:tt)*] [$($acc:tt)*]
        $field:ident : $fty:ty, $($rest:tt)*
    ) => {
        define_world_state!(@parse
            [$($fields)* pub $field: $fty,]
            [$($acc)*]
            $($rest)*
        );
    };

    // ── terminal: emit struct + accessor registration ────────────────
    (@parse [$($fields:tt)*] [$(($afield:ident, $aname:expr)),* $(,)?]) => {
        #[repr(C)]
        pub struct WorldStateBuffer {
            $($fields)*
        }

        impl WorldStateBuffer {
            /// Register read-only V8 accessors for all `accessor` fields.
            /// Called once per pattern isolate during `setup_shared_buffers()`.
            pub fn register_v8_accessors(
                &self,
                scope: &mut v8::PinScope<'_, '_>,
                obj: v8::Local<v8::Object>,
            ) {
                $(
                    {
                        let key = v8::String::new(scope, $aname).unwrap();
                        let ptr = &self.$afield as *const f64 as *mut c_void;
                        let ext = v8::External::new(scope, ptr);
                        let config = v8::AccessorConfiguration::new(world_f64_getter)
                            .data(ext.into())
                            .property_attribute(v8::PropertyAttribute::READ_ONLY);
                        obj.set_accessor_with_configuration(scope, key.into(), config);
                    }
                )*
            }
        }
    };
}

define_world_state! {
    consts {
        MAX_NOTES = 12;
        FOLDED_BINS = 24;
    }
    fields {
        // Timing — each becomes world.framerate, world.delta, etc.
        accessor framerate: f64,
        accessor(name = "index") frame_index: f64,
        accessor delta: f64,
        accessor phase: f64,
        accessor bar: f64,

        // Tempo
        accessor bpm: f64,
        accessor tempo_confidence: f64,
        accessor tempo_period: f64,

        // Colorchord (custom accessor registered separately)
        note_count: f64,
        notes: [f64; MAX_NOTES * 3],
        folded: [f64; FOLDED_BINS],

        // Pixel info
        accessor pixel_count: f64,

        // Reserved for future inputs
        reserved: [f64; 16],
    }
}

/// Total number of f64 fields in the buffer.
/// Must match the layout of WorldStateBuffer exactly.
pub const BUFFER_LEN: usize = {
    5  // timing: framerate, frame_index, delta, phase, bar
  + 3  // tempo: bpm, confidence, period
  + 1  // note_count
  + MAX_NOTES * 3  // notes (amp, amp_filt, position)
  + FOLDED_BINS    // folded bins
  + 1  // pixel_count
  + 16 // reserved
};

impl WorldStateBuffer {
    pub fn new() -> Self {
        // Safety: all-zeros is valid for f64
        unsafe { std::mem::zeroed() }
    }

    /// Interpret the struct as a mutable f64 slice (zero-copy).
    pub fn as_f64_slice(&self) -> &[f64] {
        unsafe { std::slice::from_raw_parts(self as *const Self as *const f64, BUFFER_LEN) }
    }

    pub fn as_f64_slice_mut(&mut self) -> &mut [f64] {
        unsafe { std::slice::from_raw_parts_mut(self as *mut Self as *mut f64, BUFFER_LEN) }
    }
}

// ── V8 getter callbacks ──────────────────────────────────────────────

/// Generic getter for scalar f64 world-state fields.
/// The `data` slot carries a `v8::External` wrapping `*const f64`.
fn world_f64_getter(
    scope: &mut v8::PinScope<'_, '_>,
    _key: v8::Local<'_, v8::Name>,
    args: v8::PropertyCallbackArguments<'_>,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let ext = unsafe { v8::Local::<v8::External>::cast_unchecked(args.data()) };
    let value = unsafe { *(ext.value() as *const f64) };
    rv.set(v8::Number::new(scope, value).into());
}

/// Getter for `world.colorchord` — builds `{ notes: [{amp, amp_filt, position}, ...], folded: [...] }`.
/// The `data` slot carries a `v8::External` wrapping `*const WorldStateBuffer`.
pub fn colorchord_getter(
    scope: &mut v8::PinScope<'_, '_>,
    _key: v8::Local<'_, v8::Name>,
    _args: v8::PropertyCallbackArguments<'_>,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let ext = unsafe { v8::Local::<v8::External>::cast_unchecked(_args.data()) };
    let buf = unsafe { &*(ext.value() as *const WorldStateBuffer) };

    let count = buf.note_count as usize;
    let notes_arr = v8::Array::new(scope, count as i32);
    for i in 0..count.min(MAX_NOTES) {
        let base = i * 3;
        let note_obj = v8::Object::new(scope);

        let k_amp = v8::String::new(scope, "amp").unwrap();
        let v_amp = v8::Number::new(scope, buf.notes[base]);
        note_obj.set(scope, k_amp.into(), v_amp.into());

        let k_af = v8::String::new(scope, "amp_filt").unwrap();
        let v_af = v8::Number::new(scope, buf.notes[base + 1]);
        note_obj.set(scope, k_af.into(), v_af.into());

        let k_pos = v8::String::new(scope, "position").unwrap();
        let v_pos = v8::Number::new(scope, buf.notes[base + 2]);
        note_obj.set(scope, k_pos.into(), v_pos.into());

        notes_arr.set_index(scope, i as u32, note_obj.into());
    }

    let folded_arr = v8::Array::new(scope, FOLDED_BINS as i32);
    for j in 0..FOLDED_BINS {
        let val = v8::Number::new(scope, buf.folded[j]);
        folded_arr.set_index(scope, j as u32, val.into());
    }

    let result = v8::Object::new(scope);
    let k_notes = v8::String::new(scope, "notes").unwrap();
    result.set(scope, k_notes.into(), notes_arr.into());
    let k_folded = v8::String::new(scope, "folded").unwrap();
    result.set(scope, k_folded.into(), folded_arr.into());

    rv.set(result.into());
}

// ── TypeScript-friendly types (for .d.ts generation) ─────────────────

#[derive(Serialize, TS)]
#[ts(export, export_to = "../../files/types/")]
pub struct WorldState {
    pub framerate: f64,
    pub index: f64,
    pub delta: f64,
    pub phase: f64,
    pub bar: f64,
    pub bpm: f64,
    pub colorchord: ColorchordState,
}

#[derive(Serialize, TS)]
#[ts(export, export_to = "../../files/types/")]
pub struct ColorchordState {
    pub notes: Vec<NoteState>,
    pub folded: Vec<f64>,
}

#[derive(Serialize, TS)]
#[ts(export, export_to = "../../files/types/")]
pub struct NoteState {
    pub amp: f64,
    pub amp_filt: f64,
    pub position: f64,
}

// Compile-time check that BUFFER_LEN matches the struct size
const _: () = {
    assert!(std::mem::size_of::<WorldStateBuffer>() == BUFFER_LEN * std::mem::size_of::<f64>());
};
